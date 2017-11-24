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

//! CTYun OOS SDK

use url::Url;
use chrono::UTC;
use md5::{Digest, Md5};

use hyper::client::Client;
use hyper::header::Headers;

use std::str;
use std::io::Write;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use rustc_serialize::base64::{ToBase64, STANDARD};

use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET, QUERY_ENCODE_SET};

use aws_sdk_rust::aws::common::region::Region;
use aws_sdk_rust::aws::common::params::Params;
use aws_sdk_rust::aws::common::signature::SignedRequest;
use aws_sdk_rust::aws::common::credentials::{AwsCredentials, DefaultCredentialsProvider};
use aws_sdk_rust::aws::s3::endpoint::{Endpoint, Signature};
use aws_sdk_rust::aws::s3::s3client::S3Client;

/// A trait to abstract the idea of generate a pre-signed Url for an S3 object from a SignedRequest.
pub trait CTSignedRequest<'a> {
    /// Pre-signed SignedRequest use SignV2
    fn presigned(&mut self, creds: &AwsCredentials, date: &Option<String>) -> (String, String);

    /// Add a value to the array of headers for the specified key.
    /// Headers are kept sorted by key name for use at signing (BTreeMap).
    /// But in the query the content don`t need to lowercase.(RFC2616)
    fn add_header_raw(&mut self, key: &str, value: &str);
    /// Generate Url from a SignedRequest
    fn gen_url(&mut self) -> String;
    fn url(&mut self) -> String;
}

impl<'a> CTSignedRequest<'a> for SignedRequest<'a> {
    fn presigned(&mut self, creds: &AwsCredentials, date: &Option<String>) -> (String, String) {
        // NOTE: Check the BUCKET and path
        if self.endpoint.is_bucket_virtual {
            if self.bucket.contains(".") && !self.path.contains(&format!("/{}/", self.bucket)) {
                self.path = format!("/{}{}", self.bucket, self.path);
            }
        } else if !self.path.contains(&format!("/{}/", self.bucket)) {
            self.path = format!(
                "{}{}{}",
                if self.bucket.len() > 0 { "/" } else { "" },
                self.bucket,
                self.path
            );
        } // Leave untouched if none of the above match

        // Signature::V2
        let hostname = match self.hostname {
            Some(ref h) => h.to_string(),
            None => build_hostname(&self.service, self.region),
        };

        // Gotta remove and re-add headers since by default they append the value.
        // If we're following　a 307 redirect we end up with Three Stooges in the
        // headers with duplicate values.
        self.update_header("Host", &hostname);

        // V2 uses GMT in long format
        let date_str = match *date {
            Some(ref h) => h.to_string(),
            None => format!("{}", UTC::now().timestamp() + (60 * 60)),
        };
        self.update_header("Date", &date_str);

        self.canonical_query_string = build_canonical_query_string(&self.params);

        let md5 = self.get_header("Content-MD5");

        // NOTE: canonical_headers_v2 may should pull back /{BUCKET}/{key}
        // AWS takes BUCKET (host) and uses it for calc

        let string_to_sign = format!(
            "{}\n{}\n\n{}\n{}{}",
            &self.method,
            md5,
            date_str,
            canonical_headers_v2(&self.headers),
            canonical_resources_v2(&self.bucket, &self.path, self.endpoint.is_bucket_virtual)
        );

        debug!("String to Sign: {}", string_to_sign);

        match self.payload {
            None => {
                self.update_header("Content-Length", &format!("{}", 0));
            }
            Some(payload) => {
                self.update_header("Content-Length", &format!("{}", payload.len()));
                // println!("--------payload---------");
                // println!("{:#?}", payload);
            }
        }

        // println!("canonical_query_string {:#?}", self.canonical_query_string);
        // println!("string_to_sign {:#?}", string_to_sign);
        // println!("===================");

        let signature = {
            let hmac_pkey = PKey::hmac(creds.aws_secret_access_key().as_bytes()).unwrap();
            let mut hmac = Signer::new(MessageDigest::sha1(), &hmac_pkey).unwrap();
            let _ = hmac.write_all(string_to_sign.as_bytes());
            hmac.finish().unwrap().to_base64(STANDARD)
        };

        (date_str, signature)
    }

    fn add_header_raw(&mut self, key: &str, value: &str) {
        // let key_lower = key.to_ascii_lowercase().to_string(); // RFC2616
        let key_lower = key.to_string(); // For Convert Request To Url
        let value_vec = value.as_bytes().to_vec();

        match self.headers.entry(key_lower) {
            Entry::Vacant(entry) => {
                let mut values = Vec::new();
                values.push(value_vec);
                entry.insert(values);
            }
            Entry::Occupied(entry) => {
                entry.into_mut().push(value_vec);
            }
        }
    }

    fn url(&mut self) -> String {
        let epp = self.endpoint().clone().endpoint.unwrap().port();
        let port_str = match epp {
            Some(port) => format!(":{}", port),
            _ => "".to_string(),
        };

        format!(
            "{}://{}{}{}",
            self.endpoint_scheme(),
            self.hostname(),
            port_str,
            self.path()
        ).to_string()
    }

    fn gen_url(&mut self) -> String {
        //self.endpoint()
        //println!("{:#?} {} {}", self, creds.aws_access_key_id(), self.signature);
        let epp = self.endpoint().clone().endpoint.unwrap().port();
        let port_str = match epp {
            Some(port) => format!(":{}", port),
            _ => "".to_string(),
        };

        let mut final_uri = format!(
            "{}://{}{}{}",
            self.endpoint_scheme(),
            self.hostname(),
            port_str,
            self.path()
        );

        let mut hyper_headers = Headers::new();
        for h in self.headers().iter() {
            hyper_headers.set_raw(h.0.to_owned(), h.1.to_owned());
        }


        let mut serializer = form_urlencoded::Serializer::new(String::new());

        for hyper_header in hyper_headers.iter() {
            serializer.append_pair(
                hyper_header.name().as_ref(),
                hyper_header.value_string().as_ref(),
            );
        }
        final_uri.push_str("?");
        final_uri.push_str(serializer.finish().as_ref());

        return final_uri;
    }
}

