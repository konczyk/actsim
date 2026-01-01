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

MIN_POS = -25000
MAX_POS = 25000
MIN_SPEED = 100
MAX_SPEED = 250


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
    return random.uniform(MIN_POS, MAX_POS)

def random_vel():
    return random.uniform(MIN_SPEED, MAX_SPEED) * random.choice([-1, 1])

def main():
    flights = [
        Aircraft(
            generate_icao(), callsign,
            random_pos(), random_pos(),
            random_vel(), random_vel()
        )
        for callsign in [f"{name}{i}" for _, name in enumerate(CALLSIGNS) for i in range(1, 3)]
    ]

    collide1 = Aircraft(generate_icao(), 'COLLIDE1', MIN_POS, 100, MAX_SPEED, 0)
    collide2 = Aircraft(generate_icao(), 'COLLIDE2', MAX_POS, 100, -MAX_SPEED, 0)
    flights.extend([collide1, collide2])

    dt = 0.5
    try:
        while True:

            for aircraft in flights:
                aircraft.update(dt)
                print(json.dumps(aircraft.to_dict()))

            if collide1.px > collide2.px:
                collide1.px = MIN_POS
                collide2.px = MAX_POS

            for _ in range(0, 10):
                print(json.dumps(generate_noise()))

            sys.stdout.flush()

            time.sleep(dt)
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()