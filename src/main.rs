#![recursion_limit = "1024"]

use std::process::exit;

extern crate clap;

extern crate memmap;
extern crate memchr;

extern crate jobsteal;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain!{}
}

mod count;
mod sort;
mod run_counts;
mod runner;

mod readers;
mod parsers;

#[cfg(test)]
mod tests;

fn main() {
    env_logger::init().unwrap();

    let args = clap::App::new("kmer-counter")
        .version("1.0")
        .author("Lee Bousfield <ljbousfield@gmail.com>")
        .about("Counts k-mers")
        .arg(clap::Arg::with_name("inputs")
             .required(true)
             .takes_value(true)
             .value_name("INPUT...")
             .help("The input FASTA files"))
        .arg(clap::Arg::with_name("join_methods")
             .multiple(true)
             .value_name("METHODS...")
             .possible_values(&["concat", "sort"])
             .help("The methods (from lowest level to highest level) used to join kmer lists together"))
        .arg(clap::Arg::with_name("kmer_len")
             .short("n")
             .long("kmer-length")
             .required(true)
             .takes_value(true)
             .value_name("LENGTH")
             .help("The length of generated k-mers"))
        .arg(clap::Arg::with_name("only_presence")
             .short("p")
             .long("only-presence")
             .help("If enabled, only outputs 1 instead of the count to the output file"))
        .arg(clap::Arg::with_name("min_count")
             .short("c")
             .long("min-count")
             .default_value("1")
             .help("The minimum count to be outputted"))
        .get_matches();

    let input = args.value_of("input").unwrap();

    let join_methods = args.values_of("join_methods").unwrap().map(|m| match m {
        "concat" => runner::JoinMethod::Concat,
        "sort" => runner::JoinMethod::Sort,
        method @ _ => {
            error!("Unknown join method {}", method);
            exit(1);
        }
    }).collect::<Vec<_>>();

    let kmer_len = args.value_of("kmer_len")
        .unwrap()
        .parse::<u8>()
        .unwrap_or_else(|_| {
            error!("Failed to parse k-mer length as a positive integer");
            exit(1);
        });
    if kmer_len == 0 {
        error!("Kmer length cannot be 0");
        exit(1);
    }
    if kmer_len > 32 {
        error!("The kmer length {} is invalid as there is a limit of 32",
               kmer_len);
        exit(1);
    }

    let min_count = args.value_of("min_count")
        .unwrap()
        .parse::<u16>()
        .unwrap_or_else(|_| {
            error!("Failed to parse minimum count as a positive integer");
            exit(1);
        });
    let only_presence = args.is_present("only_presence");

    let runner_opts = runner::Options {
        input: input.to_string(),
        kmer_len: kmer_len,
        min_count: min_count,
        only_presence: only_presence,
        join_methods: join_methods,
    };
    info!("Argument parsing complete");
    if let Err(ref e) = runner::run(runner_opts) {
        error!("{}", e);

        for e in e.iter().skip(1) {
            error!("Caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            error!("Backtrace:\n{:?}", backtrace);
        }

        exit(2);
    }
}
