
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;

fn main() {

	let mut args = env::args();

	let _exec = args.next();

	for arg in args {
		match load_img(&arg) {
			Ok(_) => println!("Load image {:?}...", arg),
			Err(_) => println!("warn: file {:?} does not exist", arg)
		}
	}
}

fn load_img(filename: &str) -> Result<(), io::Error>
{
	let file = try!(File::open(filename));
	
	println!("debug: {:?}", filename);

	Ok(())
}
