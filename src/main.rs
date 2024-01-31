use std::time::Instant;
use brainfuck::BFI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let code = brainfuck::read_file_from_args(std::env::args())?;
    let program= BFI::build(code)?;
    BFI::run(program)?;

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
    Ok(())
}
