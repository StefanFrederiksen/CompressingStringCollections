use relative_lempel_ziv::RelativeLempelZiv;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

// CLI tools
use anyhow::{Context, Result};
use console::style;
use indicatif::{HumanBytes, HumanDuration};
use structopt::StructOpt;

// Todo: Other arguments, like outputting the encoded data
// Which probably needs some form of serialization
// Docs @ https://docs.rs/structopt/0.3.20/structopt/
#[derive(StructOpt)]
struct CliInput {
    // The path of the file to compress data from
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    // Todo:
    // Output compressed data to file (needs serde Serialize trait first)
    #[structopt(short = "o", long = "output")]
    _output: Option<PathBuf>,
}

// Example input: "../test_data/dna.50MB"
fn main() -> Result<()> {
    let args = CliInput::from_args();

    let file = File::open(&args.path)
        .with_context(|| format!("Could not read file `{}`", args.path.display()))?;
    let file_metadata = file.metadata();

    let buf_reader = BufReader::new(file);
    let lines: Vec<_> = buf_reader.lines().map(|l| l.unwrap()).collect();

    // Uses the file's metadata for the file size if it exists, otherwise it has to
    // calculate this using the len of every line.
    let file_size = match file_metadata {
        Ok(metadata) => metadata.len(),
        Err(_) => lines.iter().fold(0, |acc, l| acc + l.len() as u64),
    };

    // Only times the time it takes to encode the data
    let stopwatch = Instant::now();
    let encoded = RelativeLempelZiv::<u32>::encode(&lines);
    let elapsed_time = stopwatch.elapsed();

    let memory_size = encoded.memory_footprint();

    let compression_rate = memory_size as f64 / file_size as f64;
    let styled_compression_rate = {
        let style = style(compression_rate);
        match compression_rate {
            c if c > 1.0 => style.red(),
            c if c < 1.0 => style.green(),
            _ => style,
        }
    };

    println!(
        "Compression rate of `{}`: {:.2} ({} compressed / {} raw), taking {}",
        args.path.display(),
        styled_compression_rate,
        HumanBytes(memory_size as u64),
        HumanBytes(file_size as u64),
        HumanDuration(elapsed_time)
    );

    let stopwatch = Instant::now();
    let _ = encoded.decode();
    let decompressed_time = stopwatch.elapsed();

    println!(
        "Decompression time took {:?}",
        // HumanDuration(decompressed_time)
        decompressed_time
    );

    Ok(())
}
