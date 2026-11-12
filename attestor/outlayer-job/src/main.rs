//! OutLayer WASI entrypoint. Stdin is the public JSON input. `HL_ADDRESS` comes from the environment.
//! This process never prints that address or the raw fills.
use outlayer_job::{execute, RejectingFetch};
use std::io::Read;

fn main() {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let address = std::env::var("HL_ADDRESS").ok();
    match execute(&input, address.as_deref(), &RejectingFetch) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
