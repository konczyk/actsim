#!/usr/bin/python

import random
import time
import sys

ICAO_CHARS = "0123456789ABCDEF"
ACTIVE_POOL_SIZE = 15
NEW_PLANE_CHANCE = 0.1

def generate_icao():
    return "".join(random.choices(ICAO_CHARS, k=6))

def main():
    # Initialize our "airspace"
    pool = [generate_icao() for _ in range(ACTIVE_POOL_SIZE)]

    try:
        while True:
            if random.random() < NEW_PLANE_CHANCE:
                pool[random.randrange(ACTIVE_POOL_SIZE)] = generate_icao()

            target_plane = random.choice(pool)

            sys.stdout.write(f"{target_plane}\n")
            sys.stdout.flush()

            time.sleep(0.05)
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()