/// Used to perform client-side encryption for storing data securely in OOS. Data
/// encryption is done using a one-time randomly generated content encryption
/// key (CEK) per S3 object.
use std::ops::Deref;
use bytes::{BufMut, Bytes, BytesMut};
use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;

/// A trait to set the CTYun OOS Config default, like SignV2 and Endpoint.

pub struct CTClient {
    p: S3Client<DefaultCredentialsProvider, Client>,
    /// Encryption password (key)
    password: String,
    /// Encryption type (method)
    method: CipherType,
    /// Encryption key
    enc_key: Bytes,
}

impl Deref for CTClient {
    type Target = S3Client<DefaultCredentialsProvider, Client>;
    fn deref<'a>(&'a self) -> &'a S3Client<DefaultCredentialsProvider, Client> {
        &self.p
    }
}

use ct::crypto_io::CipherType;

impl CTClient {
    pub fn new(
        credentials_provider: DefaultCredentialsProvider,
        pwd: Option<String>,
        method: Option<CipherType>,
    ) -> CTClient {
        // Init new s3 connect
        // V4 is the default signature for AWS. However, other systems also use V2.
        let endpoint = Endpoint::new(
            Region::UsEast1,
            Signature::V2,
            match Url::parse("http://oos-bj2.ctyunapi.cn") {
                Ok(url) => Some(url),
                Err(e) => {
                    error!("{:#?}", e);
                    None
                }
            },
            None,
            None,
            None,
        );

        let method = match method {
            Some(method) => method,
            None => CipherType::Aes256Cfb
        };

        let pwd = pwd.unwrap_or_default();

        let enc_key = method.bytes_to_key(pwd.as_bytes());
        trace!("Initialize config with pwd: {:?}, key: {:?}", pwd, enc_key);

        CTClient {
            password: pwd,
            method: method,
            enc_key: enc_key,
            p: S3Client::new(credentials_provider, endpoint),
        }
    }

    /// Get encryption key
    pub fn key(&self) -> &[u8] {
        &self.enc_key[..]
    }

    // Get password
    //pub fn password(&self) -> &str {
    //    &self.password[..]
    //}

    /// Get method
    pub fn method(&self) -> CipherType {
        self.method
    }

