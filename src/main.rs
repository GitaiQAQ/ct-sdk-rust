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

//! Command Line Interface for CTYun OOS Services
//!
//! # Getting Started
//! ## Credentials configure
//!
//! ```shell
//! ct conf
//! ```
//! Other ways:
//! * Environment variables
//! * Shared credentials file
//! * Config file
//! * IAM Role
//! ## Object Operations

#[macro_use]
extern crate clap;
extern crate colored;
extern crate ct_sdk;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate md5;
extern crate prettytable;
extern crate rustc_serialize;

use std::env;
use clap::ArgMatches;

mod cli;

// http://oos-bj2.ctyunapi.cn
#[allow(unused_variables)]
fn main() {
    let matches: ArgMatches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Gitai<i@gitai.me>")
        (about: "Does awesome things")
        (@subcommand account =>
            (about: "Write AK/SK to disk(~/.aws/credentials)")
            (@arg aws_access_key_id: +required -a --ak +takes_value "AK/AWS Access Key Id")
            (@arg aws_secret_access_key: +required -s --sk +takes_value "SK/AWS Secret Access Key")
        )
        (@subcommand bucket =>
            (about: "管理仓库")
            (@subcommand ls =>
                (about: "列出全部储存仓库")
                (@arg quiet: -q --quiet "精简模式，只显示仓库唯一 ID")
            )
            (@subcommand new =>
                (about: "新建空储存仓库")
                (@arg bucket_name: +required +takes_value)
            )
            (@subcommand rm =>
                (about: "删除空储存仓库")
                (@arg buckets: +required +multiple +takes_value "仓库名称")
                //(@arg force: -f --force "强制删除非空仓库")
            )
            (@subcommand set =>
                (about: "更改仓库属性（私有、公有、只读）")
                (@arg bucket_name: +required +takes_value)
                (@arg read: -r --read +takes_value "公开读取")
                (@arg write: -w --write +takes_value "公开写入")
            )
        )
        (@subcommand object =>
            (about: "管理对象")
            (@arg bucket_name: +required +takes_value "储存仓库")
            (@subcommand ls =>
                (about: "列出全部储存对象")
                (@arg prefix: +takes_value "过滤前缀")
                (@arg quiet: -q --quiet "精简模式，只显示对象唯一 ID")
            )
            /*(@subcommand new =>
                (about: "从内容新建储存对象")
                (@arg bucket_name: +required +takes_value "储存仓库")
                (@arg key: +required +takes_value "对象唯一 ID")
                (@arg body: +required +multiple +takes_value "内容")
            )*/
            (@subcommand up =>
                (about: "上传对象")
                (@arg keys: +required +multiple +takes_value "对象 ID 列表")
                (@arg multithread: -m --multithread "多线程上传")
                (@arg reverse: -r --reverse "递归子目录")
                (@arg prefix: -p --prefix "前缀")
            )
            /*(@subcommand get =>
                (about: "读取对象")
                (@arg bucket_name: +required +takes_value "储存仓库")
                (@arg key: +required +takes_value "对象唯一 ID")
            )*/
            (@subcommand down =>
                (about: "下载对象")
                (@arg keys: +required +multiple +takes_value "对象 ID 列表")
                (@arg dir: -o --output +takes_value "储存文件夹")
            )
            (@subcommand rm =>
                (about: "删除对象")
                (@arg keys: +required +multiple +takes_value "对象 ID 列表")
            )
            (@subcommand share =>
                (about: "分享对象")
                (@arg key: +required +takes_value "对象唯一 ID")
                (@arg expires: -e --expires +takes_value "时间（1500s）")
            )
        )
        (@subcommand iam =>
            (@subcommand ls =>
                (about: "列出全部 AK/SK")
                (@arg quiet: -q --quiet "精简模式，只显示 AK")
            )
            (@subcommand new =>
                (about: "新建 AK/SK")
            )
            (@subcommand rm =>
                (about: "删除 AK/SK")
                (@arg access_key_id: --ak +required +takes_value)
            )
            (@subcommand set =>
                (about: "更改 AK/SK 属性")
                (@arg access_key_id: --ak +required +takes_value)
                (@arg status: -s --status "生效/禁用")
                (@arg is_primary: -p --isprimary "主秘钥/普通秘钥")
            )
        )
        (@arg aws_access_key_id: -a --ak +takes_value "AK/AWS Access Key Id")
        (@arg aws_secret_access_key: -s --sk +takes_value "SK/AWS Secret Access Key")
        (@arg verbosity: -v +multiple "设置调试等级")
    ).get_matches();

    match matches.occurrences_of("v") {
        1 => env::set_var("RUST_LOG", "error"),
        2 => env::set_var("RUST_LOG", "warm"),
        3 => env::set_var("RUST_LOG", "debug"),
        _ => {}
    }
    env_logger::init().unwrap();

    debug!("{:#?}", matches);

    match matches.subcommand() {
        ("bucket", Some(matches)) => {
            use cli::bucket::*;
            match matches.subcommand() {
                ("new", Some(args)) => create(args),
                ("rm", Some(args)) => delete(args),
                ("ls", Some(args)) => list(args),
                _ => {}
            }
        }
        ("object", Some(matches)) => {
            use cli::object::*;
            let bucket = matches.value_of("bucket_name").unwrap();
            match matches.subcommand() {
                ("ls", Some(args)) => list(bucket, args),
                ("up", Some(args)) => put_args(bucket, args),
                // ("get", Some(args)) => get(bucket, args),
                ("down", Some(args)) => download(bucket, args),
                ("rm", Some(args)) => delete(bucket, args),
                ("share", Some(args)) => share(bucket, args),
                _ => {}
            }
        }
        ("account", Some(matches)) => {
            use cli::iam::*;
            match matches.subcommand() {
                ("new", Some(args)) => create(args),
                ("rm", Some(args)) => delete(args),
                ("ls", Some(args)) => list(args),
                ("set", Some(args)) => update(args),
                _ => {}
            }
        }
        _ => {}
    }

    {
        // use cli::object::CTCLIObject;
        // s3.share(String::from("gitai"), String::from("date.txt"), None);

        /*use std::thread;
        use std::sync::{Arc, Mutex};
        thread::spawn(|| {
            use cli::bucket::*;
            let s3 = CTClient::default_ctyun_securely_client();
            create(&s3, String::from("testqaq"));
            list(&s3);
        });*/

    }

    {
        /*use cli::bucket::*;
        let s3 = CTClient::default_ctyun_securely_client();
        create(&s3, String::from("testqaq"));
        list(&s3);*/
    }

    {
        /*use std::thread;
        use std::sync::{Arc, Mutex};
        thread::spawn(|| {
            use cli::object::put;
            let s3 = CTClient::default_ctyun_securely_client();
            put(&s3,
                String::from("testqaq"),
                String::from(""),
                Path::new("/home/gitai/project/stanford-cpp-library/autograder"),
                None);
        });*/
        /*use std::path::Path;
        use cli::object::put_thread;
        put_thread(String::from("testqaq"),
            String::from(""),
            Path::new("/home/gitai/project/stanford-cpp-library/autograder"),
            None);*/
    }

    {
        /*s3.put_securely(
            String::from("gitai.test"),
            String::from("date_securely.txt"),
            Path::new("/home/gitai/date.txt"),
            None);*/
        // list(s3, false, String::from("gitai.test"), Some("/data".to_string()));
        /*s3.get(
            String::from("gitai.test"),
            String::from("date.txt"));*/
        /*s3.get(
            String::from("gitai.test"),
            String::from("date_securely.txt"));*/
        /*s3.get_securely(
            String::from("gitai.test"),
            String::from("date_securely.txt"));*/
    }

    // s3.create(String::from("gitai.test"));
    //s3.acl(String::from("gitai.test"), CannedAcl::PublicReadWrite);
    // s3.delete(String::from("gitai.test"));
    {
        // use cli::iam::CTCLIAM;
        // s3.list();
        // s3.create();
        // s3.delete(String::from("d72e05685a5e7d7b0eb7"));
        // s3.update(String::from("2aa302a2e7182784409e"));
    }
    debug!("END");
}
