//! Example project for the `netpbm` crate
//!
//! `merge` is an example binary that uses the `netpbm`
//! crate to provide easy command line merging of `.ppm` files
//!
//! # Example
//!
//! When wanting to merge two or more files, just use:
//!
//! ```sh
//! $ merge CPU.ppm GPU.ppm
//! reading: CPU.ppm [ 1024 x 768 ] iters = 285
//! reading: GPU.ppm [ 1024 x 768 ] iters = 332
//! writing: output.ppm [ 1024 x 768 ] iters = 617
//! ```
//!
//! By default the output file will be named `output.ppm`.
//! To change the output name the `-o` option can be given:
//!
//! ```sh
//! $ merge CPU.ppm GPU.ppm -o result.ppm
//! reading: CPU.ppm [ 1024 x 768 ] iters = 285
//! reading: GPU.ppm [ 1024 x 768 ] iters = 332
//! writing: result.ppm [ 1024 x 768 ] iters = 617
//! ```
//!

extern crate netpbm;
extern crate getopts;

use netpbm::Image;

use std::env;
use std::io;

fn main() {

    // Argument Parsing

    let mut opts = getopts::Options::new();

    opts.optopt("o", "output", "Set custom Output filename", "FILE");
    opts.optflag("h", "help", "Print this help menu");

    let args = env::args().skip(1);

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            return;
        }
    };

    if matches.opt_present("h") {
        help();
        return;
    }

    let output = match matches.opt_str("o") {
        Some(filename) => filename,
        None => "output.ppm".into(),
    };

    let mut files = matches.free.iter();

    // Image Loading

    let mut image = Image::new();

    for arg in &mut files {
        match Image::open(arg) {
            Ok(img) => {
                image = img;
                println!("reading: {} [ {} x {} ] iters = {}",
                         arg,
                         image.width,
                         image.height,
                         image.iters);
                break;
            }
            Err(e) => handle_error(e, arg),
        }
    }

    for arg in &mut files {
        match image.add(arg) {
            Ok(img) => {
                println!("reading: {} [ {} x {} ] iters = {}",
                         arg,
                         img.width,
                         img.height,
                         img.iters);
            }
            Err(e) => handle_error(e, arg),
        }
    }

    if !image.is_empty() {
        println!("writing: {} [ {} x {} ] iters = {}",
                 output,
                 image.width,
                 image.height,
                 image.iters);

        image.save(&output).unwrap();
    } else {
        help();
        return;
    }
}

fn help() {
    println!("Usage: merge [-o FILE] [-h] file [file ...]");
    println!("");
    println!("Options:");
    println!("    -o, --output FILE   Set custom Output filename");
    println!("    -h, --help          Print this help menu");

    // println!("{}", opts.usage(&opts.short_usage(prog)));
}

fn handle_error(e: io::Error, arg: &str) {
    match e.kind() {
        io::ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}
