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

use colored::*;

use ct_sdk::ct::s3::acl::CannedAcl;
use ct_sdk::ct::s3::bucket::*;
use ct_sdk::ct::s3::acl::*;
use ct_sdk::ct::sdk::CTClient;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

use clap::ArgMatches;

/// 显示仓库列表(ls)
/// * `-q`, `--quiet`: 只显示名字
/// ```shell
/// $ ct-cli bucket ls
/// ```
/// * `-q`,`--quiet` 只显示名字
pub fn list(args: &ArgMatches) {
    debug!("List Buckets");

    let quiet = args.is_present("quiet");

    match CTClient::default_client().list_buckets() {
        Ok(out) => match quiet {
            false => printstd!(out.buckets, name, creation_date),
            true => printlist!(out.buckets, name),
        },
        Err(err) => print_aws_err!(err),
    }
}

/// 创建新仓库(mb)
/// ```shell
/// $ ct-cli bucket new <bucket_name>
/// ```
pub fn create(args: &ArgMatches) {
    debug!("Create Bucket");

    let bucket = args.value_of("bucket_name").unwrap();

    print!("{}", bucket);
    match CTClient::default_client().create_bucket(&CreateBucketRequest {
        bucket: bucket.to_string(),
        ..Default::default()
    }) {
        Ok(out) => {
            debug!("{:#?}", out);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            print_aws_err!(err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}

/// 更改属性（私有、公有、只读）
/// ```shell
/// $ ct-cli bucket set [-rw] <bucket_name>
/// ```
/// * `-r`,`--read` 可公开读取
/// * `-w`,`--write` 可公开写入
pub fn acl(args: &ArgMatches) {
    debug!("ACL");

    let bucket = args.value_of("bucket_name").unwrap();
    let read = args.is_present("read");
    let write = args.is_present("write");

    print!("{}", bucket);
    match CTClient::default_client().put_bucket_acl(&PutBucketAclRequest {
        bucket: bucket.to_string(),
        acl: Some(match (read, write) {
            (true, true) => CannedAcl::PublicReadWrite,
            (true, false) => CannedAcl::PublicRead,
            (false, false) => CannedAcl::Private,
            _ => CannedAcl::Private,
        }),
        ..Default::default()
    }) {
        Ok(out) => {
            debug!("{:#?}", out);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            print_aws_err!(err);
            println!("{}", " ✗ ".red().bold());
        }
    };
}

/// 删除空仓库(rb)
/// **只能删除**空仓库，但是可以采用 (-f, --force) 自动删除仓库对象，并删除仓库．
/// ```shell
/// $ ct-cli bucket rm <buckets>...
/// ```
/// 本条命令可以和 `ls -q` 配合使用
/// ```shell
/// $ ct-cli bucket rm $(ct-cli bucket ls -q)
/// ```
/// *Note: `--force` 暂未实现，可使用如下命令代替*
// TODO: `--force` 暂未实现
/// ```shell
/// $ ct-cli object <bucket> rm $(ct-cli object <bucket> ls -q)
/// $ ct-cli bucket rm <bucket>
/// ```
pub fn delete(args: &ArgMatches) {
    debug!("Delete Bucket");
    let count = args.occurrences_of("buckets");
    let buckets = args.values_of("buckets").unwrap().collect::<Vec<_>>();
    let force = args.is_present("force");

    let ct = CTClient::default_client();

    let mut success = 0;
    let mut error = 0;

    buckets.iter().for_each(|bucket| {
        print!("{}\t", bucket);
        match ct.delete_bucket(&DeleteBucketRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }) {
            Ok(output) => {
                debug!("{:#?}", output);
                println!("{}", " ✓ ".green().bold());
                success += 1;
            }
            Err(err) => {
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
