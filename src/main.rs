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
extern crate prettytable;
extern crate rustc_serialize;

use std::env;
use env_logger::LogBuilder;
use log::{LogLevel, LogLevelFilter};
use clap::ArgMatches;

mod cli;

#[allow(unused_variables)]
fn main() {
    let matches: ArgMatches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Gitai<i@gitai.me>")
        (about: "Does awesome things")
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
                (@arg read: -r --read "公开读取")
                (@arg write: -w --write "公开写入")
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
                (@arg prefix: -p --prefix +takes_value "前缀")
                (@arg securely: -s --securely +takes_value "加密")
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
                (@arg securely: -s --securely +takes_value "解密")
            )
            (@subcommand get =>
                (about: "读取对象")
                (@arg key: +required +takes_value "对象 ID")
                (@arg securely: -s --securely +takes_value "解密")
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
        (@subcommand account =>
            (@subcommand ls =>
                (about: "列出 AK/SK")
                (@arg quiet: -q --quiet "精简模式，只显示 AK")
                (@arg all: -a --all "显示所有　AK （默认不显示主 Key）")
            )
            (@subcommand new =>
                (about: "新建 AK/SK")
            )
            (@subcommand rm =>
                (about: "删除 AK/SK")
                (@arg access_keys: +required +multiple +takes_value)
            )
            (@subcommand set =>
                (about: "更改 AK/SK 属性")
                (@arg access_key_id: +required +takes_value)
                (@arg status: -s --status "生效/禁用")
                (@arg is_primary: -p --isprimary "主秘钥/普通秘钥")
            )
        )
        (@arg aws_access_key_id: -a --ak +takes_value "Access Key Id")
        (@arg aws_secret_access_key: -s --sk +takes_value "Secret Access Key")
        (@arg verbosity: -v +multiple "设置调试等级")
    ).get_matches();

    LogBuilder::new()
        .parse(match matches.occurrences_of("verbosity") {
            1 => "ct_cli=Debug",
            2 => "ct_cli=Debug, aws_sdk_rust=Debug",
            3 => "Trace",
            _ => "Error",
        })
        .init();

    match (
        matches.value_of("aws_access_key_id"),
        matches.value_of("aws_secret_access_key"),
    ) {
        (Some(ak), Some(sk)) => {
            env::set_var("AWS_ACCESS_KEY_ID", ak);
            env::set_var("AWS_SECRET_ACCESS_KEY", sk);
            debug!("[ParametersProvider] AK: {} SK: {}", ak, sk);
        }
        _ => debug!("No Parameters Provider"),
    }

    debug!("{:#?}", matches);

    match matches.subcommand() {
        ("bucket", Some(matches)) => {
            use cli::bucket::*;
            match matches.subcommand() {
                ("new", Some(args)) => create(args),
                ("rm", Some(args)) => delete(args),
                ("ls", Some(args)) => list(args),
                ("set", Some(args)) => acl(args),
                _ => {}
            }
        }
        ("object", Some(matches)) => {
            use cli::object::*;
            let bucket = matches.value_of("bucket_name").unwrap();
            match matches.subcommand() {
                ("ls", Some(args)) => list(bucket, args),
                ("up", Some(args)) => put_args(bucket, args),
                ("get", Some(args)) => get_args(bucket, args),
                ("down", Some(args)) => down_args(bucket, args),
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
    debug!("END");
}
