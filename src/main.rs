use clap::{Parser, ValueEnum};
use std::io;
use std::io::BufRead;
use std::sync::Arc;
use std::time::Duration;
use crate::filter::filter_manager;
use crate::simulator::model::AdsbPacket;
use crate::simulator::sim_manager;

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
}

mod filter;
mod simulator;

fn process_adsb_stream<F: FnMut(AdsbPacket)>(mut action: F) -> io::Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();

    while handle.read_line(&mut buf)? > 0 {
        let line = buf.trim();

        if line.is_empty() {
            continue;
        }

        if let Ok(packet) = serde_json::from_str::<AdsbPacket>(line) {
            action(packet);
        }

        buf.clear();
    }

    Ok(())
}

fn run_filter(args: Args) -> io::Result<()> {
    let mut filter_manager = filter_manager::FilterManager::new();
    let mut iter = 0;

    process_adsb_stream(|packet| {

        if filter_manager.insert(&packet.id) {
            println!("NEW:\t{}", &packet.id);
        } else {
            println!("MATCH:\t{} (Est. FPR: {:.4}%)", &packet.id, filter_manager.fpr() * 100.0);
        }

        iter += 1;

        if iter % 1000 == 0 {
            filter_manager.prune(Duration::from_secs(args.max_age));
        }
    })
}

fn run_simulation(args: Args) -> io::Result<()> {
    let mut filter_manager = filter_manager::FilterManager::new();
    let mut sim_manager = sim_manager::SimManager::new();
    let mut iter = 0;

    process_adsb_stream(|packet| {
        if !filter_manager.insert(&packet.id) {
            let name = packet.callsign.unwrap_or(packet.id);
            sim_manager.handle_update(
                Arc::from(name),
                packet.px, packet.py,
                packet.vx, packet.vy
            );
        }
        for (pair, prob) in &sim_manager.collisions {
            if *prob > 0.0 {
                println!("ALERT: {:?} -> Risk: {:.2}%", pair, prob * 100.0);
            }
        }

        iter += 1;

        if iter % 1000 == 0 {
            filter_manager.prune(Duration::from_secs(args.max_age));
        }

    })
}

fn main() -> io::Result<()>{
    let args = Args::parse();

    match args.command {
        Command::Filter => run_filter(args),
        Command::Simulate => run_simulation(args),
    }
}
