#[macro_use]
extern crate metaparticle_sync as sync;

#[macro_use]
extern crate emit;

use std::thread;
use std::time;

use emit::PipelineBuilder;
use emit::collectors::stdio::StdioCollector;
use emit::formatters::raw::RawFormatter;

fn main() {
    let _flush = PipelineBuilder::new()
        .write_to(StdioCollector::new(RawFormatter::new()))
        .init();

    let lock = lock!("hello-world");
    lock.lock(|| {
        let duration = time::Duration::new(1,0);
        info!("Sleeping! {}", duration: duration.as_secs());
        thread::sleep(time::Duration::new(1,0));
        info!("Waking up! {}", lock: "some-lock")
    });

    lock!("hello-world", { 
        let duration = time::Duration::new(1,0);
        info!("Sleeping! {}", duration: duration.as_secs());
        thread::sleep(time::Duration::new(1,0));
        info!("Waking up! {}", lock: "some-lock")
    });
}
