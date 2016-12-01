//! Example project for the `netpbm` crate
//!
//! `rmse` is an example binary that uses the `netpbm`
//! crate to provide command line error comparition of `.ppm` files
//!
//! # Example
//!
//! To check the RMSE between two `.ppm` images, just use:
//!
//! ```sh
//! $ rmse REF.ppm IMG.ppm
//! ```

extern crate netpbm;
extern crate getopts;

use netpbm::Image;

use std::env;
use std::io;

fn main() {

    // Argument Parsing

    let mut opts = getopts::Options::new();

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

    // Image Loading

    let args = matches.free;

    if let 2 = args.len() {
        let img = match Image::open(&args[0]) {
            Ok(img) => img,
            Err(e) => {
                handle_error(e, &args[0]);
                return;
            }
        };

        match img.rmse(&args[1]) {
            Ok(val) => println!("RMSE: {}", val),
            Err(e) => handle_error(e, &args[1]),
        }
    } else {
        println!("error: Wrong number or arguments!");
    }
}

fn help() {
    println!(concat!("Usage: rmse [-v] [-h] file [file ...]\n",
                     "\n",
                     "Options:\n",
                     "    -v, --version       Show app version\n",
                     "    -h, --help          Show this help menu\n"));
}

fn handle_error(e: io::Error, arg: &str) {
    match e.kind() {
        io::ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}