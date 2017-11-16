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

use ct_sdk::ct::s3::acl::CannedAcl;
use ct_sdk::ct::s3::bucket::*;
use ct_sdk::ct::s3::acl::*;
use ct_sdk::ct::sdk::CTClient;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

/// List buckets(ls)
pub fn list(ct: &CTClient) {
    debug!("List Bucket");
    match ct.list_buckets() {
        Ok(out) => printstd!(out.buckets, name, creation_date),
        Err(err) => print_aws_err!(err),
    }
}

/// 创建一个 Bucket
/// Creates an BUCKET(mb)
pub fn create(ct: &CTClient, name: String) {
    debug!("Create Bucket");
    match ct.create_bucket(&CreateBucketRequest {
        bucket: name.clone(),
        ..Default::default()
    }) {
        Ok(out) => println!("Create {} SUCCESS in {}", name, out.location),
        Err(err) => print_aws_err!(err),
    }
}

// TODO: 更改创建的 Bucket属性（私有、公有、只读）
pub fn acl(ct: &CTClient, name: String, acl: CannedAcl) {
    debug!("acl");
    match ct.put_bucket_acl(&PutBucketAclRequest {
        bucket: name.clone(),
        acl: Some(acl),
        ..Default::default()
    }) {
        Ok(_) => println!("Update ACL of {} SUCCESS.", name),
        Err(err) => print_aws_err!(err),
    };
}

/// 删除已创建的 Bucket
/// Deletes an empty BUCKET.(rb)
/// A BUCKET must be completely empty of objects and versioned objects before it can be deleted.
/// However, the --force parameter can be used to delete the non-versioned objects in the BUCKET
/// before the BUCKET is deleted.
pub fn delete(ct: &CTClient, name: String) {
    debug!("Delete Bucket");
    match ct.delete_bucket(&DeleteBucketRequest {
        bucket: name.clone(),
        ..Default::default()
    }) {
        Ok(_) => println!("Remove {} SUCCESS", name),
        Err(err) => print_aws_err!(err),
    }
}