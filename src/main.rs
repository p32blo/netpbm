
use std::env;

use std::fs::File;

use std::io;
use std::io::{Read, Error, ErrorKind};


fn main() {

	let mut args = env::args();

	let _exec = args.next();

	for arg in args {
		match load_img(&arg) {
			Ok(_) => println!("Load image {:?}...", arg),
			Err(e) => {
				match e.kind() {
					ErrorKind::InvalidData => println!("warn: file {:?} is not a netpbm file", arg),
					_ => println!("warn: file {:?} does not exist", arg)
				}
			}
		}
	}
}

fn load_img(filename: &str) -> Result<(), io::Error>
{
	let mut file = try!(File::open(filename));

	let mut content = String::new();
	try!(file.read_to_string(&mut content));

	let mut split = content.split_whitespace();


	if let Some(val) = split.next() {
		if val != "P3" {
			return Err(Error::new(ErrorKind::InvalidData, "File does not contain 'P3' tag"));
		}
	}

	Ok(())
}
