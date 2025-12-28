use clap::Parser;
use std::io;
use std::io::BufRead;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    /// Max age in seconds for a filter before pruning
    #[arg(
        long,
        default_value_t = 60*5,
    )]
    max_age: u64,
}

mod bloom_filter;

fn main() -> io::Result<()>{
    let args = Args::parse();

    let mut sbf = bloom_filter::ScalableBloomFilter::new();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();
    let mut iter = 0;

    while handle.read_line(&mut buf)? > 0 {
        let line = buf.trim();
        if line.is_empty() {
            buf.clear();
            continue;
        }

        if sbf.contains(&line) {
            println!("MATCH:\t{} (Est. FPR: {:.4}%)", line, sbf.fpr() * 100.0);
        } else {
            sbf.insert(&line);
            println!("NEW:\t{}", line);
        }

        buf.clear();

        iter += 1;

        if iter % 1000 == 0 {
            sbf.prune(Duration::from_secs(args.max_age));

        }
    }

    Ok(())
}
