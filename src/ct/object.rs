// Copyright 2017 Gitai<i@gitai.me> All rights reserved.
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify,
// merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall
// be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR
// ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
// CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Additional API for Object Operations

use aws_sdk_rust::aws::common::signature::SignedRequest;
pub use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;

use aws_sdk_rust::aws::s3::bucket::*;
pub use aws_sdk_rust::aws::s3::object::*;

use aws_sdk_rust::aws::errors::s3::S3Error;

use ct::sdk::CTClient;
use ct::sdk::CTSignedRequest;

//#[derive(Debug, Default)]
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct PresignedObjectRequest {
    pub bucket: BucketName,
    pub expires: Option<Expires>,
    pub key: ObjectKey,
}

/// A trait to additional pre-signed for CTClient.
pub trait CTClientObject<P> {
    /// Generate a pre-signed url for an S3 object, the returned url can be shared.
    /// ```
    /// match s3.presigned_object() {
    ///     Ok(out) => println!("{:#?}", out),
    ///     Err(err) => println!("{:#?}", err),
    /// }
    /// ```
    fn presigned_object(&self, input: &PresignedObjectRequest)
                        -> Result<String, S3Error>;
}

impl<P> CTClientObject<P> for CTClient<P>
    where P: AwsCredentialsProvider,
{
    fn presigned_object(&self, input: &PresignedObjectRequest)
                        -> Result<String, S3Error>
    {
        let mut request = SignedRequest::new("GET",
                                             "s3",
                                             self.region(),
                                             &input.bucket,
                                             &format!("/{}", input.key),
                                             self.endpoint());

        // TODO: new PR for make the methor public
        let hostname = self.hostname(Some(&input.bucket));

        request.set_hostname(Some(hostname));

        let (date, signature) = request.presigned(
            &self.credentials_provider().credentials().unwrap(),
            &input.expires);

        request.remove_header("authorization");
        request.remove_header("content-length");
        request.remove_header("content-type");
        request.remove_header("user-agent");
        request.remove_header("date");
        request.remove_header("host");


        request.add_header_raw("Signature", signature.as_ref());
        request.add_header_raw("Expires", date.as_ref());
        request.add_header_raw("AWSAccessKeyId", &self.credentials_provider().credentials().unwrap().aws_access_key_id());


        let url = request.gen_url();

        Ok(url)
    }
}

use aws_sdk_rust::aws::common::common::Operation;

pub trait CTClientEncryptionObject<P> {
    fn put_object_securely(&self, input: &PutObjectRequest, operation: Option<&mut Operation>)
                           -> Result<PutObjectOutput, S3Error>;
}

impl<P> CTClientEncryptionObject<P> for CTClient<P>
    where P: AwsCredentialsProvider,
{
    #[allow(unused_variables)]
    fn put_object_securely(&self, input: &PutObjectRequest, operation: Option<&mut Operation>)
                           -> Result<PutObjectOutput, S3Error>
    {
        // self.multipart_upload_create()
        // encrypt_file(input.body);
        println!("put_object_securely");
        Err(S3Error {
            message: String::new(),
            ..Default::default()
        })
    }
}


// use openssl::rsa::Rsa;

use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::Read;
use std::io::Write;

use crypto::buffer::WriteBuffer;
use crypto::buffer::ReadBuffer;

use crypto::*;
use crypto::buffer::*;


/// https://github.com/DaGenix/rust-crypto/issues/330
fn encrypt_file(src_file: &Path, dest_file: &Path, key: &Vec<u8>)
                -> Result<(), symmetriccipher::SymmetricCipherError> {
    let mut input = File::open(src_file).unwrap();
    let mut output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dest_file).unwrap();

    let iv = &[0u8; 16];

    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize128,
        key,
        iv,
        blockmodes::PkcsPadding);

    let file_size = input.metadata().unwrap().len();
    let mut bytes_read: u64 = 0;
    let buffer_size = 4096;
    let mut data = vec![0u8; buffer_size];

    loop {
        let result = input.read(&mut data);
        match result {
            Ok(size) => {
                bytes_read += size as u64;
                if size == 0 {
                    break;
                } else if size > 0 && size < buffer_size {
                    data.truncate(size);
                }
            }
            Err(err) => { println!("Error in read file: {:?}", err); }
        };

        let mut read_buffer = buffer::RefReadBuffer::new(&data);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, bytes_read == file_size)
                .expect("encrypt data");
            output.write_all(write_buffer.take_read_buffer().take_remaining()).expect("write file");

            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }
        if bytes_read == file_size {
            break;
        }
    }
    println!("file size: {} vs {}", file_size, output.metadata().unwrap().len());
    Ok(())
}