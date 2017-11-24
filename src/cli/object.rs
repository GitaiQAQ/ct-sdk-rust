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

use colored::*;

use rustc_serialize::base64::{ToBase64, STANDARD};

use ct_sdk::ct::sdk::CTClient;
use ct_sdk::ct::sdk::md5;
use ct_sdk::ct::object::*;
use ct_sdk::ct::errors::s3::S3Error;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

use clap::ArgMatches;

/// 列出对象
///
/// ```shell
/// ct-cli object <bucket> ls [-p] [-q]
/// ```
///
/// ### 选项
/// * `-p`, `--prefix`: 过滤前缀
/// * `-q`, `--quiet`: 只显示名字
///
/// ### 截图
/// ![object--prefix-delete.png](https://i.loli.net/2017/11/21/5a13b0bfa6f07.png)
pub fn list(bucket: &str, args: &ArgMatches) {
    debug!("List Objects");
    //let version = args.value_of("version").unwrap();
    let prefix = args.value_of("prefix");
    //let max_keys = args.value_of("max_keys").unwrap();
    //let delimiter = args.value_of("delimiter").unwrap();
    //let marker = args.value_of("marker").unwrap();
    //let encoding_type = args.value_of("encoding_type").unwrap();
    let quiet = args.is_present("quiet");

    match CTClient::default_client().list_objects(&ListObjectsRequest {
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

/// 新建对象（暂未提供接口）
///
/// ```shell
/// ct-cli object <bucket> new <key> <body>
/// ```
pub fn new(bucket: &str, args: &ArgMatches) {
    debug!("Create Object");
    let key = args.value_of("key").unwrap();
    let body = args.value_of("body").unwrap();

    match CTClient::default_client().put_object(
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
                info!("Create {} to {}", key, bucket);
            } else {
                info!("Create {} with \'{}\' to {}", key, body, bucket);
            }
        }
        Err(err) => print_aws_err!(err),
    }
}

/// 读取对象，从 `clap` 中解析参数
///
/// ```shell
/// ct-cli object <bucket> get <key> [-e] [-k]
/// ```
///
/// ### 选项
/// * `-e`, `--encryptmethod` 加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）
/// * `-k`, `--password` 密钥
pub fn get_args(bucket: &str, args: &ArgMatches) {
    debug!("Get Object");
    let key = args.value_of("key").unwrap();

    match match (args.value_of("PASSWORD"), args.value_of("ENCRYPT_METHOD")) {
        (Some(password), Some(method)) => get_securely(
            bucket.to_string(),
            key.to_string(),
            method.to_string(),
            password.to_string(),
        ),
        (Some(password), None) => get_securely(
            bucket.to_string(),
            key.to_string(),
            "".to_string(),
            password.to_string(),
        ),
        _ => get(bucket.to_string(), key.to_string()),
    } {
        Ok(out) => info!(
            "+--[ START ]----+\n{}\n+--[  END  ]----+",
            String::from_utf8_lossy(out.get_body())
        ),
        Err(err) => print_aws_err!(err),
    }
}

/// 下载对象，从 `clap` 中解析参数
///
/// ```shell
/// ct-cli object <bucket> down [-e] [-k] <keys>... -o <output>
/// ```
/// ### 选项
/// * `-e`, `--encryptmethod` 加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）
/// * `-k`, `--password` 密钥
/// * `-o`, `--output` 储存文件夹
///
/// ### 截图
/// ![object-down.png](https://i.loli.net/2017/11/21/5a13b0c349efe.png)
pub fn down_args(bucket: &str, args: &ArgMatches) {
    debug!("Download Object");
    let keys = args.values_of("keys").unwrap().collect::<Vec<_>>();
    let output = args.value_of("dir").unwrap_or("./");

    let ct = CTClient::default_client();

    keys.iter().for_each(|key| {
        match match (
            (args.value_of("PASSWORD"), args.value_of("ENCRYPT_METHOD")),
            args.is_present("multithread"),
        ) {
            ((Some(password), Some(method)), true) => unimplemented!(),
            ((Some(password), Some(method)), false) => get_securely(
                bucket.to_string(),
                key.to_string(),
                method.to_string(),
                password.to_string(),
            ),
            ((None, None), true) => unimplemented!(),
            _ => get(bucket.to_string(), key.to_string()),
        } {
            Ok(out) => {
                let mut file = match File::create(Path::new(format!("{}{}", output, key).as_str()))
                {
                    Ok(file) => file,
                    Err(err) => {
                        debug!("{:#?}", err);
                        info!("{}\t{}", " ✗ ".red().bold(), key);
                        return;
                    }
                };

                match file.write_all(out.get_body()) {
                    Ok(output) => {
                        debug!("{:#?}", output);
                        info!("{}\t{}", " ✓ ".green().bold(), key);
                    }
                    Err(err) => {
                        debug!("{:#?}", err);
                        info!("{}\t{}", " ✗ ".red().bold(), key);
                    }
                }
            }
            Err(err) => print_aws_err!(err),
        }
    });
}

/// 读取对象
///
/// ```shell
/// ct-cli object <bucket> get <key>
/// ```
pub fn get(bucket: String, key: String) -> Result<GetObjectOutput, S3Error> {
    debug!("Get Object");

    CTClient::default_client().get_object(
        &GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        },
        None,
    )
}

