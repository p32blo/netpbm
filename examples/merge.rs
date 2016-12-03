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
    opts.optflag("v", "version", "Show app version");
    opts.optflag("h", "help", "Show this help menu");

    let args = env::args().skip(1);

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            return;
        }
    };

    if matches.opt_present("h") {
        // println!("{}", opts.usage(&opts.short_usage("merge")));
        help();
        return;
    }

    if matches.opt_present("v") {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }

    let output = matches.opt_str("o").unwrap_or("output.pfm".into());

    // Image Loading
    let mut image = Image::new();

    for arg in &matches.free {
        match Image::open(arg) {
            Ok(img) => {
                image += &img;
                println!("reading: {} {}", arg, img);
            }
            Err(e) => handle_error(e, arg),
        }
    }

    if !image.is_empty() {
        println!("writing: {} {}", output, image);
        image.save(&output).unwrap();
    } else {
        help();
        return;
    }
}

fn help() {
    println!(concat!("Usage: merge [-o FILE] [-v] [-h] file [file ...]\n",
                     "\n",
                     "Options:\n",
                     "    -o, --output FILE   Set custom Output filename\n",
                     "    -v, --version       Show app version\n",
                     "    -h, --help          Show this help menu\n"));
}

fn handle_error(e: io::Error, arg: &str) {
    match e.kind() {
        io::ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}
