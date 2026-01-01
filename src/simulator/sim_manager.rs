use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::simulator::math::Vector2D;
use crate::simulator::model::Aircraft;

pub struct SimManager {
    pub aircrafts: HashMap<Arc<str>, Aircraft>,
    pub collisions: HashMap<(Arc<str>, Arc<str>), f64>,
    max_distance: f64,
}

impl SimManager {
    pub fn new() -> Self {
        Self { aircrafts: HashMap::new(), collisions: HashMap::new(), max_distance: 50_000.0 }
    }

    pub fn handle_update(&mut self, callsign: Arc<str>, px: f64, py: f64, vx: f64, vy: f64) {
        let p = Vector2D::new(px, py);
        let v = Vector2D::new(vx, vy);

        self.aircrafts.entry(callsign.clone())
            .and_modify(|a| a.update(p, v))
            .or_insert(Aircraft::new(p, v));

        self.aircrafts.iter()
            .filter(|(k, _)| **k != callsign)
            .for_each(|(k, other)| {
                let d = p.distance(other.position);
                let key = if callsign < *k {
                    (callsign.clone(), k.clone())
                } else {
                    (k.clone(), callsign.clone())
                };
                if d < self.max_distance {
                    let risk = Self::calculate_risk((p, v), (other.position, other.velocity));
                    if risk < 0.01 {
                        self.collisions.remove(&key);
                    } else {
                        self.collisions.insert(key, risk);
                    }
                } else {
                    self.collisions.remove(&key);
                }
            });

    }

    fn calculate_risk(aircraft: (Vector2D, Vector2D), other: (Vector2D, Vector2D)) -> f64 {
        let mut hits = 0;
        let loops = 1000;
        let max_range = 200.0;
        let noise_magnitude = 1.0;

        for _ in 0..loops {
            let av = aircraft.1.add_noise(noise_magnitude);
            let ov = other.1.add_noise(noise_magnitude);

            for t in 1..=30 {
                let ap = aircraft.0 + av * t as f64;
                let op = other.0 + ov * t as f64;
                if ap.distance(op) < max_range {
                    hits += 1;
                    break;
                }
            }
        }

        hits as f64/loops as f64
    }

    pub fn prune(&mut self, max_age: Duration, center: Vector2D, boundry: f64) {
        let now = Instant::now();

        self.aircrafts.retain(|_, a| {
            now.duration_since(a.last_seen) < max_age
                && a.position.distance(center) < boundry
        });

        self.collisions.retain(|(a, b), _| {
            self.aircrafts.contains_key(a) && self.aircrafts.contains_key(b)
        })
    }

}