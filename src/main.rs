
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

	let (hash, number) = split.next().unwrap().split_at(1);

	let iters: usize = number.parse().unwrap();

	if hash != "#" {
		return Err(Error::new(ErrorKind::InvalidData, "File does not have required metadata"));
	}

	let size_x: usize = split.next().unwrap().parse().unwrap();
	let size_y: usize = split.next().unwrap().parse().unwrap();
	let max_val: usize = split.next().unwrap().parse().unwrap();

	println!("debug: iters = {:?}, {:?}", hash, iters);
	println!("debug: size_x = {:?}", size_x);
	println!("debug: size_y = {:?}", size_y);
	println!("debug: max_val = {:?}", max_val);

	let img_size = size_x * size_y;
	let img_rgb_size = img_size * 3;

	let mut img = vec![0; img_rgb_size];

	for (i, word) in split.enumerate() {
		img[i] = word.parse().unwrap();
//		println!("debug: {}, {:?}, {}", i, word, img[i]);
	}

	println!("debug: vec_size = {}", img.len());

	Ok(())
}
