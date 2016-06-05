
extern crate netpbm;

use netpbm::Image;

use std::env;
use std::io;

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
        io::ErrorKind::InvalidData => println!("warn: file '{}' is not a netpbm file", arg),
        _ => println!("warn: file '{}' does not exist", arg),
    }
}
