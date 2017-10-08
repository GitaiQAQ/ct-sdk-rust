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

// NOTE: See the src/main.rs for more examples...
#[macro_use]
extern crate clap;
extern crate aws_sdk_rust;
#[macro_use]
extern crate log;
extern crate md5;
extern crate hyper;
extern crate env_logger;
extern crate ct_sdk;
extern crate prettytable;
extern crate rustc_serialize;

use clap::{ArgMatches};

use aws_sdk_rust::aws::common::credentials::*;
use aws_sdk_rust::aws::s3::s3client::S3Client;

mod cli;
use cli::CTClient;

fn main() {
    env_logger::init().unwrap();
    debug!("ct-cli start...");

    let _matches: ArgMatches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Gitai<i@gitai.me>")
        (about: "Does awesome things")
        (@subcommand config =>
            (about: "Write AK/SK to disk(~/.aws/credentials)")
            (@arg aws_access_key_id: +required -a --ak +takes_value "AK/AWS Access Key Id")
            (@arg aws_secret_access_key: +required -s --sk +takes_value "SK/AWS Secret Access Key")
        )
        (@subcommand bucket =>
            (@subcommand ls =>
                (about: "List buckets")
                (@arg quiet: -q --quiet "Only display Names")
            )
            (@subcommand new =>
                (about: "Create a bucket")
                (@arg name: +required +takes_value "Bucket while be create")
            )
            (@subcommand rm =>
                (about: "Delete bucket")
                (@arg name: +required +takes_value "Bucket while be delete")
            )
        )
        (@subcommand object =>
            (@subcommand ls =>
                (about: "List objects")
                (@arg bucket: +required +takes_value "Where")
                (@arg quiet: -q --quiet "Only display Names")
            )
            (@subcommand new =>
                (about: "Create a objects")
                (@arg bucket: +required +takes_value "Object while be create")
                (@arg key: +required +takes_value "Path")
                (@arg body: +required +multiple +takes_value "Body")
            )
            (@subcommand put =>
                (about: "Put a objects")
                (@arg bucket: +required +takes_value "Object while be create")
                (@arg keys: +required +multiple +takes_value "Key")
            )
            (@subcommand get =>
                (about: "Get object")
                (@arg bucket: +required +takes_value "Object while be delete")
                (@arg keys: +required +multiple +takes_value "Key")
            )
            (@subcommand rm =>
                (about: "Delete objects")
                (@arg bucket: +required +takes_value "Object while be delete")
                (@arg keys: +required +multiple +takes_value "Key")
            )
            (@subcommand share =>
                (about: "Share object")
                (@arg bucket: +required +takes_value "Object while be delete")
                (@arg key: +required +takes_value "Key")
                (@arg expires: -e --expires +takes_value "Expires")
            )
        )
        (@arg aws_access_key_id: -a --ak +takes_value "AK/AWS Access Key Id")
        (@arg aws_secret_access_key: -s --sk +takes_value "SK/AWS Secret Access Key")
    ).get_matches();

    debug!("OOS connecting.");

    let provider = DefaultCredentialsProvider::new(None).unwrap();
    let s3 = S3Client::default_ctyun_client(provider);
    debug!("OOS connected.");
    // use cli::object::CTCLIObject;
    // s3.share(String::from("gitai"), String::from("date.txt"), None);
    // s3.list(false, String::from("gitai"), None);
    use cli::bucket::CTCLIBucket;
    s3.create(String::from("gitai.test"));
    s3.delete(String::from("gitai.test"));
    debug!("end");
}
