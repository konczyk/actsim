# ACTsim 

Simple ACT simulator including a scalable ADS-B filter with a near constant false positive ratio and pruning of old data.

## Usage

Build project
```shell
cargo build -r
```

Run tests
```shell
cargo test
```

Options
```shell
$ ./target/release/actsim -h
Usage: actsim [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  [possible values: filter]

Options:
      --max-age <MAX_AGE>  Max age in seconds for a filter before pruning [default: 300]
  -h, --help               Print help
```

## Examples

Run the ADS-B filter against a test data stream
```shell
$ ./tools/adsb_gen.py | ./target/release/actsim filter
NEW:    17DDE3
NEW:    BA344C
NEW:    DF9C6D
NEW:    372D6C
NEW:    2258C3
NEW:    136A77
NEW:    74AB7C
NEW:    42F157
MATCH:  136A77 (Est. FPR: 1.5625%)
MATCH:  372D6C (Est. FPR: 1.5625%)
MATCH:  2258C3 (Est. FPR: 1.5625%)
NEW:    B89861
MATCH:  74AB7C (Est. FPR: 1.5625%)
MATCH:  17DDE3 (Est. FPR: 1.5625%)
MATCH:  BA344C (Est. FPR: 1.5625%)
NEW:    EB9C72
[...]
```

Run simulation on a 200km scale plane, with 4096 aircraft flying into the center and up to 64 noise packets/s
```shell
$ ./tools/adsb_gen.py --planes 4096 --noise 64 | ./target/release/actsim simulate -d
[...]
[DEBUG] Layers: 2 | Fill: 5.1% | Bits: 475136 | Est. FPR: 0.29% | Pending: 149 | Tracks: 176

--- 🚨 CRITICAL ALERTS | [03:31:29.311] ---
Plane A      | Plane B      | Dist (km)  | Alt (m)| St | Risk %  
---------------------------------------------------------------
FLIGHT-27-32 | FLIGHT-33-32 | 0.12       | 10000 | 💥 | 100.0%
FLIGHT-35-31 | FLIGHT-35-33 | 0.52       | 11800 | 🔸 | 100.0%
FLIGHT-29-33 | FLIGHT-31-29 | 1.15       | 11200 | 🔸 | 100.0%
FLIGHT-29-31 | FLIGHT-35-31 | 1.55       | 11800 | 🔸 | 100.0%
FLIGHT-29-31 | FLIGHT-35-33 | 1.63       | 11800 | 🔸 | 100.0%
FLIGHT-34-29 | FLIGHT-35-30 | 1.31       | 12400 |    | 43.0%
FLIGHT-26-32 | FLIGHT-33-32 | 5.63       | 10000 | 🔸 | 85.0%
FLIGHT-30-29 | FLIGHT-35-34 | 6.54       | 11800 |    | 51.8%
FLIGHT-30-35 | FLIGHT-34-29 | 6.67       | 12400 |    | 49.0%
FLIGHT-30-35 | FLIGHT-35-30 | 6.54       | 12400 |    | 46.8%
---------------------------------------------------------------

[...]
```
