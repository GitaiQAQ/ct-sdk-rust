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

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use md5;

use rustc_serialize::base64::{STANDARD, ToBase64};

use ct_sdk::sdk::CTClient;
use ct_sdk::sdk::object::*;

pub use prettytable::Table;
pub use prettytable::row::Row;
pub use prettytable::cell::Cell;
pub use prettytable::format::FormatBuilder;

/// High-level OOS object operations commands
/// Like http://docs.aws.amazon.com/cli/latest/reference/s3/index.html

/// Additional object operations commands for CTClient.
pub trait CTCLIObject {
    fn list(&self, quiet: bool, bucket: String, prefix: Option<String>);
    fn new(&self, bucket: String, key: String, body: String);

    /// 下载已上传的 Object到本地（Download）
    fn get(&self, bucket: String, key: String, path: &Path);
    fn put(&self, bucket: String, key: String, path: &Path);

    fn put_securely(&self, bucket: String, key: String, path: &Path);

    /// 删除已上传的 Object（Delete）
    /// Deletes an object(rm)
    fn delete(&self, bucket: String, key: String);

    /// 分享已上传的 Object（Share）
    /// presign
    fn share(&self, bucket: String, key: String, expires: Option<String>);
}

impl<P> CTCLIObject for CTClient<P>
    where P: AwsCredentialsProvider,
{
    // TODO: list 出带前缀“prefix/”的所有对象, 读取这些对象, 删除其他对象 (Pipeline)
    // TODO: cto ls test -p prefix | cto get | cto del --other
    fn list(&self, _quiet: bool, bucket: String, prefix: Option<String>) {
        debug!("List Objects");
        match self.list_objects(&ListObjectsRequest {
            bucket,
            prefix,
            ..Default::default()
        }) {
            Ok(h) => printstd!(h.contents, key, last_modified, size),
            Err(e) => debug!("{:#?}", e),
        }
    }

    fn new(&self, bucket: String, key: String, body: String) {
        debug!("Create Object");
        match self.put_object(&PutObjectRequest {
            bucket: bucket.clone(),
            key: key.clone(),
            body: Some(body.as_bytes()),
            ..Default::default()
        }, None) {
            Ok(out) => {
                debug!("{:#?}", out);
                if body.len() > 40 {
                    println!("Create {} to {}", key, bucket);
                } else {
                    println!("Create {} with \'{}\' to {}", key, body, bucket);
                }
            }
            Err(err) => print_aws_err!(err),
        }
    }

    fn get(&self, bucket: String, key: String, path: &Path) {
        debug!("Downland Object");
        match self.get_object(&GetObjectRequest {
            bucket: bucket.clone(),
            key: key.clone(),
            ..Default::default()
        }, None) {
            Ok(out) => {
                let mut file = match File::create(path) {
                    Ok(file) => file,
                    Err(e) => {
                        debug!("{:#?}", e);
                        println!("Error reading file {}", e);
                        return;
                    }
                };

                match file.write_all(out.get_body()) {
                    Ok(out) => {
                        debug!("{:#?}", out);
                        println!("Download {} from {}", key, bucket);
                    }
                    Err(err) => println!("{}", err),
                }
            }
            Err(err) => print_aws_err!(err),
        }
    }

    /// 1. 通过 Put方式上传本地文件（文件小于 100M）
    /// 2. 分段上传一个本地文件
    // TODO: 设置 Object上传时的冗余模式，使上传时可实现自定义分片模式
    /// Upload an object to your BUCKET - You can easily upload a file to
    /// S3, or upload directly an InputStream if you know the length of
    /// the data in the stream. You can also specify your own metadata
    /// when uploading to S3, which allows you set a variety of options
    /// like content-type and content-encoding, plus additional metadata
    /// specific to your applications.
    fn put(&self, bucket: String, key: String, path: &Path) {
        debug!("Put Object");
        if path.is_dir() {
            if let Ok(entries) = path.read_dir() {
                for entry in entries {
                    if let Ok(entry) = entry {
                        self.put(bucket.clone(),
                                 entry.path().into_os_string().into_string().unwrap(),
                                 entry.path().as_ref());
                    }
                }
            }
            return;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                debug!("{:#?}", error);
                println!("{}", error);
                return;
            }
        };

        let metadata = file.metadata().unwrap();

        let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);

        match file.take(metadata.len()).read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }

        let mut request = PutObjectRequest::default();
        request.bucket = bucket.clone();
        request.key = key.clone();
        request.body = Some(&buffer);

        // Compute hash - Hash is slow
        let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
        request.content_md5 = Some(hash);

        match self.put_object(&request, None) {
            Ok(output) => {
                debug!("{:#?}", output);
                println!("Put {} to {}", key, bucket);
            }
            Err(err) => print_aws_err!(err),
        }
    }

    // TODO: 设置专属签名，实现自定义加密，使用户拥有独特的签名方式
    fn put_securely(&self, bucket: String, key: String, path: &Path) {
        /*match self.put_object_securely(&PutObjectRequest{
            ..Default::default()
        }, None) {
            Ok(output) => {
                debug!("{:#?}", output);
                println!("Put {} to {}", key, bucket);
            },
            Err(err) => print_aws_err!(err),
        }*/
    }

    // TODO: 通过 Post方式上传本地文件（文件小于 100M）

    // TODO: 实现多线程上传多个对象

    // recursive: bool
    fn delete(&self, bucket: String, key: String) {
        debug!("Remove Object");
        match self.delete_object(&DeleteObjectRequest {
            bucket: bucket.clone(),
            key: key.clone(),
            ..Default::default()
        }, None) {
            Ok(output) => {
                debug!("{:#?}", output);
                println!("Delete {} from {}", key, bucket);
            }
            Err(err) => print_aws_err!(err),
        }
    }

    /// 分享已上传的 Object（Share）
    // TODO: URL有效期为一周
    fn share(&self, bucket: String, key: String, expires: Option<String>) {
        debug!("Share Object");
        match self.presigned_object(&PresignedObjectRequest {
            bucket,
            key,
            expires,
        }) {
            Ok(h) => print!("{}", h),
            Err(err) => print_aws_err!(err),
        }
    }
}