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
extern crate ct_sdk;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate md5;
extern crate prettytable;
extern crate rustc_serialize;

use clap::ArgMatches;

mod cli;
use cli::CTClient;

// http://oos-bj2.ctyunapi.cn
#[allow(unused_variables)]
fn main() {
    env_logger::init().unwrap();
    debug!("Application START");
    debug!("ArgMatches init start");
    let matches: ArgMatches = clap_app!(myapp =>
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
                (about: "Create a BUCKET")
                (@arg name: +required +takes_value "Bucket while be create")
            )
            (@subcommand rm =>
                (about: "Delete BUCKET")
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
    debug!("ArgMatches init end");

    match matches.subcommand() {
        ("bucket", Some(matches)) => {
            use cli::bucket::*;
            match matches.subcommand() {
                ("new", Some(matches)) => {}
                ("rm", Some(matches)) => {}
                ("ls", Some(matches)) => {}
                _ => {}
            }
        }
        ("object", Some(matches)) => {
            use cli::object::*;
            match matches.subcommand() {
                ("put", Some(matches)) => {}
                ("get", Some(matches)) => {}
                ("down", Some(matches)) => {}
                ("rm", Some(matches)) => {}
                ("share", Some(matches)) => {}
                _ => {}
            }
        }
        ("iam", Some(matches)) => {
            use cli::iam::*;
            match matches.subcommand() {
                ("new", Some(matches)) => {}
                ("rm", Some(matches)) => {}
                ("ls", Some(matches)) => {}
                ("set", Some(matches)) => {}
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
