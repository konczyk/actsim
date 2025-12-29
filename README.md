# ADS-B Filter 

Tiny, scalable ADS-B filter with a near constant false positive ratio and pruning of old data.

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
$ ./target/release/adsb-filter -h
Usage: adsb-filter [OPTIONS]

Options:
      --max-age <MAX_AGE>  Max age in seconds for a filter before pruning [default: 300]
  -h, --help               Print help
```

## Examples

Run the filter against a test data stream
```shell
$ ./tools/adsb_gen.py | ./target/release/adsb-filter
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