use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Local;
use crate::simulator::math::Vector2D;
use crate::simulator::model::Aircraft;

pub struct SimManager {
    pub aircrafts: HashMap<Arc<str>, Aircraft>,
    pub collisions: HashMap<(Arc<str>, Arc<str>), f64>,
    pub adsb_blacklist: HashSet<Arc<str>>,
    scale: f64,
    radar_range: f64,
}

impl SimManager {
    pub fn new(scale: f64) -> Self {
        Self {
            aircrafts: HashMap::new(),
            collisions: HashMap::new(),
            adsb_blacklist: HashSet::new(),
            scale,
            radar_range: (scale * 0.2).powi(2),
        }
    }

    pub fn handle_update(&mut self, callsign: Arc<str>, px: f64, py: f64, vx: f64, vy: f64, alt: f64) {
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
            .or_insert(Aircraft::new(p, v, alt));

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
                    (plane.altitude - other.altitude).abs() < 290.0 &&
                    plane.altitude == other.altitude &&
                    other.position.distance_sq(c) < self.radar_range &&
                    plane.position.distance_sq(other.position) < self.radar_range * 2.0
                {
                    let key = if i < j { (i.clone(), j.clone()) } else { (j.clone(), i.clone()) };
                    let risk = Self::calculate_risk((plane.position, plane.velocity), (other.position, other.velocity));
                    if risk < 0.01 {
                        self.collisions.remove(&key);
                    } else {
                        self.collisions.insert(key, risk);
                        if plane.position.distance_sq(other.position) < 150f64.powi(2) {
                            self.adsb_blacklist.insert(i.clone());
                            self.adsb_blacklist.insert(j.clone());
                        }
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
        let (p1, v1) = aircraft;
        let (p2, v2) = other;

        let mut hits = 0;
        let loops = 1000;
        let noise_magnitude = 5.0f64;
        let collision_range = 150.0f64;
        let collision_range_sq = collision_range.powi(2);

        if p1.distance_sq(p2) <= collision_range_sq {
            return 1.0;
        }

        for _ in 0..loops {
            let dp = p1 - p2;
            let dv_initial = v1 - v2;
            let t_cpa = -(dp.dot(dv_initial) / dv_initial.length_sq()).clamp(0.0, 30.0);
            let scaled_noise = noise_magnitude.max(noise_magnitude * (1.0 + t_cpa * 0.5));

            let v1_new = v1.add_noise(scaled_noise);
            let v2_new = v2.add_noise(scaled_noise);

            let dv = v1_new - v2_new;
            let dv_sq = dv.length_sq();

            if dv_sq > 0.001 {
                let t_cpa = -(dp.dot(dv) / dv_sq);

                if t_cpa > 0.0 && t_cpa < 30.0 {
                    let closest_dist_sq = (dp + dv * t_cpa).length_sq();
                    if closest_dist_sq < collision_range_sq {
                        hits += 1;
                    }
                }
            }
        }

        hits as f64 / loops as f64
    }

    pub fn prune(&mut self, max_age: Duration, center: Vector2D) {
        let now = Instant::now();

        self.aircrafts.retain(|k, a| {
            !self.adsb_blacklist.contains(k) &&
                now.duration_since(a.last_seen) < max_age &&
                a.position.distance(center) < self.scale
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
                    let urgency = r/d.max(1.0);
                    Some((id1, id2, d, p1.altitude, r, urgency))
                } else {
                    None
                }
            })
            .collect();

        display_list.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap());

        println!("\n--- 🚨 CRITICAL ALERTS | [{}] ---", now);
        println!("{:<12} | {:<12} | {:<10} | {:<5}| {} | {:<8}", "Plane A", "Plane B", "Dist (km)", "Alt (m)", "St", "Risk %");
        println!("{}", "-".repeat(63));

        for alert in display_list.iter().take(10) {
            let icon = if self.adsb_blacklist.contains(alert.0) {
                "💥"
            } else if *alert.4 > 0.75 {
                "🔸"
            } else {
                "  "
            };
            println!(
                "{:<12} | {:<12} | {:<10.2} | {:<5} | {} | {:.1}%",
                alert.0,
                alert.1,
                alert.2 / 1000.0,
                alert.3,
                icon,
                alert.4 * 100.0
            );
        }
        println!("{}", "-".repeat(63));
    }

}