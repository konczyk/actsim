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
$ ./target/release/adsb_filter -h
Usage: adsb_filter [OPTIONS]

Options:
      --max-age <MAX_AGE>  Max age in seconds for a filter before it gets removed [default: 300]
  -h, --help               Print help
```

## Examples

Run the filter against a test data stream
```shell
$ ./adsb_gen.py | ./target/release/adsb_filter 
NEW:    05A48E
NEW:    A91DDD
NEW:    31A689
NEW:    0689BA
NEW:    504DD1
MATCH:  A91DDD (Est. FPR: 1.5625%)
NEW:    105C37
NEW:    560784
MATCH:  05A48E (Est. FPR: 1.5625%)
MATCH:  0689BA (Est. FPR: 1.5625%)
NEW:    3B36BD
NEW:    8F2BDD
MATCH:  560784 (Est. FPR: 1.5625%)
NEW:    5C893C
[...]
```