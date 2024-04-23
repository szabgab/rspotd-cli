#![allow(warnings)]

use chrono::Local;
use clap::builder::PossibleValuesParser;
use clap::Parser;
use rspotd::{generate, generate_multiple, seed_to_des};
use serde_json::to_string_pretty;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::{path::Path, process::exit};
use std::writeln;

#[derive(Parser)]
#[clap(
    author = "Shea Zerda",
    version,
    about = "ARRIS/Commscope password-of-the-day generator"
)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short = 's',
        long = "seed",
        help = "String of 4-8 characters, used in password generation to mutate output"
    )]
    seed: Option<String>,

    #[arg(
        short = 'd',
        long = "date",
        conflicts_with = "range",
        help = "Generate a password for the given date"
    )]
    date: Option<String>,

    #[arg(
        short = 'D',
        long = "des",
        conflicts_with = "date",
        conflicts_with = "range",
        num_args = 0,
        help = "Output DES representation of seed"
    )]
    des: bool,

    #[arg(
        short = 'f',
        long = "format",
        value_parser = PossibleValuesParser::new(["json", "text"]),
        help="Password output format, either text or json"
    )]
    format: Option<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "Password or list will be written to given filename"
    )]
    output: Option<String>,

    #[arg(
        short = 'r',
        long = "range",
        conflicts_with = "date",
        num_args = 2, value_names = ["START", "END"],
        help="Generate a list of passwords given start and end dates"
    )]
    range: Option<Vec<String>>,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Print output to console even when writing to file"
    )]
    verbose: bool,
}

fn current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn main() {
    use rspotd::vals::DEFAULT_SEED;
    let args = Args::parse();
    let format;
    let seed;
    let output;
    let path;

    // determine output format
    if args.format.is_none() {
        format = "text";
    } else {
        format = args.format.as_ref().unwrap();
    }

    // determine output file, if any
    if args.output.is_none() {
        path = Path::new(".").to_path_buf();
        output = false;
    } else {
        let user_input = args.output.unwrap();
        path = Path::new(".").join(user_input.to_string());
        output = true;
    }

    // determine seed
    if args.seed.is_none() {
        seed = DEFAULT_SEED;
    } else {
        seed = args.seed.as_ref().unwrap().as_str();
    }

    // determine date or range
    if args.range.is_none() {
        let date: String;
        if args.date.is_none() {
            date = current_date();
        } else {
            date = args.date.as_ref().unwrap().to_string();
        }
        let result = generate(date.as_ref(), seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);
        } else {
            if output {
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .append(false)
                    .open(path)
                    .unwrap();

                let potd = format!("{}\n", result.as_ref().unwrap());
                file.write_all(potd.as_bytes());
                if args.verbose {
                    println!("{}\n", result.unwrap());
                    exit(0)
                }
            }
            println!("{}", result.as_ref().unwrap());
            exit(0);
        }
    } else {
        let range = args.range.unwrap();
        let begin = &range[0];
        let end = &range[1];
        let result = generate_multiple(&begin, &end, seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);
        } else {
            if output {
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(false)
                    .open(&path);
                // file doesn't exist or bad permissions
                if file.is_err() {
                    file = OpenOptions::new()
                        .write(true)
                        .append(false)
                        .create_new(true)
                        .open(&path);
                    // file cannot be created due to permissions
                    if file.is_err() {
                        println!("Unable to create file '{}' due to permissions.", path.display());
                        exit(1);
                    }
                }
                let mut writer = BufWriter::new(file.as_mut().unwrap());
                let potd = serde_json::to_string_pretty(result.as_ref().unwrap());
                if potd.is_err() {
                    println!("{}", potd.as_ref().unwrap_err());
                    exit(1)
                } else {
                    writer.write_all(potd.as_ref().unwrap().as_bytes());
                    writer.write_all("\n".as_bytes());
                    if args.verbose {
                        println!("{}", potd.unwrap());
                        exit(0);
                    }
                }
            }
        }
    }

    // TODO:
    // - implement format
    // - output to file
    // - verbose (print even when output to file)
    // - add date formatting
    //   - default format
}