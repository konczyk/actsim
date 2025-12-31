#!/usr/bin/python

import random
import time
import sys
import json

ICAO_CHARS = "0123456789ABCDEF"
ACTIVE_POOL_SIZE = 20
NEW_PLANE_CHANCE = 0.1
CALLSIGNS = [
    "ALPHA", "BRAVO", "CHARLIE", "DELTA", "ECHO", "FOXTROT", "GOLF",
    "HOTEL", "INDIA", "JULIETT", "KILO", "LIMA", "MIKE", "NOVEMBER",
    "OSCAR", "PAPA"
]

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

def generate_icao():
    return "".join(random.choices(ICAO_CHARS, k=6))

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
    return random.uniform(-20000, 20000)

def random_vel():
    return random.uniform(-250, 250)

def main():
    flights = [Aircraft(generate_icao(), callsign, random_pos(), random_pos(), random_vel(), random_vel()) for callsign in [f"{v}{k}" for k,v in enumerate(CALLSIGNS, start=1)]]
    collide1 = Aircraft(generate_icao(), 'COLLIDE1', -20000, 100, 250, 0)
    collide2 = Aircraft(generate_icao(), 'COLLIDE2', 20000, 100, -250, 0)
    flights.extend([collide1, collide2])

    dt = 0.5
    try:
        while True:

            for aircraft in flights:
                aircraft.update(dt)
                print(json.dumps(aircraft.to_dict()))

            if -300 < collide1.px < 300 or -300 < collide2.px < 300:
                collide1.px = -20000
                collide2.px = 20000

            for _ in range(0, 10):
                print(json.dumps(generate_noise()))

            sys.stdout.flush()

            time.sleep(dt)
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()