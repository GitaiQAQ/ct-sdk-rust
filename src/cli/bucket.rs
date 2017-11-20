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

/// List buckets(ls)
pub fn list(args: &ArgMatches) {
    debug!("List Bucket");

    let quiet = args.is_present("quiet");

    match CTClient::default_client().list_buckets() {
        Ok(out) => match quiet {
            false => printstd!(out.buckets, name, creation_date),
            true => printlist!(out.buckets, name),
        },
        Err(err) => print_aws_err!(err),
    }
}

/// 创建一个 Bucket
/// Creates an BUCKET(mb)
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

// TODO: 更改创建的 Bucket属性（私有、公有、只读）
pub fn acl(args: &ArgMatches) {
    debug!("acl");
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

/// 删除已创建的 Bucket
/// Deletes an empty BUCKET.(rb)
/// A BUCKET must be completely empty of objects and versioned objects before it can be deleted.
/// However, the --force parameter can be used to delete the non-versioned objects in the BUCKET
/// before the BUCKET is deleted.
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
