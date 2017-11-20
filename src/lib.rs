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

//! `ct_sdk` is a Keep-It-Simple-Stupid CTYun OOS SDK for Rust.
//!
//! ## About
//!
//! `ct_sdk` is based on `aws_sdk_rust` and add some method.
//!
//! ## Quick Example
//!
//! At first, Connecting to the CTYun Object-Oriented-Storage Service.
//!
//! **NOTE:** The Config of CTYun OOS in the method
//! [`default_ctyun_client`](ct/sdk/trait.CTClient.html) by trait `cli::CTClient`.
//!
//! The following examples show a quick example of list buckets.
//! For more advanced usage, such as `share_object` is added, But other usage like CURD of
//! object, BUCKET see the
//! [documentation](https://lambdastackio.github.io/aws-sdk-rust/aws_sdk_rust/aws/index.html),
//!
//! ```
//! # extern crate aws_sdk_rust;
//! # extern crate ct_sdk;
//!
//! # use aws_sdk_rust::aws::common::credentials::DefaultCredentialsProvider;
//! # use aws_sdk_rust::aws::s3::s3client::S3Client;
//! # use ct_sdk::sdk::CTClient;
//!
//! # fn main() {
//!     let provider = DefaultCredentialsProvider::new(None).unwrap();
//!     let s3 = S3Client::default_ctyun_client(provider);
//!
//!     match s3.list_buckets() {
//!         Ok(out) => println!("{:#?}", out),
//!         Err(err) => println!("{:#?}", err),
//!     }
//! # }
//! ```

#![crate_type = "lib"]
extern crate aws_sdk_rust;
extern crate chrono;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate md_5 as md5;
extern crate bytes;
extern crate ring;
extern crate rand;
extern crate openssl;
extern crate rustc_serialize;
extern crate url;
extern crate xml;
extern crate typenum;
extern crate digest;
extern crate byteorder;
extern crate byte_string;

pub mod ct;

#[cfg(test)]
mod tests {}
