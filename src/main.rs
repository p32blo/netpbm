
use std::env;

use std::fs::File;

use std::io;
use std::io::{Read, Write, Error, ErrorKind};


#[derive(Default)]
struct Image {
    iters: usize,
    size_x: usize,
    size_y: usize,
    max_val: usize,
    data: Vec<u32>,
}

fn main() {

    let mut args = env::args().skip(1);

    let mut image = Image::new();

    for arg in &mut args {
        match Image::open(&arg) {
            Ok(img) => {
                image = img;
                println!("reading: {} [ {} x {} ] iters = {}",
                         arg,
                         image.size_x,
                         image.size_y,
                         image.iters);
                break;
            }
            Err(e) => handle_error(e, &arg),
        }
    }

    for arg in &mut args {
        match image.add(&arg) {
            Ok(img) => {
                println!("reading: {} [ {} x {} ] iters = {}",
                         arg,
                         img.size_x,
                         img.size_y,
                         img.iters);
            }
            Err(e) => handle_error(e, &arg),
        }
    }

    if !image.is_empty() {

        let out = "output.ppm";

        println!("writing: {} [ {} x {} ] iters = {}",
                 out,
                 image.size_x,
                 image.size_y,
                 image.iters);

        image.save(out).unwrap();
    }
}

fn handle_error(e: io::Error, arg: &str) {
    match e.kind() {
        ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}

impl Image {
    fn new() -> Self {
        Image::default()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn get_file_content(filename: &str) -> io::Result<String> {

        let mut file = try!(File::open(filename));

        let mut content = String::new();
        try!(file.read_to_string(&mut content));

        Ok(content)
    }

    fn load_metadata(content: &str) -> io::Result<Self> {

        let iters: usize;
        let size_x: usize;
        let size_y: usize;
        let max_val: usize;

        {
            let mut split = content.split_whitespace();

            if let Some(val) = split.next() {
                if val != "P3" {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "File does not contain 'P3' tag"));
                }
            }

            let (hash, number) = split.next().unwrap().split_at(1);

            iters = number.parse().unwrap();

            if hash != "#" {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "File does not have required metadata"));
            }

            size_x = split.next().unwrap().parse().unwrap();
            size_y = split.next().unwrap().parse().unwrap();
            max_val = split.next().unwrap().parse().unwrap();

            // 			println!("debug: iters = {:?}, {:?}", hash, iters);
            // 			println!("debug: size_x = {:?}", size_x);
            // 			println!("debug: size_y = {:?}", size_y);
            // 			println!("debug: max_val = {:?}", max_val);
        }

        Ok(Image {
            iters: iters,
            size_x: size_x,
            size_y: size_y,
            max_val: max_val,
            data: Vec::default(),
        })
    }

    fn open(filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let mut image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        let img_size = image.size_x * image.size_y;
        let img_rgb_size = img_size * 3;

        let mut data = vec![0; img_rgb_size];

        for (i, word) in split.enumerate() {
            let val: u32 = word.parse().unwrap();
            data[i] = val * image.iters as u32;
        }

        image.data = data;

        Ok(image)
    }

    fn add(&mut self, filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        for (i, word) in split.enumerate() {
            let val: u32 = word.parse().unwrap();
            self.data[i] += val * image.iters as u32;
        }

        self.iters += image.iters;

        Ok(image)
    }

    fn save(&self, filename: &str) -> io::Result<()> {

        let mut file = try!(File::create(filename));

        let mut res = String::with_capacity(self.size_x * self.size_y * 3 * 4);

        res.push_str("P3\n");
        res.push_str(&format!("#{}\n", self.iters));
        res.push_str(&format!("{} {} {}\n", self.size_x, self.size_y, self.max_val));

        for val in &self.data {
            res.push_str(&format!("{} ", val / self.iters as u32));
        }

        try!(file.write_all(res.as_bytes()));

        Ok(())
    }
}
