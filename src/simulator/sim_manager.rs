use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Local;
use crate::simulator::math::Vector2D;
use crate::simulator::model::Aircraft;

pub struct SimManager {
    pub aircrafts: HashMap<Arc<str>, Aircraft>,
    pub collisions: HashMap<(Arc<str>, Arc<str>), f64>,
    scale: f64,
    radar_range: f64,
}

impl SimManager {
    pub fn new(scale: f64) -> Self {
        Self {
            aircrafts: HashMap::new(),
            collisions: HashMap::new(),
            scale,
            radar_range: (scale * 0.2).powi(2),
        }
    }

    pub fn handle_update(&mut self, callsign: Arc<str>, px: f64, py: f64, vx: f64, vy: f64) {
        let p = Vector2D::new(px, py);
        let v = Vector2D::new(vx, vy);
        let c = Vector2D::new(0.0, 0.0);

        let max_speed = 250.0;
        let lookahead_seconds = 30.0;
        let safety_buffer = ((max_speed * 2.0) * lookahead_seconds as f64).powi(2);

        if p.distance_sq(c) > self.radar_range + safety_buffer {
            self.aircrafts.remove(&callsign);
            return;
        }

        self.aircrafts.entry(callsign.clone())
            .and_modify(|a| a.update(p, v))
            .or_insert(Aircraft::new(p, v));

    }

    pub fn check_collisions(&mut self) {
        let c = Vector2D::new(0.0, 0.0);
        let ids: Vec<Arc<str>> = self.aircrafts.keys().cloned().collect();

        for (idx, i) in ids.iter().enumerate() {
            let plane = &self.aircrafts[i];
            let plane_in_radar = plane.position.distance_sq(c) < self.radar_range;

            if !plane_in_radar && self.collisions.is_empty() {
                continue;
            }

            for j in &ids[idx + 1..] {
                let other = &self.aircrafts[j];
                if plane_in_radar &&
                    other.position.distance_sq(c) < self.radar_range &&
                    plane.position.distance_sq(other.position) < self.radar_range * 2.0
                {
                    let key = if i < j { (i.clone(), j.clone()) } else { (j.clone(), i.clone()) };
                    let risk = Self::calculate_risk((plane.position, plane.velocity), (other.position, other.velocity));
                    if risk < 0.01 {
                        self.collisions.remove(&key);
                    } else {
                        self.collisions.insert(key, risk);
                    }
                } else if !self.collisions.is_empty() {
                    let key = if i < j { (i.clone(), j.clone()) } else { (j.clone(), i.clone()) };
                    if self.collisions.contains_key(&key) {
                        self.collisions.remove(&key);
                    }
                }

            }
        }
    }

    fn calculate_risk(aircraft: (Vector2D, Vector2D), other: (Vector2D, Vector2D)) -> f64 {
        let mut hits = 0;
        let loops = 1000;
        let noise_magnitude = 1.0;
        let collision_range = 500.0f64.powi(2);

        for _ in 0..loops {
            let av = aircraft.1.add_noise(noise_magnitude);
            let ov = other.1.add_noise(noise_magnitude);

            for t in 1..=30 {
                let ap = aircraft.0 + av * t as f64;
                let op = other.0 + ov * t as f64;
                if ap.distance_sq(op) < collision_range {
                    hits += 1;
                    break;
                }
            }
        }

        hits as f64/loops as f64
    }

    pub fn prune(&mut self, max_age: Duration, center: Vector2D) {
        let now = Instant::now();

        self.aircrafts.retain(|_, a| {
            now.duration_since(a.last_seen) < max_age
                && a.position.distance(center) < self.scale
        });

        self.collisions.retain(|(a, b), _| {
            self.aircrafts.contains_key(a) && self.aircrafts.contains_key(b)
        })
    }

    pub fn print_collision_summary(&self) {
        if self.collisions.is_empty() {
            return;
        }

        let now = Local::now().format("%H:%M:%S%.3f");

        let mut entries: Vec<_> = self.collisions.iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        let mut display_list: Vec<_> = entries.into_iter()
            .take(20)
            .filter_map(|((id1, id2), r)| {
                if let (Some(p1), Some(p2)) = (self.aircrafts.get(id1), self.aircrafts.get(id2)) {
                    let d = p1.position.distance(p2.position);
                    Some((id1, id2, d, r))
                } else {
                    None
                }
            })
            .collect();

        display_list.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        println!("\n--- 🚨 CRITICAL ALERTS | [{}] ---", now);
        println!("{:<12} | {:<12} | {:<10} | {:<8}", "Plane A", "Plane B", "Dist (km)", "Risk %");
        println!("{}", "-".repeat(50));

        for alert in display_list.iter().take(10) {
            println!(
                "{:<12} | {:<12} | {:<10.2} | {:.1}%",
                alert.0,
                alert.1,
                alert.2 / 1000.0,
                alert.3 * 100.0
            );
        }
        println!("{}", "-".repeat(50));
    }

}