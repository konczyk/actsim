use crate::filter::filter_manager::{FilterManager, FilterResult};
use crate::simulator::math::Vector2D;
use crate::simulator::model::AdsbPacket;
use crate::simulator::sim_manager::SimManager;
use crate::Args;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;
use std::io;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct AppMetrics {
    pub pairs_checked: u64,
    pub throughput: u64,
    pub total_processing_time: Duration,
}

pub struct SimApp {
    terminal: DefaultTerminal,
    filter_manager: FilterManager<Arc<str>>,
    sim_manager: SimManager,
    receiver: Receiver<AdsbPacket>,
    tick_interval: Duration,
    last_tick: Instant,
    prune_interval: Duration,
    last_prune: Instant,
    last_reported_risk: HashMap<(Arc<str>, Arc<str>), f64>,
    metrics: AppMetrics,
    args: Args,
}

impl SimApp {

    const SCALE: f64 = 200_000.0;

    pub fn new(args: Args, receiver: Receiver<AdsbPacket>) -> SimApp {
        SimApp {
            terminal: ratatui::init(),
            filter_manager: FilterManager::new(),
            sim_manager: SimManager::new(Self::SCALE),
            receiver,
            tick_interval: Duration::from_millis(100),
            last_tick: Instant::now(),
            prune_interval: Duration::from_secs(5),
            last_prune: Instant::now(),
            last_reported_risk: HashMap::new(),
            metrics: AppMetrics {
                pairs_checked: 0,
                throughput: 0,
                total_processing_time: Duration::from_secs(0),
            },
            args,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let mut processed_this_frame = 0;
            while let Ok(packet) = self.receiver.try_recv() {
                self.handle_packet(packet);
                processed_this_frame += 1;
                if processed_this_frame == 1000 {
                    break;
                }
            }
            if self.last_tick.elapsed() >= self.tick_interval {
                self.sim_manager.colliding.clear();
                self.sim_manager.check_collisions();
                self.metrics.total_processing_time = self.last_tick.elapsed() - self.tick_interval;
                self.metrics.pairs_checked = self.sim_manager.metrics.pairs_checked.swap(0, Ordering::Relaxed);
                self.metrics.throughput = if self.metrics.pairs_checked == 0 { 0 } else { (self.metrics.pairs_checked as f64 / self.metrics.total_processing_time.as_millis() as f64).ceil() as u64 };
                self.last_tick = Instant::now();
            }

            self.terminal.draw(|mut frame| Self::draw(&mut frame, &self.metrics, &self.filter_manager, &self.sim_manager))?;

            if crossterm::event::poll(Duration::from_millis(16))? {
                match crossterm::event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') => return Ok(()),
                    _ => continue
                }
            }
        }
    }

    fn draw(frame: &mut Frame, app: &AppMetrics, filter: &FilterManager<Arc<str>>, sim_manager: &SimManager) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title("ACT Simulator")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        frame.render_widget(block, frame.area());

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(frame.area());

        Self::draw_radar(frame, main_layout[0], sim_manager);

        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(5),
            ])
            .split(main_layout[1]);

        Self::draw_metrics(frame, sidebar_chunks[0], &app);
        Self::draw_filter_status(frame, sidebar_chunks[1], &filter, &sim_manager);
        Self::draw_alerts(frame, sidebar_chunks[2], &sim_manager);
    }

    fn draw_radar(frame: &mut Frame, area: Rect, sim_manager: &SimManager) {
        let range = sim_manager.radar_range.sqrt()*1.1;

        let canvas = Canvas::default()
            .block(Block::default()
                .title(" [RADAR SCOPE] ")
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default()))
            .x_bounds([-range, range])
            .y_bounds([-range, range])
            .paint(|ctx| {

                for (id, aircraft) in &sim_manager.aircraft {
                    let color = if sim_manager.colliding.contains(id) { Color::Red } else { Color::Green };

                    ctx.print(aircraft.position.x, aircraft.position.y,
                        Span::styled("✦", Style::default().fg(color)));
                }
            });

        frame.render_widget(canvas, area);
    }

    fn draw_metrics(frame: &mut Frame, area: Rect, app: &AppMetrics) {
        let stats_text = vec![
            Line::from(vec![
                Span::styled(" Pairs Checked: ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{}", app.pairs_checked), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" Throughput:    ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{} p/ms", app.throughput), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" Tick Time:     ", Style::default().fg(Color::LightBlue)),
                Span::styled(
                    format!("{:.1?}", app.total_processing_time),
                    Style::default().fg(if app.total_processing_time.as_millis() > 100 { Color::Red } else { Color::default()})
                ),
            ]),
        ];

        let block = Block::default()
            .title(" System Metrics ")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        frame.render_widget(Paragraph::new(stats_text).block(block), area);
    }

    fn draw_filter_status(frame: &mut Frame, area: Rect, filter: &FilterManager<Arc<str>>, sim_manager: &SimManager) {
        let stats = filter.stats();

        let filled = (stats.fill_ratio * 100.0).min(10.0) as usize;
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(10 - filled));

        let stats_text = vec![
            Line::from(vec![
                Span::styled(" Layers:  ", Style::default().fg(Color::LightBlue)),
                Span::styled(bar, Style::default()),
                Span::styled(format!(" {}", stats.layer_count), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" Bits:    ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{}", stats.total_bits), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" Tracks:  ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{}", sim_manager.aircraft.len()), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" Pending: ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{}", stats.pending), Style::default()),
            ]),
            Line::from(vec![
                Span::styled(" FPR:     ", Style::default().fg(Color::LightBlue)),
                Span::styled(format!("{:.2}%", stats.est_fpr * 100.0), Style::default()),
            ]),
        ];

        let block = Block::default()
            .title(" [Filter Status] ") // Brackets in title for that "sketch" look
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded) // Rounded corners look cleaner
            .border_style(Style::default());

        frame.render_widget(Paragraph::new(stats_text).block(block), area);

    }

    fn draw_alerts(frame: &mut Frame, area: Rect, sim_manager: &SimManager) {
        let mut entries: Vec<_> = sim_manager.collisions.iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        let mut display_list: Vec<_> = entries.into_iter()
            .take(20)
            .filter_map(|((id1, id2), (r, t))| {
                if let (Some(p1), Some(p2)) = (sim_manager.aircraft.get(id1), sim_manager.aircraft.get(id2)) {
                    let d = p1.position.distance(p2.position);
                    let urgency = r/(t.unwrap_or(1.0) * d.max(1.0));
                    Some((id1, id2, d, p1.altitude, t, r, urgency))
                } else {
                    None
                }
            })
            .collect();

        display_list.sort_by(|a, b| b.6.partial_cmp(&a.6).unwrap());

        let rows: Vec<Row> = display_list.iter().take(10).map(|(id1, id2, d, _alt, t, r, _u)| {
            let icon = if sim_manager.adsb_blacklist.contains(*id1) {
                "💥"
            } else if **r > 0.75 {
                "🔸"
            } else {
                "  "
            };
            Row::new(vec![
                Cell::from(format!("{}<->{}", id1, id2)),
                Cell::from(format!("{:.0}m", d)),
                Cell::from(format!("{}", icon)),
                Cell::from(t.map(|x| format!("{:.1}", x)).unwrap_or("".to_string())),
                Cell::from(format!("{:.0}%", *r * 100.0)),
            ])
        }).collect();

        let table = Table::new(rows, [
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ])
        .header(Row::new(vec!["ID PAIR", "DIST", "ST", "TTI", "RISK"]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default().title(" [Active Alerts] ").borders(Borders::ALL).border_type(BorderType::Rounded));

        frame.render_widget(table, area);
    }

    pub fn handle_packet(&mut self, packet: AdsbPacket) {
        let name = Arc::from(packet.callsign.unwrap_or(packet.id));

        if self.sim_manager.adsb_blacklist.contains(&name) {
            return;
        }
        if self.last_prune.elapsed() > self.prune_interval {

            self.sim_manager.prune(
                Duration::from_secs(10),
                Vector2D::new(0.0, 0.0)
            );
            self.filter_manager.prune(
                Duration::from_secs(self.args.max_age)
            );
            self.last_reported_risk.retain(|k, _| self.sim_manager.collisions.contains_key(k));

            self.last_prune = Instant::now();
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

impl Drop for SimApp {
    fn drop(&mut self) {
        ratatui::restore();
    }
}