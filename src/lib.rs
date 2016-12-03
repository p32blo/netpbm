//! A library for `NetPBM` files. _**`Warning: This is an highly experimental crate`**_
//!
//! Reading, Writing and Merging of `.ppm` files.
//!
//! # Example
//!
//! ```
//! use netpbm::Image;
//!
//! // Create an empty image
//! let new = Image::new();
//! // Writing of an empty image
//! new.save("output.pfm").unwrap();
//!
//! // Reading an image
//! let mut image = Image::open("output.pfm").unwrap();
//! // Merging an image into the current image
//! image += Image::open("output.pfm").unwrap();
//! // Writing an image
//! image.save("output.pfm").unwrap();
//! ```

extern crate byteorder;

use byteorder::{NativeEndian, LittleEndian, BigEndian};
use byteorder::{WriteBytesExt, ReadBytesExt};

use std::fmt;
use std::ops::AddAssign;

use std::path::Path;
use std::fs::File;

use std::io;
use std::io::{Seek, SeekFrom};
use std::io::{Read, BufRead, BufReader};
use std::io::{Write, BufWriter};
use std::io::{Error, ErrorKind};


/// The main structure of this crate
#[derive(Debug)]
pub struct Image {
    /// Image iteration count
    iters: Option<usize>,
    /// Width of an image
    pub width: usize,
    /// Height of an image
    pub height: usize,
    /// The Maximum value for each pixel
    ratio: f32,
    /// Pixel data. If empty the image is considered empty
    data: Vec<f32>,
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ {} x {} ]", self.width, self.height)?;
        if let Some(it) = self.iters {
            write!(f, " iters = {}", it)?;
        }
        Ok(())
    }
}

impl AddAssign for Image {
    fn add_assign(&mut self, other: Image) {
        if self.is_empty() {
            *self = other;
        } else {
            let self_iters= self.iters.unwrap_or(1);
            let other_iters = other.iters.unwrap_or(1);

            for (data, &val) in self.data.iter_mut().zip(other.data.iter()) {
                *data *= self_iters as f32;
                *data += val * other_iters as f32;
            }

            self.iters = Some(self_iters + other_iters);
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

    /// Return the size of the image
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    /// Check if image is little endian
    fn is_le(&self) -> bool {
        self.ratio.is_sign_negative()
    }

    /// Check if image is little endian
    fn is_be(&self) -> bool {
        self.ratio.is_sign_positive()
    }

    fn load_metadata<R: BufRead + Seek>(&mut self, content: &mut R) -> io::Result<()> {
        let mut it = String::new();
        content.read_line(&mut it)?;


        self.iters = if it.starts_with("#>") {
            Some(it.split_whitespace().nth(1).unwrap().parse().expect("Metadata is missing"))
        } else {
            content.seek(SeekFrom::Start(0))?;
            None
        };

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

        self.width = lines.next().unwrap().parse().expect("Metadata is missing");
        self.height = lines.next().unwrap().parse().expect("Metadata is missing");
        self.ratio = lines.next().unwrap().parse().expect("Metadata is missing");

        Ok(())
    }

    fn load_data<R: Read>(&mut self, f: &mut R) -> io::Result<()> {

        self.data.clear();

        let img_rgb_size = self.size() * 3;
        // reserver image rgb size
        self.data.reserve_exact(img_rgb_size);

        // if data is little endian
        if self.is_le() {
            while let Ok(val) = f.read_f32::<LittleEndian>() {
                self.data.push(val)
            }
        } else {
            while let Ok(val) = f.read_f32::<BigEndian>() {
                self.data.push(val)
            }
        }

        Ok(())
    }

    /// Load the contents of a file to an `Image`
    ///
    /// - The values of a loaded image are multiplied
    /// by its number of iterations
    pub fn open<P: AsRef<Path>>(filename: P) -> io::Result<Self> {

        let mut image = Image::default();

        let mut f = BufReader::new(File::open(filename)?);
        image.load_metadata(&mut f)?;
        image.load_data(&mut f)?;

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

    fn store_metadata<W: Write>(&self, handle: &mut W) -> io::Result<()> {

        if let Some(it) = self.iters {
            write!(handle, "#> {}\n", it)?;
        }

        let ratio = if cfg!(target_endian = "little") {
            -self.ratio.abs()
        } else {
            self.ratio.abs()
        };

        write!(handle, "PF\n{} {} {}\n", self.width, self.height, ratio)?;
        Ok(())
    }

    fn store_data<W: Write>(&self, handle: &mut W) -> io::Result<()> {
        for rgb in self.data.chunks(3) {
            handle.write_f32::<NativeEndian>(rgb[0])?;
            handle.write_f32::<NativeEndian>(rgb[1])?;
            handle.write_f32::<NativeEndian>(rgb[2])?;
        }
        Ok(())
    }

    /// Output a file for this `Image`
    ///
    /// - All values are devided by `self.iters` to mantain the values
    /// in the `0` to `self.ratio` range.
    pub fn save(&self, filename: &str) -> io::Result<()> {

        let mut file = BufWriter::new(File::create(filename)?);

        self.store_metadata(&mut file)?;
        self.store_data(&mut file)?;

        Ok(())
    }

    /// Calculate luminance from `RGB` acording to `Photometric/digital ITU BT.709`
    fn luminance(rgb: &[f32]) -> f32 {
        0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
    }


    /// Calculate the RMSE of an image in relation to a ref Image
    pub fn rmse(&self, img: &Image) -> f32 {

        let mut mse: f32 = 0.0;
        let mut max_r: f32 = -1.0;

        for (rgb_img, rgb_ref) in self.data.chunks(3).zip(img.data.chunks(3)) {

            let yi = Self::luminance(rgb_img);
            let yr = Self::luminance(rgb_ref);

            let sqdiff = (yi - yr).powf(2.0);

            mse += sqdiff;
            max_r = max_r.max(yr);
        }
        mse /= self.size() as f32;

        let rmse = mse.sqrt();
        rmse
    }
}
