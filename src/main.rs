use crate::filter::filter_manager;
use crate::filter::filter_manager::FilterResult;
use crate::simulator::math::Vector2D;
use crate::simulator::model::AdsbPacket;
use crate::simulator::sim_manager;
use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::sync::Arc;
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
    let mut filter_manager = filter_manager::FilterManager::new();
    let mut sim_manager = sim_manager::SimManager::new(200_000.0);

    let mut last_prune = Instant::now();
    let prune_interval = Duration::from_secs(5);
    let mut last_tick = Instant::now();
    let tick_interval  = Duration::from_millis(500);

    let mut last_reported_risk: HashMap<(Arc<str>, Arc<str>), f64> = HashMap::new();

    process_adsb_stream(|packet| {
        if last_prune.elapsed() > prune_interval {
            let pending = filter_manager.pending.len();

            sim_manager.prune(
                Duration::from_secs(10),
                Vector2D::new(0.0, 0.0)
            );
            filter_manager.prune(
                Duration::from_secs(args.max_age)
            );
            last_reported_risk.retain(|k, _| sim_manager.collisions.contains_key(k));

            last_prune = Instant::now();
            if args.debug {
                let s = filter_manager.stats();
                eprintln!(
                    "[DEBUG] Layers: {} | Fill: {:.1}% | Bits: {} | Est. FPR: {:.2}% | Pending: {} | Tracks: {}",
                    s.layer_count,
                    s.fill_ratio * 100.0,
                    s.total_bits,
                    s.est_fpr * 100.0,
                    pending,
                    sim_manager.aircrafts.len(),
                );
            }
        }

        if last_tick.elapsed() > tick_interval {
            sim_manager.check_collisions();
            last_tick = Instant::now();
        }

        if filter_manager.insert(&packet.id) != FilterResult::Pending {
            let name = packet.callsign.unwrap_or(packet.id);
            sim_manager.handle_update(
                Arc::from(name),
                packet.px, packet.py,
                packet.vx, packet.vy,
            );

            for (pair, prob) in &sim_manager.collisions {
                if *prob > 0.0 && last_reported_risk.get(pair).map(|x| (x-prob).abs() > 0.05).unwrap_or(true) {
                    println!("ALERT: {:?} -> Risk: {:.2}%", pair, prob * 100.0);
                    last_reported_risk.insert(pair.clone(), *prob);
                }
            }
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
