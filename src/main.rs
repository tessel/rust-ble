#![feature(alloc_system)]

extern crate alloc_system;
extern crate ble;

use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting scan...");

    let mut scan = ble::scan().unwrap();

    // Comment out these lines to let scan go on forever.
    thread::sleep(Duration::from_millis(3000));
    scan.stop();

    println!("Results:");
    for discovery in scan {
        println!("{:?}", discovery);
    }

    println!("... done.");
}
