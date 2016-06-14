
use std::fs::File;

use std::io;
use std::io::{Read, Write, Error, ErrorKind};


#[derive(Default)]
pub struct Image {
    pub iters: usize,
    pub width: usize,
    pub height: usize,
    pub max_val: usize,
    pub data: Vec<u32>,
}


impl Image {
    pub fn new() -> Self {
        Image::default()
    }

    pub fn is_empty(&self) -> bool {
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
        let width: usize;
        let height: usize;
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

            width = split.next().unwrap().parse().unwrap();
            height = split.next().unwrap().parse().unwrap();
            max_val = split.next().unwrap().parse().unwrap();

            // println!("debug: iters = {:?}, {:?}", hash, iters);
            // println!("debug: width = {:?}", width);
            // println!("debug: height = {:?}", height);
            // println!("debug: max_val = {:?}", max_val);
        }

        Ok(Image {
            iters: iters,
            width: width,
            height: height,
            max_val: max_val,
            data: Vec::default(),
        })
    }

    pub fn open(filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let mut image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        let img_size = image.width * image.height;
        let img_rgb_size = img_size * 3;

        image.data.reserve(img_rgb_size);

        for word in split {
            let val: u32 = word.parse().unwrap();
            image.data.push(val * image.iters as u32);
        }

        Ok(image)
    }

    pub fn add(&mut self, filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        for (word, item) in split.zip(self.data.iter_mut()) {
            let val: u32 = word.parse().unwrap();
            *item += val * image.iters as u32;
        }

        self.iters += image.iters;

        Ok(image)
    }

    pub fn save(&self, filename: &str) -> io::Result<()> {

        let mut file = try!(File::create(filename));

        let mut res = String::with_capacity(self.width * self.height * 3 * 4);

        res.push_str("P3\n");
        res.push_str(&format!("#{}\n", self.iters));
        res.push_str(&format!("{} {} {}\n", self.width, self.height, self.max_val));

        let mut iter = self.data.iter();

        for _ in 0..self.height {
            for _ in 0..self.width {
                let (r, g, b) = (iter.next().unwrap() / self.iters as u32,
                                 iter.next().unwrap() / self.iters as u32,
                                 iter.next().unwrap() / self.iters as u32);

                res.push_str(&format!("{} {} {} ", r, g, b));
            }
            res.pop();
            res.push('\n');
        }

        try!(file.write_all(res.as_bytes()));

        Ok(())
    }
}
