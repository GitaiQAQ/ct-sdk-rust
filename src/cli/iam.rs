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

use ct_sdk::sdk::CTClient;

pub use prettytable::Table;
pub use prettytable::row::Row;
pub use prettytable::cell::Cell;
pub use prettytable::format::FormatBuilder;

use ct_sdk::sdk::iam::*;

pub trait CTCLIAM {
    fn list(&self);
    fn create(&self);
    fn delete(&self, access_key_id:String);
    fn update(&self, access_key_id:String);
}

impl<P> CTCLIAM for CTClient<P>
    where P: AwsCredentialsProvider,
{
    fn list(&self) {
        match self.list_access_key(&ListAccessKeyRequest {
            ..Default::default()
        }) {
            Ok(out) => printstd!(out.access_key_metadata.member, user_name, access_key_id, status, is_primary),
            Err(err) => println!("{:?}", err),
        }
    }

    /// 创建一组 AK/SK
    fn create(&self) {
        match self.create_access_key() {
            Ok(out) => {println!("{:?}", out)},
            Err(err) => println!("{:?}", err),
        }
    }

    /// 删除已有的 AK/SK
    fn delete(&self, access_key_id:String) {
        match self.delete_access_key(&DeleteAccessKeyRequest{
            access_key_id,
        }) {
            Ok(out) => {println!("{:?}", out)},
            Err(err) => println!("{:?}", err),
        }
    }

    /// 更改 AK/SK属性（主秘钥/普通秘钥）
    fn update(&self, access_key_id:String) {
        match self.update_access_key(&UpdateAccessKeyRequest{
            access_key_id,
            status: Status::Inactive,
            is_primary: true,
        }) {
            Ok(out) => {println!("{:?}", out)},
            Err(err) => println!("{:?}", err),
        }
    }
}