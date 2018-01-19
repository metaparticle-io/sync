# Metaparticle/Sync rust library

Metaparticle/Sync for rust is a library that implements distributed synchronization for
cloud-native applications using a container sidecar and Kubernetes primitives.


## Quickstart

Add the following to your `Cargo.toml`

```
[dependencies]
metaparticle_sync = "0.1"
```

## metaparticle_sync in action

```
#[macro_use]
extern crate metaparticle_sync as sync;


fn main() {
    let lock = lock!("some-lock");

    // Attempt to lock without retrying.
    lock.lock(|| {
        // do some important work
    });

    // Block and attempt to grab the lock up to 10 times.
    lock.lock_with_retry(|| {
        // do some important work
    });

    // Block and attempt to grab the lock forever.
    lock.lock_with_retry_forever(|| {
        // do some important work
    });

    // Election
    let election = elect!("some-election", 
                          || {
                            // Do some work when leader.
                          },
                          || {
                            // Do some work when follower.
                          });
    election.run();
}
```




## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