/// 读取加密对象
///
/// ```shell
/// ct-cli object <bucket> get <key> [-e] [-k]
/// ```
/// ### 选项
/// * `-e`, `--encryptmethod` 加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）
/// * `-k`, `--password` 密钥
pub fn get_securely(
    bucket: String,
    key: String,
    method: String,
    password: String,
) -> Result<GetObjectOutput, S3Error> {
    debug!("Downland Object");

    let method = match method.parse() {
        Ok(m) => m,
        Err(err) => {
            panic!("Does not support {:?} method: {:?}", method, err);
        }
    };

    CTClient::default_securely_client(password, method).get_object_securely(
        &GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        },
        None,
    )
}

/// 上传对象，从 `clap` 中解析参数
///
/// ```shell
/// ct-cli object <bucket> up <keys> [-e] [-k] -p [prefix]
/// ```
/// ### 选项
/// * `-e`, `--encryptmethod` 加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）
/// * `-k`, `--password` 密钥
/// * `-m`, `--multithread` 多线程上传
/// * `-p`, `--prefix` 上传到指定前缀
/// * `-s`, `--storageclass` 储存模式
///
///
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
        if args.is_present("multipart") {
            put_multipart(bucket.to_string(), Path::new(key), 1024);
        } else {
            match (
                (args.value_of("PASSWORD"), args.value_of("ENCRYPT_METHOD")),
                args.is_present("multithread"),
            ) {
                ((Some(password), Some(method)), true) => unimplemented!(),
                ((Some(password), None), true) => unimplemented!(),
                ((Some(password), Some(method)), false) => put_securely(
                    bucket.to_string(),
                    Path::new(key),
                    prefix.to_string(),
                    method.to_string(),
                    password.to_string(),
                    storage_class.clone(),
                    reverse,
                ),
                ((Some(password), None), false) => put_securely(
                    bucket.to_string(),
                    Path::new(key),
                    prefix.to_string(),
                    "".to_string(),
                    password.to_string(),
                    storage_class.clone(),
                    reverse,
                ),
                ((None, None), true) => put_multithread(
                    bucket.to_string(),
                    Path::new(key),
                    prefix.to_string(),
                    storage_class.clone(),
                    reverse,
                ),
                _ => put(
                    bucket.to_string(),
                    Path::new(key),
                    prefix.to_string(),
                    storage_class.clone(),
                    reverse,
                ),
            }
        }
    });
}

/// 上传对象
///
/// ```shell
/// ct-cli object <bucket> up <keys> -p [prefix]
/// ```
/// ### 选项
/// * `-p`, `--prefix` 上传到指定前缀
/// * `-s`, `--storageclass` 储存模式
///
/// ### 截图
/// ![object-up.png](https://i.loli.net/2017/11/21/5a13b0c28da64.png)
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
        Err(err) => {
            debug!("{:#?}", err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
            return;
        }
    };

    let metadata = file.metadata().unwrap();

    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);

    match file.take(metadata.len()).read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(err) => info!("{}", err),
    }

    let mut request = PutObjectRequest::default();
    request.bucket = bucket.to_string();
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = format!("{}{}", prefix, path.file_name().unwrap().to_str().unwrap());
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5(request.body.unwrap());

    request.content_md5 = Some(hash);

    match CTClient::default_client().put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            info!("{}\t{:?}", " ✓ ".green().bold(), path);
        }
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
        }
    }
}


