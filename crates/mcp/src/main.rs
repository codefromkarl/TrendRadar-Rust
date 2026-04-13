//! trendradar-mcp 二进制入口。

use std::io::{self, Read};
use std::process;

fn main() {
    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {error}");
        process::exit(1);
    }

    match trendradar_mcp::handle_request(&input) {
        Ok(response) => println!("{response}"),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
