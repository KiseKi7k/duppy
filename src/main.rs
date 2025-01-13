mod duppy;

use std::time::Instant;
use std::env;
use duppy::run;

fn build<'a>(args: &'a Vec<String>) -> Result<Vec<&'a str>, &'static str> {

    if args.len() != 2{
        return Err("Arguments error: expected 2")
    }

    let config: Vec<&str> = args[1].split("||").collect();

    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config = build(&args)?;
    println!("{:?}", args[1]);
    println!("{:?}", config);

    let now = Instant::now();

    //let _path = r"H:\Download 2\ภาพ||H:\Pics";
    //let paths: Vec<&str> = _path.split("||").collect();

    run(config).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}