/// 分片上传
///
/// ```shell
/// ct-cli object <bucket> up <keys> --mp
/// ```
/// ### 选项
/// * `--mp` 分片上传
///
/// ### 截图
/// ![object-up-multipart.png](https://i.loli.net/2017/11/22/5a157da71db3a.png)
/// 
pub fn put_multipart(bucket: String,
                     path: &Path,
                     part_size: u64) {
    let ct = CTClient::default_client();
    let correct_key = path.file_name().unwrap().to_str().unwrap().to_string();

    // Create multipart
    let create_multipart_upload: MultipartUploadCreateOutput;
    let mut request = MultipartUploadCreateRequest::default();
    request.bucket = bucket.to_string();
    request.key = correct_key.clone();

    match ct.multipart_upload_create(&request) {
        Ok(output) => {
            create_multipart_upload = output;
        },
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
            return;
        },
    }

    let upload_id: &str = &create_multipart_upload.upload_id;
    let mut parts_list: Vec<String> = Vec::new();

    // NB: To begin with the multipart will be a sequential upload in this thread! Aftwards, it will
    // be split out to a multiple of threads...

    let file = File::open(path).unwrap();
    let metadata = file.metadata().unwrap();

    let mut part_buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);

    match file.take(metadata.len()).read_to_end(&mut part_buffer) {
        Ok(_) => {},
        Err(err) => {
            debug!("{:#?}", err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
            return;
        },
    }

    let mut request = MultipartUploadPartRequest::default();
    request.bucket = bucket.to_string();
    request.upload_id = upload_id.to_string();
    request.key = correct_key.clone();

    request.body = Some(&part_buffer);
    request.part_number = 1;

    // Compute hash - Hash is slow
    let hash = md5(request.body.unwrap());

    request.content_md5 = Some(hash);

    match ct.multipart_upload_part(&request) {
        Ok(output) => {
            // Collecting the partid in a list.
            let new_output = output.clone();
            parts_list.push(output);

            debug!("{:#?}", new_output);
            info!("├─ {}\t{}", " ✓ ".green().bold(), new_output);
        },
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
            return;
        },
    }
    // End of upload

    // Complete multipart
    let item_list: Vec<u8>;

    let mut request = MultipartUploadCompleteRequest::default();
    request.bucket = bucket.to_string();
    request.upload_id = upload_id.to_string();
    request.key = correct_key;

    // parts_list gets converted to XML and sets the item_list.
    match multipart_upload_finish_xml(&parts_list) {
        Ok(parts_in_xml) => item_list = parts_in_xml,
        Err(err) => {
            print_aws_err!(err);
            info!("{}", " ✗ ".red().bold());
            return;
        },
    }

    request.multipart_upload = Some(&item_list);

    match ct.multipart_upload_complete(&request) {
        Ok(out) => {
            debug!("{:#?}", out);
            info!("└─ {}\t{:?}", " ✓ ".green().bold(), out.key);
        },
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
        },
    }
}

/// 多线程上传
///
/// ```shell
/// ct-cli object <bucket> up <keys> -m -p [prefix] -s [storageclass]
/// ```
/// ### 选项
/// * `-p`, `--prefix` 上传到指定前缀
/// * `-s`, `--storageclass` 储存模式
///
/// ### 截图
/// ![object-up-mutilthread.png](https://i.loli.net/2017/11/21/5a13b99533148.png)
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
        Err(err) => {
            debug!("{:#?}", err);
            info!("{:?} {}", path, " ✗ ".red().bold());
            return;
        }
    };

    let metadata = file.metadata().unwrap();

    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);

    match file.take(metadata.len()).read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(err) => info!("{}", err),
    }

    let mut request = PutObjectRequest::default();
    request.bucket = bucket.to_string();
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = format!("{}{}", prefix, path.file_name().unwrap().to_str().unwrap());
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5(request.body.unwrap());

    request.content_md5 = Some(hash);
    match CTClient::default_client().put_object(&request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            info!("{}\t{:?}", " ✓ ".green().bold(), path);
        }
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
        }
    }
}

