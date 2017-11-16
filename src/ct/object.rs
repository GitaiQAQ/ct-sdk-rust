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
//!
use md5;
use rustc_serialize::base64::{ToBase64, STANDARD};

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
pub trait CTClientObject {
    /// Generate a pre-signed url for an S3 object, the returned url can be shared.
    /// ```
    /// match s3.presigned_object() {
    ///     Ok(out) => println!("{:#?}", out),
    ///     Err(err) => println!("{:#?}", err),
    /// }
    /// ```
    fn presigned_object(&self, input: &PresignedObjectRequest) -> Result<String, S3Error>;
}

impl CTClientObject for CTClient {
    fn presigned_object(&self, input: &PresignedObjectRequest) -> Result<String, S3Error> {
        let mut request = SignedRequest::new(
            "GET",
            "s3",
            self.region(),
            &input.bucket,
            &format!("/{}", input.key),
            self.endpoint(),
        );

        // TODO: new PR for make the methor public
        let hostname = self.hostname(Some(&input.bucket));

        request.set_hostname(Some(hostname));

        let (date, signature) = request.presigned(
            &self.credentials_provider().credentials().unwrap(),
            &input.expires,
        );

        request.remove_header("authorization");
        request.remove_header("content-length");
        request.remove_header("content-type");
        request.remove_header("user-agent");
        request.remove_header("date");
        request.remove_header("host");


        request.add_header_raw("Signature", signature.as_ref());
        request.add_header_raw("Expires", date.as_ref());
        request.add_header_raw(
            "AWSAccessKeyId",
            &self.credentials_provider()
                .credentials()
                .unwrap()
                .aws_access_key_id(),
        );


        let url = request.gen_url();

        Ok(url)
    }
}

use aws_sdk_rust::aws::common::common::Operation;


//#[derive(Debug, Default)]
#[derive(Debug, Default, Clone, RustcDecodable, RustcEncodable)]
pub struct PostObjectRequest {}

//#[derive(Debug, Default)]
#[derive(Debug, Default, Clone, RustcDecodable, RustcEncodable)]
pub struct PostObjectOutput {}

pub trait CTClientEncryptionObject {
    fn put_object_securely(
        &self,
        input: PutObjectRequest,
        operation: Option<&mut Operation>,
    ) -> Result<PutObjectOutput, S3Error>;

    fn get_object_securely(
        &self,
        input: &GetObjectRequest,
        operation: Option<&mut Operation>,
    ) -> Result<GetObjectOutput, S3Error>;

    fn post_object(&self, input: &PostObjectRequest) -> Result<PostObjectOutput, S3Error>;
}

impl CTClientEncryptionObject for CTClient {
    fn put_object_securely(
        &self,
        input: PutObjectRequest,
        operation: Option<&mut Operation>,
    ) -> Result<PutObjectOutput, S3Error> {
        // let mut request = PutObjectRequest::from(*input);
        // input.metadata.unwrap().len() as usize
        let mut cipherbody = vec![0u8; 4096];
        {
            let plaintext = input.body.unwrap();
            cipherbody = vec![0u8; plaintext.len() + (16 - plaintext.len() % 16)];
            encrypt(&plaintext, &mut cipherbody);
            //let mut plain_out: Vec<u8> = repeat(0).take(plaintext.len()).collect();
            //decrypt(&cipherbody, &mut plain_out);
            // println!("cipherbody {:?}", String::from_utf8_lossy(&cipherbody));
        }

        let mut request = PutObjectRequest::from(input);
        request.body = Some(&cipherbody);

        // Compute hash - Hash is slow
        let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
        request.content_md5 = Some(hash);

        self.put_object(&request, operation)
    }

    fn get_object_securely(
        &self,
        input: &GetObjectRequest,
        operation: Option<&mut Operation>,
    ) -> Result<GetObjectOutput, S3Error> {
        match self.get_object(input, operation) {
            Ok(out) => {
                let mut output = GetObjectOutput::from(out);

                let mut plaintext: Vec<u8>;
                {
                    plaintext = vec![0u8; output.get_body().len()];
                }
                {
                    let cipherbody = &output.get_body();
                    decrypt(&cipherbody, &mut plaintext);
                    // println!("plaintext {:?}", String::from_utf8_lossy(&plaintext));
                }
                if output.is_body {
                    output.body = plaintext;
                } else {
                    output.body_buffer = plaintext;
                }
                Ok(output)
            }
            Err(err) => Err(err),
        }
    }

    fn post_object(&self, input: &PostObjectRequest) -> Result<PostObjectOutput, S3Error> {
        unimplemented!()
    }
}

// use openssl::rsa::Rsa;

use crypto::*;
use crypto::buffer::*;
use std::iter::repeat;

/// The size of the buffers used when reading and decompressing data
pub const BUFFER_SIZE: usize = 4096;

fn encrypt(plaintext: &[u8], ciphertext: &mut [u8]) {
    let key: Vec<u8> = repeat(1).take(16).collect();
    let iv: Vec<u8> = repeat(3).take(16).collect();

    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        &key[..],
        &iv.clone(),
        blockmodes::PkcsPadding,
    );

    {
        let mut buff_in = RefReadBuffer::new(&plaintext);
        let mut buff_out = RefWriteBuffer::new(ciphertext);

        let result = encryptor.encrypt(&mut buff_in, &mut buff_out, true);

        match result {
            Ok(_buffer_underflow) => {}
            Err(err) => panic!("Error {:?}", err),
        }
    }
}


fn decrypt(ciphertext: &[u8], plaintext: &mut [u8]) {
    let key: Vec<u8> = repeat(1).take(16).collect();
    let iv: Vec<u8> = repeat(3).take(16).collect();

    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        &key[..],
        &iv.clone(),
        blockmodes::PkcsPadding,
    );

    {
        let mut buff_in = RefReadBuffer::new(&ciphertext);
        let mut buff_out = RefWriteBuffer::new(plaintext);

        match decryptor.decrypt(&mut buff_in, &mut buff_out, true) {
            Ok(_buffer_underflow) => {}
            Err(err) => panic!("Error {:?}", err),
        }
    }
}
