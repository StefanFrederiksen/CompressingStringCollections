#[macro_use]
extern crate log;

use relative_lempel_ziv::memory_usage::MemoryUsage;
use relative_lempel_ziv::RelativeLempelZiv;
use simplelog::*;
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

    /// The reference string strategy. Only valid values are the following, the rest will crash
    /// 1. A single reference is chosen, either randomly or from the `i` input
    /// 2. The reference merge strategy, which recursively tries to find a better and better reference string
    #[structopt(short, long, default_value = "1")]
    strategy: usize,

    /// Flag specifying whether the path is a directory
    #[structopt(short = "d", long)]
    is_dir: bool,

    /// INCOMPLETE: Output compressed data to file (needs serde Serialize trait first)
    #[structopt(short = "o", long = "output")]
    _output: Option<PathBuf>,

    /// If you want to manually tell the cli which reference strings to take
    #[structopt(short = "i", default_value = "0")]
    i: Vec<usize>,

    /// The characters that the reference string must include, is appended at the end of the reference string to ensure all chars are present.
    #[structopt(short, long)]
    chars: String,
}

// Example input: "../test_data/dna.50MB"
fn main() -> Result<()> {
    let args = CliInput::from_args();
    init_logging();

    info!("Using {:?} as reference strings", &args.i);

    let strings: Vec<(String, String)>;
    let total_size;

    if !args.is_dir {
        trace!("Loading single file into memory");

        let file = File::open(&args.path)
            .with_context(|| format!("Could not read file `{}`", args.path.display()))?;
        let file_metadata = file.metadata();

        let buf_reader = BufReader::new(file);
        strings = buf_reader
            .lines()
            .map(|l| (l.unwrap(), String::new()))
            .collect();

        // Uses the file's metadata for the file size if it exists, otherwise it has to
        // calculate this using the len of every line.
        total_size = match file_metadata {
            Ok(metadata) => metadata.len(),
            Err(_) => strings.iter().fold(0, |acc, l| acc + l.0.len() as u64),
        };
    } else {
        trace!("Loading directory files into memory");

        let dir = fs::read_dir(&args.path)
            .with_context(|| format!("Could not read directory `{}`", args.path.display()))?;

        let mut tmp_strings = vec![];
        let mut size = 0;
        for dir_entry in dir {
            let path = dir_entry?.path();
            let file = File::open(&path)
                .with_context(|| format!("Could not read file `{}`", path.display()))?;
            let file_name = path.file_name().unwrap();
            let file_metadata = file.metadata();
            let file_string = fs::read_to_string(&path)?.replace(&['\n', '\r'][..], "");

            size += match file_metadata {
                Ok(metadata) => metadata.len(),
                Err(_) => file_string.len() as u64,
            };
            tmp_strings.push((file_string, String::from(file_name.to_str().unwrap())));
        }

        strings = tmp_strings;
        total_size = size;
    }

    let chars = if args.chars.is_empty() {
        None
    } else {
        Some(args.chars)
    };
    let stopwatch = Instant::now();
    let encoded = match args.strategy {
        1 => {
            let s = strings.iter().map(|t| &t.0).collect::<Vec<_>>();
            RelativeLempelZiv::<u32>::encode(&s, Some(args.i), chars)
        }
        2 => RelativeLempelZiv::<u32>::encode_reference_merge(&strings, chars),
        _ => panic!("Invalid strategy input"),
    };
    let elapsed_time = stopwatch.elapsed();

    let memory_size = encoded.memory_footprint(Some(total_size as usize));

    print_compression_data(args.path.display(), memory_size, elapsed_time);

    let stopwatch = Instant::now();
    // The `let _` is needed for the compiler to not throw
    // away this computation since it is not "used"
    let _ = encoded.decode();
    let decompressed_time = stopwatch.elapsed();
    print_decompression_time(decompressed_time);

    // info!("Analysis data size: {}", analysis.list.len());
    // let mut file = File::create("analysis.txt")?;
    // file.write_all(format!("{}\n", analysis).as_bytes())?;

    Ok(())
}

fn print_compression_data(path: Display, memory: MemoryUsage, time: Duration) {
    let compressed_size = memory.compressed_size();
    let compression_rate = memory.compression_rate().unwrap();
    let styled_compression_rate = {
        let style = style(compression_rate);
        match compression_rate {
            c if c > 1.0 => style.red(),
            c if c < 1.0 => style.green(),
            _ => style,
        }
    };

    let compressed_size_no_ra = memory.compression_rate_without_ra().unwrap();
    let styled_compression_rate_no_ra = {
        let style = style(compressed_size_no_ra);
        match compressed_size_no_ra {
            c if c > 1.0 => style.red(),
            c if c < 1.0 => style.green(),
            _ => style,
        }
    };

    info!(
        "Compression rate of `{}`: {:.2} ({:.2}) ({} compressed / {} raw), taking {:?}",
        path,
        styled_compression_rate,
        styled_compression_rate_no_ra,
        HumanBytes(compressed_size as u64),
        HumanBytes((memory.raw_size().unwrap()) as u64),
        time
    );
    trace!(
        "Reference sequence: {}",
        HumanBytes(memory.reference_size() as u64)
    );
    trace!(
        "Factorization size: {}",
        HumanBytes(memory.factorizations_size() as u64)
    );
}

fn print_decompression_time(time: Duration) {
    info!("Decompression time took {:?}", time);
}

fn init_logging() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("rlz.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("Loggers initialized.");
}
