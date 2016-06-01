
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;

fn main() {

	let mut args = env::args();

	let _exec = args.next();

	for arg in args {

		if let Ok(()) = load_img(&arg) {
	    	println!("Load image {:?}", arg);
		} else {
			println!("warn: file {} not exits", arg);
		}
	}
}

fn load_img(filename: &str) -> Result<(), io::Error>
{
	let file = try!(File::open(filename));
	
	println!("debug: {:?}", filename);

	Ok(())
}
