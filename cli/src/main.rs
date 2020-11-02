use relative_lempel_ziv::RelativeLempelZiv;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Display, PathBuf};
use std::time::{Duration, Instant};

// CLI tools
use anyhow::{Context, Result};
use console::style;
use indicatif::HumanBytes;
use structopt::StructOpt;

// Todo: Other arguments, like outputting the encoded data
// Which probably needs some form of serialization
// Docs @ https://docs.rs/structopt/0.3.20/structopt/
#[derive(StructOpt)]
struct CliInput {
    /// The path to the file (or directory if the is-dir flag is set) to compress data from
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Flag specifying whether the path is a directory
    #[structopt(short = "d", long)]
    is_dir: bool,

    /// INCOMPLETE: Output compressed data to file (needs serde Serialize trait first)
    #[structopt(short = "o", long = "output")]
    _output: Option<PathBuf>,
}

// Example input: "../test_data/dna.50MB"
fn main() -> Result<()> {
    let args = CliInput::from_args();

    let strings: Vec<String>;
    let total_size;

    if !args.is_dir {
        let file = File::open(&args.path)
            .with_context(|| format!("Could not read file `{}`", args.path.display()))?;
        let file_metadata = file.metadata();

        let buf_reader = BufReader::new(file);
        strings = buf_reader.lines().map(|l| l.unwrap()).collect();

        // Uses the file's metadata for the file size if it exists, otherwise it has to
        // calculate this using the len of every line.
        total_size = match file_metadata {
            Ok(metadata) => metadata.len(),
            Err(_) => strings.iter().fold(0, |acc, l| acc + l.len() as u64),
        };
    } else {
        let dir = fs::read_dir(&args.path)
            .with_context(|| format!("Could not read directory `{}`", args.path.display()))?;

        let mut tmp_strings = vec![];
        let mut size = 0;
        for dir_entry in dir {
            let path = dir_entry?.path();
            let file = File::open(&path)
                .with_context(|| format!("Could not read file `{}`", path.display()))?;
            let file_metadata = file.metadata();
            let file_string = fs::read_to_string(&path)?.replace(&['\n', '\r'][..], "");

            size += match file_metadata {
                Ok(metadata) => metadata.len(),
                Err(_) => file_string.len() as u64,
            };
            tmp_strings.push(file_string);
        }

        strings = tmp_strings;
        total_size = size;
    }

    // Only times the time it takes to encode the data
    let stopwatch = Instant::now();
    let encoded = RelativeLempelZiv::<u32>::encode(&strings);
    let elapsed_time = stopwatch.elapsed();

    let memory_size = encoded.memory_footprint();

    print_compression_data(args.path.display(), memory_size, total_size, elapsed_time);

    let stopwatch = Instant::now();
    let _ = encoded.decode();
    let decompressed_time = stopwatch.elapsed();

    print_decompression_timme(decompressed_time);

    Ok(())
}

fn print_compression_data(path: Display, memory: (usize, usize), raw_size: u64, time: Duration) {
    let (ref_data_size, data_size) = memory;
    let compressed_size = ref_data_size + data_size;
    let compression_rate = (ref_data_size + data_size) as f64 / raw_size as f64;
    let styled_compression_rate = {
        let style = style(compression_rate);
        match compression_rate {
            c if c > 1.0 => style.red(),
            c if c < 1.0 => style.green(),
            _ => style,
        }
    };

    println!(
        "Compression rate of `{}`: {:.2} ({} compressed / {} raw), taking {:?}",
        path,
        styled_compression_rate,
        HumanBytes(compressed_size as u64),
        HumanBytes(raw_size as u64),
        time
    );
    println!("Reference sequence: {}", HumanBytes(ref_data_size as u64));
    println!("Data size: {}", HumanBytes(data_size as u64));
}

fn print_decompression_timme(time: Duration) {
    println!("Decompression time took {:?}", time);
}
