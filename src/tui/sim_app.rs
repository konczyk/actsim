use crate::filter::filter_manager::{FilterManager, FilterResult};
use crate::simulator::math::Vector2D;
use crate::simulator::model::AdsbPacket;
use crate::simulator::sim_manager::SimManager;
use crate::Args;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;
use std::io;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct SimulatorApp {
    terminal: DefaultTerminal,
    filter_manager: FilterManager<Arc<str>>,
    sim_manager: SimManager,
    tick_interval: Duration,
    last_tick: Instant,
    prune_interval: Duration,
    last_prune: Instant,
    last_reported_risk: HashMap<(Arc<str>, Arc<str>), f64>,
    args: Args,
}

impl SimulatorApp {
    pub fn new(args: Args) -> SimulatorApp {
        SimulatorApp {
            terminal: ratatui::init(),
            filter_manager: FilterManager::new(),
            sim_manager: SimManager::new(200_000.0),
            tick_interval: Duration::from_millis(100),
            last_tick: Instant::now(),
            prune_interval: Duration::from_secs(5),
            last_prune: Instant::now(),
            last_reported_risk: HashMap::new(),
            args,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.terminal.draw(|mut frame| Self::draw(&mut frame))?;
            if crossterm::event::poll(Duration::from_millis(16))? {
                match crossterm::event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') => return Ok(()),
                    _ => continue
                }
            }
        }
    }

    fn draw(frame: &mut Frame) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title("ACT Simulator")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        frame.render_widget(block, frame.area());
    }

    pub fn handle_packet(&mut self, packet: AdsbPacket) {
        let name = Arc::from(packet.callsign.unwrap_or(packet.id));

        if self.sim_manager.adsb_blacklist.contains(&name) {
            return;
        }
        if self.last_prune.elapsed() > self.prune_interval {
            let pending = self.filter_manager.pending.len();

            self.sim_manager.prune(
                Duration::from_secs(10),
                Vector2D::new(0.0, 0.0)
            );
            self.filter_manager.prune(
                Duration::from_secs(self.args.max_age)
            );
            self.last_reported_risk.retain(|k, _| self.sim_manager.collisions.contains_key(k));

            self.last_prune = Instant::now();
            if self.args.debug {
                let s = self.filter_manager.stats();
                eprintln!(
                    "[DEBUG] Layers: {} | Fill: {:.1}% | Bits: {} | Est. FPR: {:.2}% | Pending: {} | Tracks: {}",
                    s.layer_count,
                    s.fill_ratio * 100.0,
                    s.total_bits,
                    s.est_fpr * 100.0,
                    pending,
                    self.sim_manager.aircraft.len(),
                );
            }
        }

        if self.last_tick.elapsed() >= self.tick_interval {
            self.sim_manager.check_collisions();
            self.sim_manager.print_collision_summary();
            if self.args.debug {
                let total_processing_time = self.last_tick.elapsed() - self.tick_interval;
                let pairs_checked = self.sim_manager.metrics.pairs_checked.swap(0, Ordering::Relaxed);
                let throughput = if pairs_checked == 0 { 0 } else { (pairs_checked as f64 / total_processing_time.as_millis() as f64).ceil() as u64 };
                println!("[DEBUG] Total Processing Time: {:.1?} | Pairs checked: {} | Throughput: {} pairs/ms", total_processing_time, pairs_checked, throughput);
            }
            self.last_tick = Instant::now();
        }

        if self.filter_manager.insert(&name) != FilterResult::Pending {
            self.sim_manager.handle_update(
                Arc::from(name),
                packet.px, packet.py,
                packet.vx, packet.vy,
                packet.alt
            );

            for (pair, (prob, _)) in &self.sim_manager.collisions {
                if *prob > 0.0 && self.last_reported_risk.get(pair).map(|x| (x-prob).abs() > 0.05).unwrap_or(true) {
                    self.last_reported_risk.insert(pair.clone(), *prob);
                }
            }
        }
    }
}

impl Drop for SimulatorApp {
    fn drop(&mut self) {
        ratatui::restore();
    }
}