use std::collections::VecDeque;
use crate::simulator::math::Vector2D;

struct Aircraft {
    callsign: String,
    position: Vector2D,
    velocity: Vector2D,
    history: VecDeque<Vector2D>
}

impl Aircraft {
    fn new(callsign: String, position: Vector2D, velocity: Vector2D) -> Self {
        Self { callsign, position, velocity, history: VecDeque::with_capacity(32) }
    }
    fn project(&self, seconds: f64) -> Vector2D {
        self.position + self.velocity * seconds
    }

    fn update(&mut self, position: Vector2D) {
        self.history.push_back(self.position);
        self.position = position;
        if self.history.len() > 32 {
            self.history.pop_front();
        }

    }
}