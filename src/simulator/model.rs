use std::collections::VecDeque;
use std::time::Instant;
use serde::Deserialize;
use crate::simulator::grid::GridCoord;
use crate::simulator::math::Vector2D;

#[derive(Debug, Deserialize)]
pub struct AdsbPacket {
    pub id: String,
    pub callsign: Option<String>,
    pub px: f64,
    pub py: f64,
    pub vx: f64,
    pub vy: f64,
    pub alt: f64,
}

pub struct Aircraft {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub altitude: f64,
    history: VecDeque<Vector2D>,
    pub last_seen: Instant,
    pub grid_coord: GridCoord,

}

impl Aircraft {
    pub fn new(position: Vector2D, velocity: Vector2D, altitude: f64, grid_coord: GridCoord) -> Self {
        Self { position, velocity, altitude, history: VecDeque::with_capacity(32), last_seen: Instant::now(), grid_coord }
    }

    pub fn update(&mut self, position: Vector2D, velocity: Vector2D, grid_coord: GridCoord) {
        self.history.push_back(self.position);
        self.position = position;
        self.velocity = velocity;
        self.last_seen = Instant::now();
        self.grid_coord = grid_coord;
        if self.history.len() > 32 {
            self.history.pop_front();
        }

    }
}