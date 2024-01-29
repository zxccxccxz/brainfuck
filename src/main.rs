use std::{process, env};
use std::time::Instant;
use brainfuck::BFI;

fn main() {
    let start_time = Instant::now();

    let mut bfi = BFI::new();
    if let Err(err) = bfi.build(env::args()) {
        eprintln!("Build error: {err}");
        process::exit(1);
    }
    if let Err(err) = bfi.run() {
        eprintln!("Error while executing program: {err}");
        process::exit(1);
    }

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
}
