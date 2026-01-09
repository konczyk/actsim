use crate::simulator::grid::SpatialGrid;
use crate::simulator::math::Vector2D;
use crate::simulator::model::Aircraft;
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::prelude::*;

pub struct SimManager {
    pub aircraft: HashMap<Arc<str>, Aircraft>,
    pub collisions: HashMap<(Arc<str>, Arc<str>), (f64, Option<f64>)>,
    pub adsb_blacklist: HashSet<Arc<str>>,
    pub spatial_grid: SpatialGrid,
    scale: f64,
    radar_range: f64,
}

impl SimManager {
    pub fn new(scale: f64) -> Self {
        Self {
            aircraft: HashMap::new(),
            collisions: HashMap::new(),
            adsb_blacklist: HashSet::new(),
            spatial_grid: SpatialGrid::new(15_000),
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
            self.aircraft.remove(&callsign);
            return;
        }

        self.aircraft.entry(callsign.clone())
            .and_modify(|a| a.update(p, v, self.spatial_grid.to_grid_coord(p)))
            .or_insert(Aircraft::new(p, v, alt, self.spatial_grid.to_grid_coord(p)));

    }

    pub fn check_collisions(&mut self) {
        let c = Vector2D::new(0.0, 0.0);
        self.spatial_grid.clear();

        for (id, plane) in &self.aircraft {
            self.spatial_grid.insert(id.clone(), plane.position);
        }

        let aircraft = &self.aircraft;

        let result: HashMap<(Arc<str>, Arc<str>), (f64, Option<f64>)> = aircraft
            .par_iter()
            .filter(|(_, plane)| plane.position.distance_sq(c) <= self.radar_range)
            .flat_map(|(id_i, plane)| {
                self.spatial_grid.get_nearby_ids(id_i, plane.position)
                    .filter(|id_j| id_i < id_j)
                    .map(|id_j| {
                        let other = &self.aircraft[id_j];
                        let key = (id_i.clone(), id_j.clone());

                        if plane.altitude == other.altitude {
                            let (risk, tti) = Self::calculate_risk((plane.position, plane.velocity), (other.position, other.velocity));
                            (key, (risk, tti))
                        } else {
                            (key, (0.0, None))
                        }
                    })
                    .collect::<Vec<_>>()
            }).collect();

        self.collisions.clear();
        result.into_iter()
            .filter(|(_, (risk, _))| *risk > 0.01)
            .for_each(|(k, risk)| {
                let plane = &self.aircraft[&k.0];
                let other = &self.aircraft[&k.1];
                self.collisions.insert(k.clone(), risk);
                if plane.position.distance_sq(other.position) < 150f64.powi(2) {
                    self.adsb_blacklist.insert(k.0.clone());
                    self.adsb_blacklist.insert(k.1.clone());
                }
            });
    }

    fn calculate_risk(aircraft: (Vector2D, Vector2D), other: (Vector2D, Vector2D)) -> (f64, Option<f64>) {
        let (p1, v1) = aircraft;
        let (p2, v2) = other;

        let mut hits = 0;
        let loops = 1000;
        let noise_magnitude = 5.0f64;
        let collision_range = 150.0f64;
        let collision_range_sq = collision_range.powi(2);

        if p1.distance_sq(p2) <= collision_range_sq {
            return (1.0, Some(0.0));
        }

        let mut total_hit_time = 0.0;
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
                        total_hit_time += t_cpa;
                        hits += 1;
                    }
                }
            }
        }

        (hits as f64 / loops as f64, if hits > 0 { Some(total_hit_time / hits as f64) } else { None } )
    }

    pub fn prune(&mut self, max_age: Duration, center: Vector2D) {
        let now = Instant::now();

        self.aircraft.retain(|k, a| {
            !self.adsb_blacklist.contains(k) &&
                now.duration_since(a.last_seen) < max_age &&
                a.position.distance(center) < self.scale
        });

        self.adsb_blacklist.clear();

        self.collisions.retain(|(a, b), _| {
            self.aircraft.contains_key(a) && self.aircraft.contains_key(b)
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
            .filter_map(|((id1, id2), (r, t))| {
                if let (Some(p1), Some(p2)) = (self.aircraft.get(id1), self.aircraft.get(id2)) {
                    let d = p1.position.distance(p2.position);
                    let urgency = r/(t.unwrap_or(1.0) * d.max(1.0));
                    Some((id1, id2, d, p1.altitude, t, r, urgency))
                } else {
                    None
                }
            })
            .collect();

        display_list.sort_by(|a, b| b.6.partial_cmp(&a.6).unwrap());

        println!("\n--- 🚨 CRITICAL ALERTS | [{}] ---", now);
        println!("{:<12} | {:<12} | {:<10} | {:<6} | {} | {:<6} | {:<8}", "Plane A", "Plane B", "Dist (km)", "Alt (m)", "St", "TTI (s)", "Risk %");
        println!("{}", "-".repeat(74));

        for alert in display_list.iter().take(10) {
            let icon = if self.adsb_blacklist.contains(alert.0) {
                "💥"
            } else if *alert.5 > 0.75 {
                "🔸"
            } else {
                "  "
            };
            println!(
                "{:<12} | {:<12} | {:<10.2} | {:<7} | {} | {:<7} | {:.1}%",
                alert.0,
                alert.1,
                alert.2 / 1000.0,
                alert.3,
                icon,
                alert.4.map(|x| format!("{:.1}", x)).unwrap_or("".to_string()),
                alert.5 * 100.0
            );
        }
        println!("{}", "-".repeat(74));
    }

}