    /// Set the CTYun OOS Config default
    pub fn default_client() -> Self {
        let credentials_provider = DefaultCredentialsProvider::new(None).unwrap();
        CTClient::new(credentials_provider, None, None)
    }

    /// Set the CTYun OOS Config default
    #[allow(unused_variables)]
    pub fn default_securely_client(pwd: String, encrypt_method: CipherType) -> Self {
        let credentials_provider = DefaultCredentialsProvider::new(None).unwrap();
        CTClient::new(credentials_provider,
                      Some(pwd),
                      Some(encrypt_method))
    }
}

// From aws_sdk_rust::aws::common::signature
// Private functions used to support the Signature Process...

fn canonical_values(values: &[Vec<u8>]) -> String {
    let mut st = String::new();
    for v in values {
        let s = str::from_utf8(v).unwrap();
        if !st.is_empty() {
            st.push(',')
        }
        if s.starts_with('\"') {
            st.push_str(s);
        } else {
            st.push_str(s.replace("  ", " ").trim());
        }
    }
    st
}

// NOTE: Don't add user-agent since it's part of the signature calc for V4
fn skipped_headers(header: &str) -> bool {
    ["authorization", "content-length", "content-type"].contains(&header)
}

fn build_canonical_query_string(params: &Params) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for item in params.iter() {
        if !output.is_empty() {
            output.push_str("&");
        }
        output.push_str(&byte_serialize(item.0));
        output.push_str("=");
        output.push_str(&byte_serialize(item.1));
    }

    output
}

pub fn md5(input: &[u8]) -> String {
    let mut sh = Md5::default();
    sh.consume(input);
    sh.hash().to_base64(STANDARD)
}

#[inline]
fn encode_uri(uri: &str) -> String {
    utf8_percent_encode(uri, QUERY_ENCODE_SET).collect::<String>()
}

#[inline]
fn byte_serialize(input: &str) -> String {
    utf8_percent_encode(input, DEFAULT_ENCODE_SET).collect::<String>()
}

// NOTE: Used to build a hostname from a set of defaults. Use set_hostname is preferred.
fn build_hostname(service: &str, region: Region) -> String {
    // iam has only 1 endpoint, other services have region-based endpoints
    match service {
        "iam" => match region {
            Region::CnNorth1 => format!("{}.{}.amazonaws.com.cn", service, region),
            _ => format!("{}.amazonaws.com", service),
        },
        "s3" => match region {
            Region::UsEast1 => "s3.amazonaws.com".to_string(),
            Region::CnNorth1 => format!("s3.{}.amazonaws.com.cn", region),
            _ => format!("s3-{}.amazonaws.com", region),
        },
        _ => match region {
            Region::CnNorth1 => format!("{}.{}.amazonaws.com.cn", service, region),
            _ => format!("{}.{}.amazonaws.com", service, region),
        },
    }
}
// Common to V2 and V4 - End

// V2 Signature related - Begin
fn canonical_headers_v2(headers: &BTreeMap<String, Vec<Vec<u8>>>) -> String {
    let mut canonical = String::new();

    // NOTE: May need to add to vec, sort and then do the following for x-amz-

    for item in headers.iter() {
        if skipped_headers(item.0) {
            continue;
        } else {
            match item.0.to_ascii_lowercase().find("x-amz-") {
                None => {}
                _ => canonical.push_str(
                    format!(
                        "{}:{}\n",
                        item.0.to_ascii_lowercase(),
                        canonical_values(item.1)
                    ).as_ref(),
                ),
            };
        }
    }

    canonical
}

// NOTE: If BUCKET contains '.' it is already formatted in path so just encode it.
fn canonical_resources_v2(bucket: &str, path: &str, is_bucket_virtual: bool) -> String {
    if bucket.to_string().contains(".") || !is_bucket_virtual {
        encode_uri(path)
    } else {
        match bucket {
            "" => {
                match path {
                    "" => "/".to_string(),
                    _ => encode_uri(path), // This assumes / as leading char
                }
            }
            _ => {
                match path {
                    "" => format!("/{}/", bucket),
                    _ => encode_uri(&format!("/{}{}", bucket, path)),
                    // This assumes path with leading / char
                }
            }
        }
    }
}
// V2 Signature related - End
