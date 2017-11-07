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

//! Additional API for Object Operations
//!
use std::str::FromStr;
use std::str;

use hyper::client::Client;

use aws_sdk_rust::aws::common::signature::SignedRequest;
use aws_sdk_rust::aws::common::credentials::AwsCredentialsProvider;
use aws_sdk_rust::aws::s3::s3client::S3Client;
use aws_sdk_rust::aws::s3::s3client::sign_and_execute;

use aws_sdk_rust::aws::common::xmlutil::*;
use aws_sdk_rust::aws::common::common::*;
use aws_sdk_rust::aws::errors::aws::AWSError;

use aws_sdk_rust::aws::errors::s3::S3Error;

use xml::EventReader;

pub type IsPrimary = bool;

pub type AccessKeyId = String;

pub type SecretAccessKey = String;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub enum Status {
    Active,
    Inactive,
}

impl Default for Status {
    fn default() -> Self {
        Status::Active
    }
}

/// `AccessKeyMetadata` used for `Contents` for ListAccessKeyOutput
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct AccessKeyMetadata {
    pub user_name: DisplayName,
    pub access_key_id: AccessKeyId,
    pub status: Status,
    pub is_primary: bool,
}

/// `AccessKeyMetadata` used for `Contents` for ListAccessKeyOutput
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct AccessKeyMetadataList {
    pub member: Vec<AccessKeyMetadata>,
}

//#[derive(Debug, Default)]
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct ListAccessKeyRequest {
    pub max_items: Option<String>,
    pub marker: Option<String>,
}

/// Default output of all admin functions
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct ListAccessKeyOutput {
    pub access_key_metadata: AccessKeyMetadataList,
    pub is_truncated: bool,
    pub marker: String,
}


/// Parse `String` from XML
pub struct StringParser;

impl StringParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<AccessKeyId, XmlParseError> {
        try!(start_element(tag_name, stack));
        let obj = try!(characters(stack));
        try!(end_element(tag_name, stack));
        Ok(obj)
    }
}

/// Parse `Bool` from XML
pub struct BoolParser;

impl BoolParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<IsPrimary, XmlParseError> {
        try!(start_element(tag_name, stack));

        let mut obj = IsPrimary::default();

        match characters(stack) {
            Err(_) => return Ok(obj),
            Ok(ref chars) => obj = bool::from_str(chars).unwrap(),
        }

        try!(end_element(tag_name, stack));
        Ok(obj)
    }
}

/// Parse `Marker` from XML
pub type MarkerParser = StringParser;

/// Parse `AccessKeyId` from XML
type AccessKeyIdParser = StringParser;

/// Parse `Status` from XML
pub struct StatusParser;

/// Parse `IsPrimary` from XML
pub type IsPrimaryParser = BoolParser;

/// Parse `AccessKeyMetadataList` from XML
pub struct AccessKeyMetadataListParser;

/// Parse `AccessKeyMetadata` from XML
pub struct AccessKeyMetadataParser;

/// Parse `ListAccessKeyOutput` from XML
pub struct ListAccessKeyOutputParser;

impl StatusParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<Status, XmlParseError> {
        try!(start_element(tag_name, stack));
        let obj = try!(characters(stack));
        try!(end_element(tag_name, stack));
        if obj == "Active" {
            Ok(Status::Active)
        } else {
            Ok(Status::Inactive)
        }
    }
}

impl AccessKeyMetadataParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<AccessKeyMetadata, XmlParseError> {
        try!(start_element(tag_name, stack));
        let mut obj = AccessKeyMetadata::default();
        loop {
            let current_name = try!(peek_at_name(stack));
            if current_name == "UserName" {
                obj.user_name = try!(DisplayNameParser::parse_xml("UserName", stack));
                continue;
            }
            if current_name == "AccessKeyId" {
                obj.access_key_id = try!(AccessKeyIdParser::parse_xml("AccessKeyId", stack));
                continue;
            }
            if current_name == "Status" {
                obj.status = try!(StatusParser::parse_xml("Status", stack));
                continue;
            }
            if current_name == "IsPrimary" {
                obj.is_primary = try!(IsPrimaryParser::parse_xml("IsPrimary", stack));
                continue;
            }
            break;
        }
        try!(end_element(tag_name, stack));
        Ok(obj)
    }
}

impl AccessKeyMetadataListParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<AccessKeyMetadataList, XmlParseError> {
        let mut obj = Vec::new();
        while try!(peek_at_name(stack)) == tag_name {
            obj.push(try!(AccessKeyMetadataParser::parse_xml(tag_name, stack)));
        }

        Ok(AccessKeyMetadataList {
            member: obj,
        })
    }
}

