use crate::filter::filter_manager;
use crate::filter::filter_manager::FilterResult;
use crate::simulator::model::AdsbPacket;
use crate::tui::sim_app::SimApp;
use clap::{Parser, ValueEnum};
use std::{io, thread};
use std::io::BufRead;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
enum Command {
    Filter,
    Simulate,
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

    #[arg(
        short,
        long
    )]
    debug: bool,
}

mod filter;
mod simulator;
mod tui;

fn process_adsb_stream<F: FnMut(AdsbPacket)>(mut action: F) -> io::Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();

    while handle.read_line(&mut buf)? > 0 {
        if let Ok(packet) = serde_json::from_str::<AdsbPacket>(&buf) {
            action(packet);
        }

        buf.clear();
    }

    Ok(())
}

fn run_filter(args: Args) -> io::Result<()> {
    let mut filter_manager = filter_manager::FilterManager::new();
    let mut last_prune = Instant::now();
    let prune_interval = Duration::from_secs(5);

    process_adsb_stream(|packet| {
        if filter_manager.insert(&packet.id) == FilterResult::Pending {
            println!("NEW:\t{}", &packet.id);
        } else {
            println!("MATCH:\t{} (Est. FPR: {:.4}%)", &packet.id, filter_manager.fpr() * 100.0);
        }

        if Instant::now().duration_since(last_prune) > prune_interval {
            filter_manager.prune(
                Duration::from_secs(args.max_age)
            );

            last_prune = Instant::now();
        }
    })
}

fn run_simulation(args: Args) -> io::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let _ = process_adsb_stream(|packet| {
            if tx.send(packet).is_err() {
                return;
            }
        });
    });

    let mut app = SimApp::new(args, rx);
    app.run()
}

fn main() -> io::Result<()>{
    let args = Args::parse();

    match args.command {
        Command::Filter => run_filter(args),
        Command::Simulate => run_simulation(args),
    }
}
