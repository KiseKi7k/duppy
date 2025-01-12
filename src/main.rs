use std::time::Instant;
use std::env;
use project_2::find_duplicate_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let now = Instant::now();

    find_duplicate_file(r"H:\Download 2\ภาพ").unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}
