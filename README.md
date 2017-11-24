# RCTYUN

[Document](https://rctyun.gitai.me) | [Github](https://github.com/GitaiQAQ/RCTYUN)


## List of features

* Rust library for call API operations which each return response data and a possible error.
* Supports client-side-encrypt
* Multi-threaded upload and download
* Command line interface, mostly useful for debugging

## Library usage

In cargo.toml:

```toml
[dependencies]
ct-sdk = "0.1"
```

## Example

```rust
extern crate output;
extern crate ct_sdk;

use ct_sdk::ct::sdk::CTClient;
use ct_sdk::ct::s3::bucket::*;

fn main() {
    match CTClient::default_client().list_buckets() {
        Ok(out) => prinln!("{:#?}", out),
        Err(err) => prinln!("{:#?}", err),
    }
}
```

参考

* shadowsocks-rust