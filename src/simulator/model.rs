use std::collections::VecDeque;
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
}

pub struct Aircraft {
    pub position: Vector2D,
    pub velocity: Vector2D,
    history: VecDeque<Vector2D>
}

impl Aircraft {
    pub fn new(position: Vector2D, velocity: Vector2D) -> Self {
        Self { position, velocity, history: VecDeque::with_capacity(32) }
    }

    pub fn update(&mut self, position: Vector2D, velocity: Vector2D) {
        self.history.push_back(self.position);
        self.position = position;
        self.velocity = velocity;
        if self.history.len() > 32 {
            self.history.pop_front();
        }

    }
}