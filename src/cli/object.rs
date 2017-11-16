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

use ct_sdk::ct::sdk::CTClient;
use ct_sdk::ct::object::*;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

/// High-level OOS object operations commands
/// Like http://docs.aws.amazon.com/cli/latest/reference/s3/index.html

/// Additional object operations commands for CTClient.
// TODO: list 出带前缀“prefix/”的所有对象, 读取这些对象, 删除其他对象 (Pipeline)
// TODO: cto ls test -p prefix | cto get | cto del --other
pub fn list(ct: &CTClient, _quiet: bool, bucket: String, prefix: Option<String>) {
    debug!("List Objects");
    match ct.list_objects(&ListObjectsRequest {
        bucket,
        prefix,
        ..Default::default()
    }) {
        Ok(h) => printstd!(h.contents, key, last_modified, size),
        Err(e) => debug!("{:#?}", e),
    }
}

pub fn new(ct: &CTClient, bucket: String, key: String, body: String) {
    debug!("Create Object");
    match ct.put_object(&PutObjectRequest {
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

pub fn get(ct: &CTClient, bucket: String, key: String) {
    debug!("Downland Object");
    match ct.get_object(&GetObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        ..Default::default()
    }, None) {
        Ok(out) => println!("+--[ START ]----+\n{}\n+--[  END  ]----+",
                            String::from_utf8_lossy(out.get_body())),
        Err(err) => print_aws_err!(err),
    }
}


pub fn get_securely(ct: &CTClient, bucket: String, key: String) {
    debug!("Downland Object");
    match ct.get_object_securely(&GetObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        ..Default::default()
    }, None) {
        Ok(out) => println!("+--[ START ]----+\n{}\n+--[  END  ]----+",
                            String::from_utf8_lossy(out.get_body())),
        Err(err) => print_aws_err!(err),
    }
}

/// 下载已上传的 Object到本地（Download）
pub fn download(ct: &CTClient, bucket: String, key: String, path: &Path) {
    debug!("Downland Object");
    match ct.get_object(&GetObjectRequest {
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

pub fn download_securely(ct: &CTClient, bucket: String, key: String, path: &Path) {
    debug!("Downland Object");
    match ct.get_object(&GetObjectRequest {
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


pub fn put_thread(bucket: String, key: String, path: &Path, storage_class: Option<String>) {
    debug!("Put Object");
    if path.is_dir() {
        if let Ok(entries) = path.read_dir() {
            let mut threads = vec![];
            for entry in entries {
                if let Ok(entry) = entry {
                    use std::thread;
                    use std::sync::{Mutex, Arc};

                    {
                        let pathbuf = entry.path();
                        let bucket = bucket.clone();
                        let storage_class = storage_class.clone();
                        //let path = pathbuf.as_path();
                        //let key = entry.path().into_os_string().into_string().unwrap();

                        threads.push(thread::spawn(move||-> i32 {
                            put_thread(bucket,
                       pathbuf.clone().into_os_string().into_string().unwrap(),
                       pathbuf.clone().as_path(),
               storage_class);
                        1
                        }));
                    }
                }
            }
            for thread in threads {
                thread.join().unwrap();
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
    request.storage_class = storage_class;

    request.key = key.clone();
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
    request.content_md5 = Some(hash);
    match CTClient::default_ctyun_securely_client()
        .put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("Put {} to {}", key, bucket);
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
pub fn put(ct: &CTClient, bucket: String, key: String, path: &Path, storage_class: Option<String>) {
    debug!("Put Object");
    if path.is_dir() {
        if let Ok(entries) = path.read_dir() {
            for entry in entries {
                if let Ok(entry) = entry {
                    put(ct,
                        bucket.clone(),
                        entry.path().into_os_string().into_string().unwrap(),
                        entry.path().as_ref(),
                        storage_class.clone());
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
    request.storage_class = storage_class;

    request.key = key.clone();
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
    request.content_md5 = Some(hash);

    match ct.put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("Put {} to {}", key, bucket);
        }
        Err(err) => print_aws_err!(err),
    }
}

// TODO: 设置专属签名，实现自定义加密，使用户拥有独特的签名方式
pub fn put_securely(ct: &CTClient, bucket: String, key: String, path: &Path, storage_class: Option<String>) {
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
    request.storage_class = storage_class;

    request.key = key.clone();
    request.body = Some(&buffer);

    match ct.put_object_securely(request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("Put {} to {}", key, bucket);
        },
        Err(err) => print_aws_err!(err),
    }
}

// TODO: 通过 Post方式上传本地文件（文件小于 100M）

// TODO: 实现多线程上传多个对象

/// 删除已上传的 Object（Delete）
/// Deletes an object(rm)
pub fn delete(ct: &CTClient, bucket: String, key: String) {
    debug!("Remove Object");
    match ct.delete_object(&DeleteObjectRequest {
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
/// presign
// TODO: URL有效期为一周
pub fn share(ct: &CTClient, bucket: String, key: String, expires: Option<String>) {
    debug!("Share Object");
    match ct.presigned_object(&PresignedObjectRequest {
        bucket,
        key,
        expires,
    }) {
        Ok(h) => print!("{}", h),
        Err(err) => print_aws_err!(err),
    }
}