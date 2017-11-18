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

use colored::*;

use rustc_serialize::base64::{ToBase64, STANDARD};

use ct_sdk::ct::sdk::CTClient;
use ct_sdk::ct::object::*;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

use clap::ArgMatches;

/// High-level OOS object operations commands
/// Like http://docs.aws.amazon.com/cli/latest/reference/s3/index.html

/// Additional object operations commands for CTClient.
// TODO: list 出带前缀“prefix/”的所有对象, 读取这些对象, 删除其他对象 (Pipeline)
// TODO: cto ls test -p prefix | cto get | cto del --other
pub fn list(bucket: &str, args: &ArgMatches) {
    debug!("List Objects");
    //let version = args.value_of("version").unwrap();
    let prefix = args.value_of("prefix");
    //let max_keys = args.value_of("max_keys").unwrap();
    //let delimiter = args.value_of("delimiter").unwrap();
    //let marker = args.value_of("marker").unwrap();
    //let encoding_type = args.value_of("encoding_type").unwrap();
    let quiet = args.is_present("quiet");

    match CTClient::default_securely_client().list_objects(&ListObjectsRequest {
        bucket: bucket.to_string(),
        //version: version.to_string(),
        prefix: match prefix {
            Some(s) => Some(s.to_string()),
            None => None,
        },
        //max_keys,
        //delimiter,
        //marker: None,
        //encoding_type,
        ..Default::default()
    }) {
        Ok(out) => match quiet {
            false => printstd!(out.contents, key, last_modified, size),
            true => printlist!(out.contents, key),
        },
        Err(e) => debug!("{:#?}", e),
    }
}

pub fn new(bucket: &str, args: &ArgMatches) {
    debug!("Create Object");
    let key = args.value_of("key").unwrap();
    let body = args.value_of("body").unwrap();

    match CTClient::default_securely_client().put_object(
        &PutObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            body: Some(body.as_bytes()),
            ..Default::default()
        },
        None,
    ) {
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

pub fn get(bucket: &str, args: &ArgMatches) {
    debug!("Downland Object");
    let key = args.value_of("key").unwrap();

    match CTClient::default_securely_client().get_object(
        &GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        },
        None,
    ) {
        Ok(out) => println!(
            "+--[ START ]----+\n{}\n+--[  END  ]----+",
            String::from_utf8_lossy(out.get_body())
        ),
        Err(err) => print_aws_err!(err),
    }
}


pub fn get_securely(bucket: &str, args: &ArgMatches) {
    debug!("Downland Object");
    let key = args.value_of("key").unwrap();

    match CTClient::default_securely_client().get_object_securely(
        &GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        },
        None,
    ) {
        Ok(out) => println!(
            "+--[ START ]----+\n{}\n+--[  END  ]----+",
            String::from_utf8_lossy(out.get_body())
        ),
        Err(err) => print_aws_err!(err),
    }
}

/// 下载已上传的 Object到本地（Download）
pub fn download(bucket: &str, args: &ArgMatches) {
    debug!("Downland Object");
    let keys = args.values_of("keys").unwrap().collect::<Vec<_>>();
    let output = args.value_of("dir").unwrap_or("./");

    let ct = CTClient::default_securely_client();

    keys.iter().for_each(|key| {
        print!("{}\t", key);
        match ct.get_object(
            &GetObjectRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                ..Default::default()
            },
            None,
        ) {
            Ok(out) => {
                let mut file = match File::create(Path::new(format!("{}{}", output, key).as_str()))
                {
                    Ok(file) => file,
                    Err(err) => {
                        debug!("{:#?}", err);
                        println!("{}", " ✗ ".red().bold());
                        return;
                    }
                };

                match file.write_all(out.get_body()) {
                    Ok(output) => {
                        debug!("{:#?}", output);
                        println!("{}", " ✓ ".green().bold());
                    }
                    Err(err) => {
                        debug!("{:#?}", err);
                        println!("{}", " ✗ ".red().bold());
                    }
                }
            }
            Err(err) => print_aws_err!(err),
        }
    });
}

