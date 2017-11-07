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

use hyper::client::Client;

use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;
use aws_sdk_rust::aws::s3::s3client::S3Client;

pub use prettytable::Table;
pub use prettytable::row::Row;
pub use prettytable::cell::Cell;
pub use prettytable::format::FormatBuilder;

use ct_sdk::sdk::iam::*;

pub trait CTCLIAM {
    fn list(&self);
    fn create(&self, quiet:bool, bucket:&str);
    fn delete(&self, quiet:bool, bucket:&str);
    fn update(&self, quiet:bool, bucket:&str);
}

impl<P> CTCLIAM for S3Client<P, Client>
    where P: AwsCredentialsProvider,
{
    fn list(&self) {
        match self.list_access_key(&ListAccessKeyRequest {
            ..Default::default()
        }) {
            Ok(out) => {println!("{:?}", out)},
            Err(err) => println!("{:?}", err),
        }
    }

    /// 创建一组 AK/SK
    fn create(&self, quiet: bool, bucket: &str) {
        unimplemented!()
    }

    /// 删除已有的 AK/SK
    fn delete(&self, quiet: bool, bucket: &str) {
        unimplemented!()
    }

    /// 更改 AK/SK属性（主秘钥/普通秘钥）
    fn update(&self, quiet: bool, bucket: &str) {
        unimplemented!()
    }
}