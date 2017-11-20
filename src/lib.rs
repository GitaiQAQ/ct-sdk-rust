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
//! [`defaul_client`](ct/sdk/struct.CTClient.html) by trait `cli::CTClient`.
//!
//! The following examples show a quick example of list buckets.
//! For more advanced usage, such as `share_object` is added, But other usage like CURD of
//! object, bucket see the
//! [documentation](https://lambdastackio.github.io/aws-sdk-rust/aws_sdk_rust/aws/index.html),
//!
//! ```
//! extern crate ct_sdk;
//!
//! use ct_sdk::ct::sdk::CTClient;
//! use ct_sdk::ct::s3::bucket::*;
//!
//! fn main() {
//!     match CTClient::default_client().list_buckets() {
//!         Ok(out) => prinln!("{:#?}", out),
//!         Err(err) => prinln!("{:#?}", err),
//!     }
//! }
//! ```

#![crate_type = "lib"]
extern crate aws_sdk_rust;
extern crate byte_string;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate digest;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate md_5 as md5;
extern crate openssl;
extern crate rand;
extern crate ring;
extern crate rustc_serialize;
extern crate typenum;
extern crate url;
extern crate xml;

pub mod ct;

#[cfg(test)]
mod tests {}
