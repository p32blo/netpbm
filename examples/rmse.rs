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
        match rmse(&args[0], &args[1]) {
            Ok(val) => println!("RMSE: {}", val),
            Err(message) => println!("{}", message),
        }

    } else {
        println!("error: Wrong number or arguments!");
    }
}

fn rmse(ref_img: &str, img: &str) -> Result<f32, String> {

    let ref_img = Image::open(ref_img).map_err(|e| handle_error(e, ref_img))?;
    let img = Image::open(img).map_err(|e| handle_error(e, img))?;

    Ok(ref_img.rmse(&img))
}

fn help() {
    println!(concat!("Usage: rmse [-v] [-h] file [file ...]\n",
                     "\n",
                     "Options:\n",
                     "    -v, --version       Show app version\n",
                     "    -h, --help          Show this help menu\n"));
}

fn handle_error(e: io::Error, arg: &str) -> String {
    match e.kind() {
        io::ErrorKind::InvalidData => {
            format!("warn: file '{}' is not a netpbm file", arg).to_string()
        }
        _ => format!("warn: file '{}' does not exist", arg).to_string(),
    }
}
