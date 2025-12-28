use std::io;
use std::io::BufRead;

mod bloom_filter;

fn main() -> io::Result<()>{
    let mut bf = bloom_filter::BloomFilter::new(1024, 4);

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();

    while handle.read_line(&mut buf)? > 0 {
        let line = buf.trim();
        if line.is_empty() {
            buf.clear();
            continue;
        }

        if bf.contains(&line) {
            println!("MATCH:\t{} (Est. FPR: {:.4}%)", line, bf.fpr() * 100.0);
        } else {
            bf.insert(&line);
            println!("NEW:\t{}", line);
        }

        buf.clear();
    }

    Ok(())
}
