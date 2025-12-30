use clap::{Parser, ValueEnum};
use std::io;
use std::io::BufRead;
use std::time::Duration;
use crate::filter::manager;

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
enum Command {
    Filter,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        value_name = "COMMAND"
    )]
    command: Command,

    /// Max age in seconds for a filter before pruning
    #[arg(
        long,
        default_value_t = 60*5,
    )]
    max_age: u64,
}

mod filter;

fn main() -> io::Result<()>{
    let args = Args::parse();

    match args.command {
        Command::Filter => {
            let mut manager = manager::Manager::new();

            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buf = String::new();
            let mut iter = 0;

            while handle.read_line(&mut buf)? > 0 {
                let line = buf.trim().to_string();
                buf.clear();

                if line.is_empty() {
                    continue;
                }

                if manager.insert(&line) {
                    println!("NEW:\t{}", line);
                } else {
                    println!("MATCH:\t{} (Est. FPR: {:.4}%)", line, manager.fpr() * 100.0);
                }

                iter += 1;

                if iter % 1000 == 0 {
                    manager.prune(Duration::from_secs(args.max_age));
                }
            }

        }
    }

    Ok(())
}
