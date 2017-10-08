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
use hyper::client::Client;

use aws_sdk_rust::aws::common::signature::SignedRequest;
use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;
use aws_sdk_rust::aws::s3::s3client::S3Client;

use aws_sdk_rust::aws::s3::bucket::*;
use aws_sdk_rust::aws::s3::object::*;

use aws_sdk_rust::aws::errors::s3::S3Error;

use ct::sdk::CTSignedRequest;

//#[derive(Debug, Default)]
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct PresignedObjectRequest {
    pub bucket: BucketName,
    pub expires: Option<Expires>,
    pub key: ObjectKey,
}

/// A trait to additional pre-signed for S3Client.
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

impl<P> CTClientObject<P> for S3Client<P, Client>
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