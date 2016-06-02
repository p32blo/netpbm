
use std::env;

use std::fs::File;

use std::io;
use std::io::{Read, Write, Error, ErrorKind};


#[derive(Default)]
struct Image {
	iters : usize,
	size_x : usize,
	size_y : usize,
	max_val : usize,
	data : Vec<usize>
}

fn main() {

	let mut args = env::args();

	let _exec = args.next();

	let mut image = Image::new();

	for arg in &mut args {
		match Image::open(&arg) {
			Ok(img) => {
				image = img;
				println!("reading: {} [ {} x {} ] iters = {}",
							arg, image.size_x, image.size_y, image.iters);
				break;
			},
			Err(e) => handle_error(e, &arg)
		}
	}

	for arg in args {
		match image.add(&arg) {
			Ok(img) => {
				println!("reading: {} [ {} x {} ] iters = {}",
							arg, img.size_x, img.size_y,	img.iters);
			}
			Err(e) => handle_error(e, &arg)
		}
	}

	if !image.is_empty() {

		image.divide();

		let out = "final.ppm";
		println!("writing: {} [ {} x {} ] iters = {}",
					out, image.size_x, image.size_y, image.iters);

		image.save(out).unwrap();
	}
}

fn handle_error(e: io::Error, arg: &str)
{
	match e.kind() {
		ErrorKind::InvalidData => println!("warn: file {:?} is not a netpbm file", arg),
		_ => println!("warn: file {:?} does not exist", arg)
	}
}

impl Image {

	fn new() -> Self { Image::default() }

	fn is_empty(&self) -> bool { self.data.is_empty() }

	fn open(filename: &str) -> io::Result<Self>
	{
		let (mut image, content) = try!(Self::read_metadata(filename));

		let mut split = content.split_whitespace();

		let _metadata = split.nth(4);

		let img_size = image.size_x * image.size_y;
		let img_rgb_size = img_size * 3;

		let mut img = vec![0; img_rgb_size];

		for (i, word) in split.enumerate() {
			let val: usize = word.parse().unwrap();
			img[i] = val * image.iters;
		}

		image.data = img;

		Ok(image)
	}

	fn read_metadata(filename: &str) -> io::Result<(Self, String)>
	{
		let mut file = try!(File::open(filename));

		let mut content = String::new();
		try!(file.read_to_string(&mut content));

		let iters: usize;
		let size_x: usize;
		let size_y: usize;
		let max_val: usize;

		{
			let mut split = content.split_whitespace();

			if let Some(val) = split.next() {
				if val != "P3" {
					return Err(Error::new(ErrorKind::InvalidData, "File does not contain 'P3' tag"));
				}
			}

			let (hash, number) = split.next().unwrap().split_at(1);

			iters = number.parse().unwrap();

			if hash != "#" {
				return Err(Error::new(ErrorKind::InvalidData, "File does not have required metadata"));
			}

			size_x = split.next().unwrap().parse().unwrap();
			size_y = split.next().unwrap().parse().unwrap();
			max_val = split.next().unwrap().parse().unwrap();

//			println!("debug: iters = {:?}, {:?}", hash, iters);
//			println!("debug: size_x = {:?}", size_x);
//			println!("debug: size_y = {:?}", size_y);
//			println!("debug: max_val = {:?}", max_val);
		}

		Ok((Image {
			iters: iters,
			size_x: size_x,
			size_y: size_y,
			max_val: max_val,
			data: Vec::default()
		}, content))
	}

	fn add(&mut self, filename: &str) -> io::Result<Self>
	{
		let (image, content) = try!(Self::read_metadata(filename));

		let mut split = content.split_whitespace();

		let _metadata = split.nth(4);

		for (i, word) in split.enumerate() {
			let val: usize = word.parse().unwrap();
			self.data[i] += val * image.iters;
		}

		self.iters += image.iters;

		Ok(image)
	}

	fn divide (&mut self)
	{
		for val in &mut self.data {
			*val /= self.iters;
		}
	}

	fn save(&self, filename: &str) -> io::Result<()>
	{
		let mut file = try!(File::create(filename));

		let mut res = String::with_capacity(self.size_x * self.size_y * 3 * 4);

		res.push_str("P3\n");
		res.push_str(&format!("#{}\n", self.iters));
		res.push_str(&format!("{} {} {}\n", self.size_x, self.size_y, self.max_val));

		for val in &self.data {
			res.push_str(&format!("{} ", val));
		}

		try!(file.write_all(res.as_bytes()));

		Ok(())
	}
}

