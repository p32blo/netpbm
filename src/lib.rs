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

extern crate byteorder;

use byteorder::{NativeEndian, LittleEndian, BigEndian};
use byteorder::{WriteBytesExt, ReadBytesExt};

use std::io;
use std::io::{Read, Write};
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};

use std::fs::File;
use std::path::Path;

use std::ops::AddAssign;


/// The main structure of this crate
pub struct Image {
    /// Image iteration count
    pub iters: usize,
    /// Width of an image
    pub width: usize,
    /// Height of an image
    pub height: usize,
    /// The Maximum value for each pixel
    pub ratio: f32,
    /// Pixel data. If empty the image is considered empty
    pub data: Vec<f32>,
}

impl AddAssign for Image {
    fn add_assign(&mut self, other: Image) {

        if self.is_empty() {
            *self = other;
        } else {
            for (data, &val) in self.data.iter_mut().zip(other.data.iter()) {
                *data += val;
            }
        }
    }
}

impl Default for Image {
    fn default() -> Image {
        Image {
            ratio: if cfg!(target_endian = "little") {
                -1.0
            } else {
                1.0
            },
            iters: Default::default(),
            height: Default::default(),
            width: Default::default(),
            data: Default::default(),
        }
    }
}

impl Image {
    /// Generate an empty `Image`
    pub fn new() -> Self {
        Self::default()
    }

    /// Test if `Image` is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if image is little endian
    fn is_le(&self) -> bool {
        self.ratio.is_sign_negative()
    }

    /// Check if image is little endian
    fn is_be(&self) -> bool {
        self.ratio.is_sign_positive()
    }

    fn get_file_content(filename: &str) -> io::Result<String> {

        let mut file = File::open(filename)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    fn load_metadata<R: BufRead>(content: R) -> io::Result<Self> {

        let mut img = Self::default();

        let mut lines = content.lines()
            .map(|l| l.unwrap())
            .filter(|l| !l.starts_with("#"))
            .flat_map(|line| {
                line.split_whitespace()
                    .map(|w| w.to_string())
                    .collect::<Vec<_>>()
                    .into_iter()
            });

        if let Some(val) = lines.next() {
            if val != "PF" {
                return Err(Error::new(ErrorKind::InvalidData, "File does not contain 'PF' tag"));
            }
        }

        img.width = lines.next().unwrap().parse().expect("Metadata is missing");
        img.height = lines.next().unwrap().parse().expect("Metadata is missing");
        img.ratio = lines.next().unwrap().parse().expect("Metadata is missing");

        Ok(img)
    }

    /// Load the contents of a file to an `Image`
    ///
    /// - The values of a loaded image are multiplied
    /// by its number of iterations
    pub fn open<P: AsRef<Path>>(filename: P) -> io::Result<Self> {

        let mut f = BufReader::new(File::open(filename)?);
        let mut image = Self::load_metadata(f.by_ref())?;

        let img_size = image.width * image.height;
        let img_rgb_size = img_size * 3;

        image.data.reserve_exact(img_rgb_size);

        if image.is_le() {
            while let Ok(val) = f.read_f32::<LittleEndian>() {
                image.data.push(val)
            }
        } else {
            while let Ok(val) = f.read_f32::<BigEndian>() {
                image.data.push(val)
            }
        }

        Ok(image)
    }

    /// Accumulate the contents of a file to the `Image`
    ///
    /// - The values of a loaded image are multiplied
    /// by its number of iterations
    pub fn add<P: AsRef<Path>>(&mut self, filename: P) -> io::Result<()> {
        let img = Image::open(filename)?;

        *self += img;

        Ok(())
    }

    /// Output a file for this `Image`
    ///
    /// - All values are devided by `self.iters` to mantain the values
    /// in the `0` to `self.ratio` range.
    pub fn save(&self, filename: &str) -> io::Result<()> {

        let mut file = File::create(filename)?;

        let mut res = String::with_capacity(self.width * self.height * 3 * 4);

        res.push_str("PF\n");
        res.push_str(&format!("#{}\n", self.iters));
        res.push_str(&format!("{} {}\n", self.width, self.height));

        if cfg!(target_endian = "little") {
            res.push_str(&format!("{}\n", -self.ratio.abs()));
        } else {
            res.push_str(&format!("{}\n", self.ratio.abs()));
        }

        file.write_all(res.as_bytes())?;

        let mut buf: Vec<u8> = Vec::new();

        let mut iter = self.data.iter();

        for _ in 0..self.height {
            for _ in 0..self.width {
                let (r, g, b) = (iter.next().unwrap() / self.iters as f32,
                                 iter.next().unwrap() / self.iters as f32,
                                 iter.next().unwrap() / self.iters as f32);

                buf.write_f32::<NativeEndian>(r)?;
                buf.write_f32::<NativeEndian>(g)?;
                buf.write_f32::<NativeEndian>(b)?;
            }
        }

        file.write_all(&buf)?;

        Ok(())
    }

    fn y_val(rgb: &[f32]) -> f32 {
        0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
    }


    /// Calculate the RMSE of an image in relation to a ref Image
    pub fn rmse(&self, ref_img: &Image) -> io::Result<f32> {

        let size = self.width * self.height;

        let mut rgb = self.data.chunks(3).zip(ref_img.data.chunks(3));

        let mut mse: f32 = 0.0;
        let mut max_r: f32 = -1.0;

        for _ in 0..self.height {
            for _ in 0..self.width {

                let (img, reference) = rgb.next().unwrap();

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
