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

//! 天翼云命令行接口
//!
//! # 快速开始
//! ## 初始化 Key
//!
//! ### 参数
//!
//! ```shell
//! $ ct-cli <some commands> -a <Access Key Id> -s <Secret Access Key>
//! ```
//!
//! ### 环境变量
//!
//! ```shell
//! $ set AWS_ACCESS_KEY_ID="Access Key Id"
//! $ set AWS_SECRET_ACCESS_KEY="Secret Access Key"
//! $ ct-cli <some commands>
//! ```
//!
//! ### 配置文件
//!
//! ```shell
//! $ cat ~/.aws/credentials
//! [default]
//! aws_access_key_id = ae2600e3194ec00fbcfb
//! aws_secret_access_key = c760152a28e608eb5b6d3bac02dd2780bff087cb
//! $ ct-cli <some commands>
//! ```
//!
//! ### 其他方式：
//!
//! * Shared credentials file
//!
//! * IAM Role
//!
//! ## 详细内容
//!
//! * [IAM](./cli/iam/)
//! * [Bucket](./cli/bucket/)
//! * [Object](./cli/object/)
//!
//! ## Check Point
//!
//! ### 初级 (2')
//!
//! - [X] [创建一组 AK/SK](./cli/iam/fn.create.html)
//! - [X] [更改 AK/SK 属性（主秘钥/普通秘钥）](./cli/iam/fn.update.html)
//! - [X] [删除已有的 AK/SK](./cli/iam/fn.delete.html)
//!
//! - [X] [创建一个 Bucket](./cli/bucket/fn.create.html)
//! - [X] [更改创建的 Bucket 属性（私有、公有、只读）](./cli/bucket/fn.acl.html)
//! - [X] [删除已创建的 Bucket](./cli/bucket/fn.delete.html)
//!
//! - [X] [通过 Put 方式上传本地文件（文件小于100M）](./cli/object/fn.put.html)
//! - [X] [下载已上传的 Object 到本地 （Download）](./cli/object/fn.down_args.html)
//! - [X] [分享已上传的 Object（Share），URL 有效期为一周](./cli/object/fn.share.html)
//! - [X] [删除已上传的 Object（Delete）](./cli/object/fn.delete.html)
//!
//! ### 中级 (15')
//!
//! - [X] [分段上传一个本地文件](./cli/object/fn.put_multipart.html)
//! - [X] [设置专属签名，实现自定义加密，使用户拥有独特的签名方式](./cli/object/fn.put_securely.html)
//!
//! ### 高级 (20')
//!
//! - [X] [设置 Object 上传时的冗余模式，使上传时可实现自定义分片模式](./cli/object/fn.put.html)
//! - [X] [通过 Post 方式上传本地文件（文件小于100M）](./cli/object/fn.post.html)
//!
//! ### 提高题 (20')
//!
//! - [X] [实现多线程上传多个对象](./cli/object/fn.put_multithread.html)
//! - [X] [list 出带前缀 `prefix/` 的所有对象，读取这些对象,删除其他对象](./cli/object/fn.delete.html)
//!

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
use env_logger::{LogBuilder, LogTarget};
use log::{LogLevel, LogLevelFilter, LogRecord};
use clap::ArgMatches;

pub mod cli;

#[allow(unused_variables)]
fn main() {
    let matches: ArgMatches = clap_app!(ct =>
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
                (@arg multipart: --mp "分片上传")
                (@arg storage_class: -s --storageclass +takes_value "储存模式")
                (@arg PASSWORD: -k --password +takes_value "密钥")
                (@arg ENCRYPT_METHOD: -e --encryptmethod +takes_value "加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）")
            )
            (@subcommand post =>
                (about: "POST 上传对象")
                (@arg key: +required +takes_value "对象唯一 ID")
                (@arg expires: -e --expires +takes_value "时间（1500s）")
            )
            (@subcommand down =>
                (about: "下载对象")
                (@arg keys: +required +multiple +takes_value "对象 ID 列表")
                (@arg dir: -o --output +takes_value "储存文件夹")
                (@arg multithread: -m --multithread "多线程下载")
                (@arg PASSWORD: -k --password +takes_value "密钥")
                (@arg ENCRYPT_METHOD: -e --encryptmethod +takes_value "加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）")
            )
            (@subcommand get =>
                (about: "读取对象")
                (@arg key: +required +takes_value "对象 ID")
                (@arg PASSWORD: -k --password +takes_value "密钥")
                (@arg ENCRYPT_METHOD: -e --encryptmethod +takes_value "加密方式（aes-128-cfb, aes-128-cfb128, aes-256-cfb, aes-256-cfb128, rc4, rc4-md5...）")
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
        (@arg hidden: --hidden "隐藏输出信息")
        (@arg verbosity: -v +multiple "设置调试等级")
    ).get_matches();

    if !matches.is_present("hidden") {
        let format = |record: &LogRecord| {
            match record.level() {
                LogLevel::Info => format!("{}", record.args()),
                _ => format!("{} - {}", record.level(), record.args()),
            }

        };
        LogBuilder::new()
            .target(LogTarget::Stdout)
            .format(format)
            .parse(match matches.occurrences_of("verbosity") {
                1 => "ct_cli=Debug",
                2 => "ct_cli=Debug, aws_sdk_rust=Debug",
                3 => "Trace",
                _ => "ct_cli=Info,Error",
            })
            .init();
    }


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
                ("post", Some(args)) => post(bucket, args),
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