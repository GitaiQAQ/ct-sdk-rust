# RCTYUN

[Website](https://rctyun.gitai.me) | [Github](https://github.com/GitaiQAQ/RCTYUN)


## List of features

* Rust library for call API operations which each return response data and a possible error.
* Supports encrypted and non encrypted formats
* Multi-threaded upload and download
* Command line decrypt utility, mostly useful for debugging

## Library usage

In cargo.toml:

```toml
[dependencies]
ct-sdk = "0.1"
```

## Example

A basic example (from `examples/restore.rs`):

```rust
extern crate output;
extern crate ct_sdk;

use std::env;
use std::ffi::OsString;
use std::io;

fn main () {

	let output =
		output::open ();

	let arguments: Vec <OsString> =
		env::args_os ().collect ();

	let stdout =
		io::stdout ();

	let mut stdout_lock =
		stdout.lock ();

	repository.restore (
		& output,
		arguments [3],
		& mut stdout_lock,
	).unwrap ();

}
```

## Command usage

### List Buckets


```sh
ct list 
```