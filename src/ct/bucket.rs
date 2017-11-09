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

pub use aws_sdk_rust::aws::s3::acl::CannedAcl::*;
pub use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;
pub use aws_sdk_rust::aws::s3::bucket::*;

#[cfg(test)]
mod tests {
    use aws_sdk_rust::aws::common::credentials::*;
    use aws_sdk_rust::aws::s3::acl::CannedAcl;
    use aws_sdk_rust::aws::s3::bucket::*;
    use aws_sdk_rust::aws::s3::acl::*;

    use super::super::sdk::CTClient;

    static BUCKET: &'static str = "gitai.test";

    #[test]
    fn _create() {
        let provider = DefaultCredentialsProvider::new(None).unwrap();
        let s3 = CTClient::default_ctyun_client(provider);

        match s3.create_bucket(&CreateBucketRequest {
            bucket: String::from(BUCKET),
            ..Default::default()
        }) {
            Ok(_) => {},
            Err(err) => assert!(false, err),
        }

        match s3.head_bucket(&HeadBucketRequest{
            bucket: String::from(BUCKET)
        }){
            Ok(_) => assert!(true),
            Err(err) => assert!(false, err),
        };
    }

    #[test]
    fn _list() {
        let provider = DefaultCredentialsProvider::new(None).unwrap();
        let s3 = CTClient::default_ctyun_client(provider);

        match s3.list_buckets() {
            Ok(out) => {
                print!("{:?}", out);
                for bucket in out.buckets {
                    if bucket.name == BUCKET {
                        return
                    }
                }
                assert!(false, format!("Bucket {} not found", BUCKET))
            },
            Err(err) => assert!(false, err),
        }
    }

    #[test]
    fn _acl() {
        let provider = DefaultCredentialsProvider::new(None).unwrap();
        let s3 = CTClient::default_ctyun_client(provider);

        match s3.put_bucket_acl(&PutBucketAclRequest {
            bucket: String::from(BUCKET),
            acl: Some(CannedAcl::PublicReadWrite),
            ..Default::default()
        }) {
            Ok(_) => {},
            Err(err) => assert!(false, err),
        };

        match s3.get_bucket_acl(&GetBucketAclRequest{
            bucket: String::from(BUCKET),
        }) {
            Ok(acl) => {
                for grant in acl.acl.grants {
                    if grant.permission.eq("FULL_CONTROL") {
                        assert!(true);
                        return
                    }
                }
                assert!(false, format!("FULL_CONTROL not found in bucket {}`s permission", BUCKET))
            },
            Err(err) => assert!(false, err),
        };
    }

    #[test]
    fn _delete() {
        let provider = DefaultCredentialsProvider::new(None).unwrap();
        let s3 = CTClient::default_ctyun_client(provider);

        match s3.delete_bucket(&DeleteBucketRequest{
            bucket: String::from(BUCKET),
        }) {
            Ok(_) => {},
            Err(err) => assert!(false, err),
        }

        match s3.head_bucket(&HeadBucketRequest{
            bucket: String::from(BUCKET)
        }){
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        };
    }
}