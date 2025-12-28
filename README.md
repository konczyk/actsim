# ADS-B Filter 

Tiny ADS-B filter

## Usage

Build project
```shell
cargo build -r
```

Run tests
```shell
cargo test
```

Execute
```shell
$ ./target/release/adsb_filter -h
Usage: adsb_filter [OPTIONS]

Options:
      --size <SIZE>      Size of the buffer in bits [default: 1024]
      --hashes <HASHES>  Number of hash functions to run in the filter [default: 4]
  -h, --help             Print help
```

## Examples

Run the filter against a test data stream
```shell
$ ./adsb_gen.py | ./target/release/tinyadsb --size 128 --hashes 8
NEW:    0A423C
NEW:    4B3B46
NEW:    191EA9
MATCH:  191EA9 (Est. FPR: 0.0002%)
NEW:    02EF42
NEW:    0F7FF6
MATCH:  4B3B46 (Est. FPR: 0.0049%)
NEW:    E41E54
MATCH:  0A423C (Est. FPR: 0.0162%)
[...]
```