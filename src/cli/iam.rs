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

use ct_sdk::ct::sdk::CTClient;
use ct_sdk::ct::iam::*;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::FormatBuilder;

use colored::*;

use clap::ArgMatches;

pub fn list(args: &ArgMatches) {
    debug!("List AccessKey");
    let quiet = args.is_present("quiet");
    let all = args.is_present("all");

    let ct = CTClient::default_client();

    match ct.list_access_key(&ListAccessKeyRequest {
        ..Default::default()
    }) {
        Ok(out) => {
            let out = match all {
                false => out.access_key_metadata
                    .member
                    .iter()
                    .filter(|elm| !elm.is_primary)
                    .collect::<Vec<_>>(),
                true => out.access_key_metadata.member.iter().collect::<Vec<_>>(),
            };
            match quiet {
                false => printstd!(out, access_key_id, user_name, status, is_primary),
                true => printlist!(out, access_key_id),
            }
        }
        Err(err) => println!("{:?}", err),
    }
}

/// 创建一组 AK/SK
pub fn create(args: &ArgMatches) {
    debug!("Create Access Key");
    match CTClient::default_client().create_access_key() {
        Ok(out) => printstc!(
            out,
            access_key_id,
            secret_access_key,
            user_name,
            status,
            is_primary
        ),
        Err(err) => {
            debug!("{:#?}", err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}

/// 删除已有的 AK/SK
pub fn delete(args: &ArgMatches) {
    debug!("Delete Access Key");

    let count = args.occurrences_of("access_keys");
    let access_keys = args.values_of("access_keys").unwrap().collect::<Vec<_>>();
    let force = args.is_present("force");

    let ct = CTClient::default_client();

    let mut success = 0;
    let mut error = 0;

    access_keys.iter().for_each(|access_key| {
        print!("{}", access_key);
        match ct.delete_access_key(&DeleteAccessKeyRequest {
            access_key_id: access_key.to_string(),
        }) {
            Ok(out) => {
                debug!("{:#?}", out);
                println!("{}", " ✓ ".green().bold());
                success += 1;
            }
            Err(err) => {
                debug!("{:#?}", err);
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

/// 更改 AK/SK属性（主秘钥/普通秘钥）
pub fn update(args: &ArgMatches) {
    debug!("Update Access Key");

    let access_key_id = args.value_of("access_key_id").unwrap().to_string();
    let status = args.is_present("status");
    let is_primary = args.is_present("is_primary");

    print!("{}", access_key_id);
    match CTClient::default_client().update_access_key(&UpdateAccessKeyRequest {
        access_key_id,
        status: match status {
            true => Some(Status::Active),
            false => Some(Status::Inactive),
        },
        is_primary: Some(is_primary),
    }) {
        Ok(out) => {
            debug!("{:#?}", out);
            println!("{}", " ✓ ".green().bold());
        }
        Err(err) => {
            debug!("{:#?}", err);
            println!("{}", " ✗ ".red().bold());
        }
    }
}
