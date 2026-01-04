use std::collections::VecDeque;
use std::time::Instant;
use serde::Deserialize;
use crate::simulator::math::Vector2D;

#[derive(Debug, Deserialize)]
pub struct AdsbPacket {
    pub id: String,
    pub callsign: Option<String>,
    pub px: f64,
    pub py: f64,
    pub vx: f64,
    pub vy: f64,
    pub alt: u16,
}

pub struct Aircraft {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub altitude: u16,
    history: VecDeque<Vector2D>,
    pub last_seen: Instant
}

impl Aircraft {
    pub fn new(position: Vector2D, velocity: Vector2D, altitude: u16) -> Self {
        Self { position, velocity, altitude, history: VecDeque::with_capacity(32), last_seen: Instant::now() }
    }

    pub fn update(&mut self, position: Vector2D, velocity: Vector2D) {
        self.history.push_back(self.position);
        self.position = position;
        self.velocity = velocity;
        self.last_seen = Instant::now();
        if self.history.len() > 32 {
            self.history.pop_front();
        }

    }
}