impl ListAccessKeyOutputParser {
    pub fn parse_xml<T: Peek + Next>(tag_name: &str, stack: &mut T) -> Result<ListAccessKeyOutput, XmlParseError> {
        try!(start_element(tag_name, stack));
        let mut obj = ListAccessKeyOutput::default();
        loop {
            let current_name = try!(peek_at_name(stack));
            if current_name == "AccessKeyMetadata" {
                obj.access_key_metadata = try!(AccessKeyMetadataListParser::parse_xml("AccessKeyMetadata", stack));
                continue;
            }
            if current_name == "IsTruncated" {
                obj.is_truncated = try!(IsTruncatedParser::parse_xml("IsTruncated", stack));
                continue;
            }
            if current_name == "Marker" {
                obj.marker = try!(MarkerParser::parse_xml("Marker", stack));
                continue;
            }
            break;
        }
        try!(end_element(tag_name, stack));
        Ok(obj)
    }
}

/// Default output of all admin functions
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct CreateAccessKeyOutput {
    /// status code from the restful server
    pub status: Status,
    pub user_name: String,
    pub access_key_id: AccessKeyId,
    pub secret_access_key: SecretAccessKey,
    pub is_primary: bool,
}

//#[derive(Debug, Default)]
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct DeleteAccessKeyRequest {
    pub access_key_id: AccessKeyId,
}

/// Default output of all admin functions
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct DeleteAccessKeyOutput {
    pub request_id: String,
}

//#[derive(Debug, Default)]
#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct UpdateAccessKeyRequest {
    pub access_key_id: AccessKeyId,
    pub status: Status,
    pub is_primary: bool,
}

/// Default output of all admin functions
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct UpdateAccessKeyOutput {
    pub request_id: String,
}

pub trait CTClientIAM {
    fn list_access_key(&self, input:&ListAccessKeyRequest)
        -> Result<ListAccessKeyOutput, S3Error>;
    fn create_access_key(&self)
        -> Result<CreateAccessKeyOutput, S3Error>;
    fn delete_access_key(&self, input:&DeleteAccessKeyRequest)
        -> Result<DeleteAccessKeyOutput, S3Error>;
    fn update_access_key(&self, input:&UpdateAccessKeyRequest)
        -> Result<UpdateAccessKeyOutput, S3Error>;
}

impl<P> CTClientIAM for S3Client<P, Client>
    where P: AwsCredentialsProvider,
{
    fn list_access_key(&self, input:&ListAccessKeyRequest)
        -> Result<ListAccessKeyOutput, S3Error> {
        let mut request = SignedRequest::new(
            "POST",
            "s3",
            self.region(),
            "",
            "/",
            self.endpoint());

        request.set_hostname(Some(String::from("oos-bj2-iam.ctyunapi.cn")));
        request.add_param("Action", "ListAccessKey");

        let result = sign_and_execute(&self.dispatcher,
                                      &mut request,
                                      try!(self.credentials_provider.credentials()));

        let status = result.status;
        let mut reader = EventReader::from_str(&result.body);
        let mut stack = XmlResponse::new(reader.into_iter().peekable());
        stack.next(); // xml start tag

        match status {
            200 => {
                Ok(try!(ListAccessKeyOutputParser::parse_xml("ListAccessKeysResult", &mut stack)))
            },
            _ => {
                let aws = try!(AWSError::parse_xml("Error", &mut stack));
                Err(S3Error::with_aws("Error listing access keys", aws))
            },
        }
    }

    /// 创建一组 AK/SK
    fn create_access_key(&self)
        -> Result<CreateAccessKeyOutput, S3Error>
    {
        unimplemented!()
    }

    /// 删除已有的 AK/SK
    fn delete_access_key(&self, input:&DeleteAccessKeyRequest)
        -> Result<DeleteAccessKeyOutput, S3Error>
    {
        unimplemented!()
    }

    /// 更改 AK/SK属性（主秘钥/普通秘钥）
    fn update_access_key(&self, input:&UpdateAccessKeyRequest)
        -> Result<UpdateAccessKeyOutput, S3Error>
    {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use aws_sdk_rust::aws::common::credentials::*;
    use aws_sdk_rust::aws::s3::s3client::S3Client;

    use super::super::sdk::CTClient;
    use super::CTClientIAM;
    use ct::iam::ListAccessKeyRequest;

    #[test]
    fn list() {
        let provider = DefaultCredentialsProvider::new(None).unwrap();
        let s3 = S3Client::default_ctyun_client(provider);

        match s3.list_access_key( &ListAccessKeyRequest {
            ..Default::default()
        }) {
            Ok(out) => {println!("{:?}", out)},
            Err(err) => println!("{:?}", err),
        }
    }
}