/// 上传自定义加密对象，使用户拥有独特的签名方式
///
/// ```shell
/// ct-cli object <bucket> up <key> [-e] [-k]
/// ```
/// ### 选项
/// * `-e`, `--encryptmethod` 加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）
/// * `-k`, `--password` 密钥
///
/// ### 截图
/// ![object-up-securely.png](https://i.loli.net/2017/11/21/5a13b0c42001c.png)
pub fn put_securely(
    bucket: String,
    path: &Path,
    prefix: String,
    method: String,
    password: String,
    storage_class: String,
    reverse: bool,
) {
    debug!("Put Securely Object");
    if path.is_dir() {
        if let Ok(entries) = path.read_dir() {
            for entry in entries {
                if let Ok(entry) = entry {
                    put_securely(
                        bucket.clone(),
                        entry.path().as_ref(),
                        prefix.clone(),
                        method.to_string(),
                        password.clone(),
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
        Err(err) => {
            debug!("{:#?}", err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
            return;
        }
    };

    let metadata = file.metadata().unwrap();

    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);

    match file.take(metadata.len()).read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(err) => info!("{}", err),
    }

    let method = match method.parse() {
        Ok(m) => m,
        Err(err) => {
            panic!("Does not support {:?} method: {:?}", method, err);
        }
    };

    let mut request = PutObjectRequest::default();
    request.bucket = bucket.to_string();
    request.storage_class = match storage_class.is_empty() {
        true => None,
        false => Some(storage_class),
    };

    request.key = format!("{}{}", prefix, path.file_name().unwrap().to_str().unwrap());
    request.body = Some(&buffer);

    // Compute hash - Hash is slow
    let hash = md5(request.body.unwrap());

    match CTClient::default_securely_client(password, method).put_object_securely(request, None) {
        Ok(output) => {
            debug!("{:#?}", output);
            info!("{}\t{:?}", " ✓ ".green().bold(), path);
        }
        Err(err) => {
            print_aws_err!(err);
            info!("{}\t{:?}", " ✗ ".red().bold(), path);
        }
    }
}

/// Post 上传文件
///
/// ```shell
/// ct-cli object <bucket> post <key>
/// ```
///
/// ### 选项
/// * `-e`, `--expires` 有效期
///
/// ### 截图
/// ![object-post.png](https://i.loli.net/2017/11/24/5a17baf75e9d8.png)
/// ![object-post-browser.png](https://i.loli.net/2017/11/24/5a17baf86770b.png)
///
pub fn post(bucket: &str, args: &ArgMatches) {
    debug!("Post Object");
    let key = args.value_of("key").unwrap();
    let expires = match args.value_of("expires") {
        Some(s) => Some(s.to_string()),
        None => None,
    };

    CTClient::default_client().post_object(&PostObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        expires,
    });
}


/// 删除对象（Delete）
///
/// ```shell
/// ct-cli object <bucket> rm <keys>
/// ```
///
/// ### 截图
/// ![object-rm.png](https://i.loli.net/2017/11/21/5a13b0c232cb1.png)
pub fn delete(bucket: &str, args: &ArgMatches) {
    debug!("Remove Object");
    let count = args.occurrences_of("keys");
    let keys = args.values_of("keys").unwrap().collect::<Vec<_>>();

    let ct = CTClient::default_client();

    let mut success = 0;
    let mut error = 0;

    keys.iter().for_each(|key| {
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
                info!("{}\t{}", " ✓ ".green().bold(), key);
                success += 1;
            }
            Err(err) => {
                print_aws_err!(err);
                info!("{}\t{}", " ✗ ".red().bold(), key);
                error += 1;
            }
        }
    });


    info!(
        "\nAll: {}, Success: {}, Error: {}",
        count,
        format!("{}", success).green(),
        format!("{}", error).red()
    )
}

/// 分享对象（share, presign）
///
/// ```shell
/// ct-cli object <bucket> share <keys>
/// ```
/// ### 选项
/// * `-e`, `--expires` 有效期
///
/// ### 截图
/// ![object-share.png](https://i.loli.net/2017/11/21/5a13b0c1d5da4.png)
pub fn share(bucket: &str, args: &ArgMatches) {
    debug!("Share Object");
    let key = args.value_of("key").unwrap();
    let expires = match args.value_of("expires") {
        Some(s) => Some(s.to_string()),
        None => None,
    };

    match CTClient::default_client().presigned_object(&PresignedObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        expires,
    }) {
        Ok(h) => info!("{}", h),
        Err(err) => print_aws_err!(err),
    }
}