pub fn download_securely(bucket: &str, args: &ArgMatches) {
    debug!("Downland Object");
    let key = args.value_of("key").unwrap();
    let path = args.value_of("path").unwrap();

    match CTClient::default_securely_client().get_object(
        &GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        },
        None,
    ) {
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

pub fn put_args(bucket: &str, args: &ArgMatches) {
    let keys = args.values_of("keys").unwrap().collect::<Vec<_>>();
    // let path = Path::new(args.value_of("path").unwrap());
    let reverse = args.is_present("reverse");
    let prefix = match args.value_of("prefix") {
        Some(s) => s,
        None => "",
    };
    let storage_class = match args.value_of("storage_class") {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };

    keys.iter().for_each(|key| {
        print!("{}\t", key);
        match args.is_present("multithread") {
            true => put_multithread(
                bucket.to_string(),
                Path::new(key),
                prefix.to_string(),
                storage_class.clone(),
                reverse,
            ),
            false => put(
                bucket.to_string(),
                Path::new(key),
                prefix.to_string(),
                storage_class.clone(),
                reverse,
            ),
        }
    });
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
pub fn put(bucket: String, path: &Path, prefix: String, storage_class: String, reverse: bool) {
    debug!("Put Object");
    if path.is_dir() {
        if let Ok(entries) = path.read_dir() {
            for entry in entries {
                if let Ok(entry) = entry {
                    put(
                        bucket.clone(),
                        entry.path().as_ref(),
                        prefix.clone(),
                        storage_class.clone(),
                        reverse,
                    );
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
    request.bucket = bucket.to_string();
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = path.file_name().unwrap().to_str().unwrap().to_string();
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
    request.content_md5 = Some(hash);

    match CTClient::default_securely_client().put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            print_aws_err!(err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}

pub fn put_multithread(
    bucket: String,
    path: &Path,
    prefix: String,
    storage_class: String,
    reverse: bool,
) {
    debug!("Put Object Multithread");
    if path.is_dir() {
        if let Ok(entries) = path.read_dir() {
            let mut threads = vec![];
            for entry in entries {
                if let Ok(entry) = entry {
                    use std::thread;
                    {
                        let pathbuf = entry.path();
                        let bucket = bucket.clone();
                        let prefix = prefix.clone();
                        let storage_class = storage_class.clone();

                        threads.push(thread::spawn(move || -> i32 {
                            put_multithread(
                                bucket,
                                entry.path().as_ref(),
                                prefix,
                                storage_class.clone(),
                                reverse,
                            );
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
    request.bucket = bucket.to_string();
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = path.file_name().unwrap().to_str().unwrap().to_string();
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5::compute(request.body.unwrap()).to_base64(STANDARD);
    request.content_md5 = Some(hash);
    match CTClient::default_securely_client().put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            print_aws_err!(err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}

// TODO: 设置专属签名，实现自定义加密，使用户拥有独特的签名方式
pub fn put_securely(bucket: &String, key: &String, path: &Path, storage_class: String) {
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
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = key.clone();
    request.body = Some(&buffer);

    match CTClient::default_securely_client().put_object_securely(request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            print_aws_err!(err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}

// TODO: 通过 Post方式上传本地文件（文件小于 100M）

// TODO: 实现多线程上传多个对象

/// 删除已上传的 Object（Delete）
/// Deletes an object(rm)
pub fn delete(bucket: &str, args: &ArgMatches) {
    debug!("Remove Object");
    let count = args.occurrences_of("keys");
    let keys = args.values_of("keys").unwrap().collect::<Vec<_>>();

    let ct = CTClient::default_securely_client();

    let mut success = 0;
    let mut error = 0;

    keys.iter().for_each(|key| {
        print!("{}\t", key);
        match ct.delete_object(
            &DeleteObjectRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                ..Default::default()
            },
            None,
        ) {
            Ok(output) => {
                debug!("{:#?}", output);
                println!("{}", " ✓ ".green().bold());
                success += 1;
            }
            Err(err) => {
                print_aws_err!(err);
                println!("{}", " ✗ ".red().bold());
                error += 1;
            }
        }
    });


    println!(
        "\nAll: {}, Success: {}, Error: {}",
        count,
        format!("{}", success).green(),
        format!("{}", error).red()
    )
}

/// 分享已上传的 Object（Share）
/// presign
// TODO: URL有效期为一周
pub fn share(bucket: &str, args: &ArgMatches) {
    debug!("Share Object");
    let key = args.value_of("key").unwrap();
    let expires = match args.value_of("expires") {
        Some(s) => Some(s.to_string()),
        None => None,
    };

    match CTClient::default_securely_client().presigned_object(&PresignedObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        expires,
    }) {
        Ok(h) => print!("{}", h),
        Err(err) => print_aws_err!(err),
    }
}
