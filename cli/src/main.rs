#![allow(dead_code, unused_imports)]

use relative_lempel_ziv::{EncodePart, RelativeLempelZiv};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem;

fn main() -> std::io::Result<()> {
    // println!("{:?}", contents);
    let mut contents = String::new();
    let file = File::open("../test_data/dna.50MB")?;
    let mut buf_reader = BufReader::new(file);
    buf_reader.read_to_string(&mut contents)?;
    println!("Size of file: {}", contents.len());
    let lines = contents.split('\n').collect::<Vec<_>>();

    let encoded = RelativeLempelZiv::<u32>::encode(&lines);
    // assert_eq!(lines, encoded.decode());
    let encoded_size = mem::size_of_val(&encoded);
    println!("{}", encoded_size);

    println!("Total size: {}", encoded.memory_footprint());

    // println!("{}", mem::size_of::<EncodePart>());

    Ok(())
}
