#!/usr/bin/python
import math
import random
import time
import sys
import json

ICAO_CHARS = "0123456789ABCDEF"

SCALE = 1_000_000
MIN_SPEED = 100
MAX_SPEED = 250
NUM_PLANES = 4096
SIDE = int(math.sqrt(NUM_PLANES))
SPACING = (SCALE * 2) / SIDE
NUM_NOISE = 64

class Aircraft:
    def __init__(self, icao, callsign, px, py, vx, vy):
        self.icao = icao
        self.callsign = callsign
        self.px = px
        self.py = py
        self.vx = vx
        self.vy = vy

    def update(self, dt):
        self.px += self.vx * dt
        self.py += self.vy * dt

    def to_dict(self):
        return {
            "id": self.icao,
            "callsign": self.callsign,
            "px": round(self.px, 2),
            "py": round(self.py, 2),
            "vx": round(self.vx, 2),
            "vy": round(self.vy, 2),
        }

def get_grid_flights():
    flights = []
    for x in range(SIDE):
        for y in range(SIDE):
            px = -SCALE + (x * SPACING)
            py = -SCALE + (y * SPACING)

            vx = -px / 100.0
            vy = -py / 100.0

            flights.append(Aircraft(f"ICAO{x}-{y}",  f"PLN-{x}-{y}", px, py, vx, vy))
    return flights

def generate_icao():
    return "".join(random.choices(ICAO_CHARS, k=8))

def generate_noise():
    return {
        "id": generate_icao(),
        "callsign": None,
        "px": random_pos(),
        "py": random_pos(),
        "vx": random_vel(),
        "vy": random_vel(),
    }

def random_pos():
    return random.uniform(-SCALE, SCALE)

def random_vel():
    return random.uniform(MIN_SPEED, MAX_SPEED) * random.choice([-1, 1])

def main():
    flights = get_grid_flights()

    dt = 0.5
    try:
        while True:

            for aircraft in flights:
                aircraft.update(dt)
                print(json.dumps(aircraft.to_dict()))

            for _ in range(0, random.randint(1,NUM_NOISE)):
                print(json.dumps(generate_noise()))

            sys.stdout.flush()

            time.sleep(dt)
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()
