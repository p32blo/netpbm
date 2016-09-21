//! A library for `NetPBM` files. _**`Warning: This is an highly experimental crate`**_
//!
//! Reading, Writing and Merging of `.ppm` files.
//!
//! # Example
//!
//! ```
//! extern crate netpbm;
//! use netpbm::Image;
//!
//! fn main() {
//!     // Create an empty image
//!     let new = Image::new();
//!     // Writing of an empty image
//!     new.save("output.ppm").unwrap();
//!
//!     // Reading an image
//!     let mut image = Image::open("output.ppm").unwrap();
//!     // Merging an image into the current image
//!     let res = image.add("output.ppm").unwrap();
//!     // Writing an image
//!     res.save("output.ppm").unwrap();
//! }
//! ```

use std::fs::File;

use std::io;
use std::io::{Read, Write, Error, ErrorKind};

/// The main structure of this crate
#[derive(Default)]
pub struct Image {
    /// Image iteration count
    pub iters: usize,
    /// Width of an image
    pub width: usize,
    /// Height of an image
    pub height: usize,
    /// The Maximum value for each pixel
    pub max_val: usize,
    /// Pixel data. If empty the image is considered empty
    pub data: Vec<u32>,
}


impl Image {
    /// Generate an empty `Image`
    pub fn new() -> Self {
        Image::default()
    }

    /// Test if `Image` is empty
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

    /// Load the contents of a file to an `Image`
    ///
    /// - The values of a loaded image are multiplied
    /// by its number of iterations
    pub fn open(filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let mut image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        let img_size = image.width * image.height;
        let img_rgb_size = img_size * 3;

        image.data.reserve_exact(img_rgb_size);

        for word in split {
            let val: u32 = word.parse().unwrap();
            image.data.push(val * image.iters as u32);
        }

        Ok(image)
    }

    /// Accumulate the contents of a file to the `Image`
    ///
    /// - The values of a loaded image are multiplied
    /// by its number of iterations
    pub fn add(&mut self, filename: &str) -> io::Result<Self> {

        let content = try!(Self::get_file_content(filename));
        let image = try!(Self::load_metadata(&content));

        // skip metadata
        let split = content.split_whitespace().skip(5);

        for (word, item) in split.zip(&mut self.data) {
            let val: u32 = word.parse().unwrap();
            *item += val * image.iters as u32;
        }

        self.iters += image.iters;

        Ok(image)
    }

    /// Output a file for this `Image`
    ///
    /// - All values are devided by `self.iters` to mantain the values
    /// in the `0` to `self.max_val` range.
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

    fn y_val(val: (f32, f32, f32)) -> f32 {
        0.2126 * val.0 + 0.7152 * val.1 + 0.0722 * val.2
    }


    /// Calculate the RMSE of an image in relation to a ref Image
    pub fn rmse(&self, filename: &str) -> io::Result<f32> {
        let content = try!(Self::get_file_content(filename));
        // let image = try!(Self::load_metadata(&content));

        // skip metadata
        let mut split = content.split_whitespace().skip(5);

        let size = self.width * self.height;

        let mut iter = self.data.iter();

        let mut mse: f32 = 0.0;
        let mut max_r: f32 = -1.0;

        for _ in 0..self.height {
            for _ in 0..self.width {
                let img: (f32, f32, f32) =
                    (*iter.next().unwrap() as f32 / self.iters as f32 / self.max_val as f32,
                     *iter.next().unwrap() as f32 / self.iters as f32 / self.max_val as f32,
                     *iter.next().unwrap() as f32 / self.iters as f32 / self.max_val as f32);

                let r = split.next().unwrap().parse::<u32>().unwrap() as f32 / self.max_val as f32;
                let g = split.next().unwrap().parse::<u32>().unwrap() as f32 / self.max_val as f32;
                let b = split.next().unwrap().parse::<u32>().unwrap() as f32 / self.max_val as f32;

                let reference: (f32, f32, f32) = (r, g, b);

                let yi = Self::y_val(img);
                let yr = Self::y_val(reference);

                let sqdiff = (yi - yr).powf(2.0);

                mse += sqdiff;

                max_r = max_r.max(yr);
            }
        }

        mse /= size as f32;

        let rmse = mse.sqrt();

        Ok(rmse)
    }
}
