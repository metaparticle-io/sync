#[macro_use]
extern crate metaparticle_sync;


use std::thread::sleep;
use std::time::Duration;

fn main() {
    // This will retry ten times before failing.
    lock!("example-lock", 
          || {
              println!("Just an example lock");
              sleep(Duration::from_millis(250))
          });
}
