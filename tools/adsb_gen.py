#!/usr/bin/python
import math
import os
import random
import time
import sys
import json
import argparse

ICAO_CHARS = "0123456789ABCDEF"

SCALE = 200_000
MIN_SPEED = 100
MAX_SPEED = 250
SPAWN_RADIUS = 50_000

class Aircraft:
    def __init__(self, icao, callsign, px, py, vx, vy, alt):
        self.icao = icao
        self.callsign = callsign
        self.px = px
        self.py = py
        self.vx = vx
        self.vy = vy
        self.alt = alt

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
            "alt": self.alt
        }

def get_grid_flights(num_planes, jitter):
    side = int(math.sqrt(num_planes))
    spacing = (SCALE * 2) / side
    flights = []
    for x in range(side):
        for y in range(side):
            px = -SCALE + (x * spacing)
            py = -SCALE + (y * spacing)

            dist_from_center = math.sqrt(px**2 + py**2)
            speed = MIN_SPEED + (dist_from_center / (SCALE * math.sqrt(2))) * (MAX_SPEED - MIN_SPEED)

            if dist_from_center > 0:
                if jitter is False:
                    vx = (-px / dist_from_center) * speed
                    vy = (-py / dist_from_center) * speed
                else:
                    angle = random.uniform(0, 2 * math.pi)
                    vx = math.cos(angle) * speed
                    vy = math.sin(angle) * speed

                flights.append(Aircraft(f"GRID{x}-{y}",  f"FLIGHT-{x}-{y}", px, py, vx, vy, random_alt()))
    return flights

def gen_corridor_flight(num_plane):
    gates = [
        (0, SPAWN_RADIUS), (0, -SPAWN_RADIUS), (SPAWN_RADIUS, 0), (-SPAWN_RADIUS, 0),
        (SPAWN_RADIUS, SPAWN_RADIUS), (-SPAWN_RADIUS, -SPAWN_RADIUS), (SPAWN_RADIUS, -SPAWN_RADIUS), (-SPAWN_RADIUS, SPAWN_RADIUS)
    ]
    start = random.choice(gates)
    valid_ends = [
        g for g in gates
        if math.sqrt((g[0]-start[0])**2 + (g[1]-start[1])**2) > (SPAWN_RADIUS * 1.4)
    ]
    end = random.choice(valid_ends)

    px, py = start[0] + random.uniform(-5000, 5000), start[1] + random.uniform(-5000, 5000)
    dist = math.sqrt((end[0]-px)**2 + (end[1]-py)**2)
    speed = random.uniform(MIN_SPEED, MAX_SPEED)
    vx, vy = (end[0]-px)/dist * speed, (end[1]-py)/dist * speed

    return Aircraft(f"CORR-{num_plane}", f"FLIGHT-{num_plane}", px, py, vx, vy, random_alt())

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
        "alt": random_alt(),
    }

def random_pos():
    return random.uniform(-SCALE, SCALE)

def random_vel():
    return random.uniform(MIN_SPEED, MAX_SPEED) * random.choice([-1, 1])

def random_alt():
    return random.randrange(10000, 13000, 300)

def main():
    parser = argparse.ArgumentParser(description="ADS-B Traffic Generator")
    parser.add_argument("--mode", choices=["grid", "jitter", "corridor"], default="grid")
    parser.add_argument("--planes", type=int, default=1024)
    parser.add_argument("--noise", type=int, default=16, help="Noise packets per tick")
    parser.add_argument("--tick", type=float, default=0.5, help="Tick time in seconds")
    args = parser.parse_args()

    flights = []
    plane_id_counter = 0
    last_spawn_time = 0

    if args.mode in ["grid", "jitter"]:
        flights = get_grid_flights(args.planes, jitter=(args.mode == "jitter"))

    try:
        while True:

            if args.mode == "corridor" and len(flights) < args.planes:
                current_time = time.time()
                if (current_time - last_spawn_time) >= 0.5:
                    num = min(random.randint(32, 64), args.planes - len(flights))
                    for _ in range(num):
                        flights.append(gen_corridor_flight(plane_id_counter))
                        plane_id_counter += 1
                    last_spawn_time = current_time

            for aircraft in flights:
                aircraft.update(args.tick)
                print(json.dumps(aircraft.to_dict()))

            for _ in range(0, random.randint(1, args.noise)):
                print(json.dumps(generate_noise()))

            sys.stdout.flush()
            time.sleep(args.tick)
    except (BrokenPipeError, KeyboardInterrupt):
        sys.exit(0)

if __name__ == "__main__":
    main()
