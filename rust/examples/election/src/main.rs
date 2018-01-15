#[macro_use]
extern crate metaparticle_sync;


use std::thread::sleep;
use std::time::Duration;

fn main() {
    let election = elect!("example-election",
                          || {
                              println!("I am the leader!");
                              sleep(Duration::from_millis(250));
                          },
                          || {
                              println!("I am the follower!");
                              sleep(Duration::from_millis(250));
                          });
    election.run();
}
