//!
//! Exemple project for the `netpbm` crate
//!

extern crate netpbm;
extern crate getopts;

use netpbm::Image;

use std::env;
use std::io;

fn main() {

    // Argument Parsing

    let mut opts = getopts::Options::new();

    let mut args = env::args();
    let prog = &mut args.next().unwrap();

    opts.optopt("o", "output", "Set custom Output filename", "FILE");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(args.collect::<Vec<String>>()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            return;
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(&opts.short_usage(prog)));
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
    }
}

fn handle_error(e: io::Error, arg: &str) {
    match e.kind() {
        io::ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}
