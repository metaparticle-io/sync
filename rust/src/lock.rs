// Copyright 2018 Christopher MacGown
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::fmt::Debug;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::{Duration};

use requests::{get, put, Error, StatusCode};


pub const DEFAULT_BASE_URI: &str = "http://localhost:8080";



/// Helper macro for invoking lock synchronization
///
/// # Example
///
/// ```
///
/// #[ macro_use ]
/// extern crate metaparticle_sync as sync;
///
///
/// fn main() {
///     let lock = lock!("some-lock");
///     lock.lock(|| {
///         // .. do some work
///     });
///
///     lock!("some-other-lock", || {
///         // .. do some work
///     });
/// }
/// ```
///
#[macro_export]
macro_rules! lock {
    ($name:tt) => ( $crate::Lock::new($name, $crate::DEFAULT_BASE_URI, 10); );
    ($name:tt, $lock_handler:expr) => {{
        let lock = lock!($name);
        lock.lock( $lock_handler );
    }}
}


pub trait MockableLockClient: Debug+Send+Sync {
    fn get_lock(&self, &str) -> Result<StatusCode, Error>;
    fn put_lock(&self, &str) -> Result<StatusCode, Error>;
}


struct Heartbeat{
    running: AtomicBool,
    wait_interval: u64,
}

impl Heartbeat {
    fn new(interval: u64) -> Self {
        Heartbeat{
            running: AtomicBool::new(false),
            wait_interval: interval,
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn beat<F>(&self, mut block: F)
    where F: FnMut() -> ()
    {
        self.start();
        while self.is_running() {
            // TODO - Add exponential backoff?
            sleep(Duration::from_millis(self.wait_interval));
            block();
        }
    }
}

#[derive(Debug)]
struct Client;
impl Client {
    fn new() -> Self {
        Client{}
    }
} 

impl MockableLockClient for Client {
    fn get_lock(&self, lock: &str) -> Result<StatusCode, Error> {
        match get(lock) {
            Ok(response) => Ok(response.status_code()),
            Err(error)   => Err(error),
        }
    }

    fn put_lock(&self, lock: &str) -> Result<StatusCode, Error> {
        match put(lock) {
            Ok(response) => Ok(response.status_code()),
            Err(error)   => Err(error),
        }
    }
}


/// Metaparticle.io Lock primitive.
///
/// As in the `lock!` macro example, you can create a lock directly using the
/// `Lock` primitive. In general, the 1-ary instance of the `lock!` macro is
/// preferable, however.
///
/// # Example (preferred)
/// ```
/// #[ macro_use ]
/// extern crate metaparticle_sync as sync;
///
/// fn main() {
///     let lock = lock!("some-held-lock");
///
///     // Attempt to lock without retrying.
///     lock.lock(|| {
///         // do some important work
///     });
///
///     // Block and attempt to grab the lock up to 10 times.
///     lock.lock_with_retry(|| {
///         // do some important work
///     });
///
///     // Block and attempt to grab the lock forever.
///     lock.lock_with_retry_forever(|| {
///         // do some important work
///     });
/// }
///
///
/// ```
///
/// # Example (raw primitives)
/// ```
/// extern crate metaparticle_sync as sync;
///
/// fn main() {
///     let interval = 10; // Heartbeat interval
///     let lock = sync::Lock::new("some-held-lock", sync::DEFAULT_BASE_URI, interval);
///
///     // Attempt to lock without retrying.
///     lock.lock(|| {
///         // do some important work
///     });
///
///     // Block and attempt to grab the lock up to 10 times.
///     lock.lock_with_retry(|| {
///         // do some important work
///     });
///
///     // Block and attempt to grab the lock forever.
///     lock.lock_with_retry_forever(|| {
///         // do some important work
///     });
/// }
/// ```
///
///
#[derive(Clone)]
pub struct Lock {
    name: String,
    base_uri: String,

    locked: Arc<AtomicBool>,
    pub (crate) client: Arc<MockableLockClient>,
    heartbeat: Arc<Heartbeat>,
}
impl Lock {
    pub fn new<S: Into<String>>(name: S, base_uri: S, interval: u64) -> Self {
        Lock{
            name: name.into(),
            base_uri: base_uri.into(),

            client: Arc::new(Client::new()),
            locked: Arc::new(AtomicBool::new(false)),
            heartbeat: Arc::new(Heartbeat::new(interval * 1000)),
        }
    }

    fn uri(&self) -> String {
        format!("{}/locks/{}", self.base_uri, self.name)
    }

    fn spin_heartbeat(&self, pair: Arc<(Mutex<bool>, Condvar)>) -> JoinHandle<()> {
        let uri = self.uri();
        let client = self.client.clone();
        let heartbeat = self.heartbeat.clone();

        spawn(move || {
            heartbeat.beat(|| {
                let &(ref lock, ref condition) = &*pair;

                match client.get_lock(&uri) {
                    Ok(status) => {
                        if status == StatusCode::Ok {
                            heartbeat.stop();
                            let mut available = lock.lock().unwrap();
                            *available = true;

                            condition.notify_one();
                        }
                    },
                    Err(reason) => {
                        error!("Could not get_lock: {}",
                               error: reason.to_string())
                    }
                }
            })
        })
    }

    fn hold_heartbeat(&self) -> JoinHandle<()> {
        let uri = self.uri();
        let client = self.client.clone();
        let locked = self.locked.clone();
        let heartbeat = self.heartbeat.clone();

        locked.store(true, Ordering::Relaxed);
        spawn(move || {
            heartbeat.beat(|| {
                match client.get_lock(&uri) {
                    Ok(status) => {
                        if status == StatusCode::Ok {
                            // Do we need to check the result here is ok?
                            let _ = client.put_lock(&uri);
                        } else {
                            heartbeat.stop();
                            locked.store(false, Ordering::Relaxed);
                        }
                    },
                    Err(err) => {
                        error!("Could not get lock {}: {}",
                               lock: uri,
                               error: err.to_string())
                    }
                }
            })
        })
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn unlock(&self) {
        self.heartbeat.stop();
        self.locked.store(false, Ordering::Relaxed);
    }

    pub fn lock<T: Fn() -> ()>(&self, func: T){
        self._lock(0, func);
    }

    pub fn lock_with_retry<T: Fn() -> ()>(&self, func: T){
        self._lock(10, func); // TODO - Should this be specified?
    }

    pub fn lock_with_retry_forever<T: Fn() -> ()>(&self, func: T) {
        self._lock(-1, func);
    }

    fn _lock<T: Fn() -> ()>(&self, retry: i8, func: T) {
        if self.is_locked() {
            error!("Locks are not reentrant {}", lock: self.name);
        }

        match self.client.get_lock(&self.uri()) {
            Ok(status) => {
                match status {
                    StatusCode::Ok | StatusCode::NotFound => {
                        match self.client.put_lock(&self.uri()) {
                            Ok(status) => {
                                match status {
                                    StatusCode::Ok => {
                                        let hold = self.hold_heartbeat();

                                        func();

                                        self.heartbeat.stop();
                                        let _ = hold.join(); // The handle output is unimportant
                                    },
                                    StatusCode::Conflict => {
                                        if retry == 0 {
                                            info!("Couldn't grab lock {} retry {}", lock: self.name, retry: retry);
                                            return
                                        }

                                        let pair = Arc::new((Mutex::new(false), Condvar::new()));
                                        let spin = self.spin_heartbeat(pair.clone());

                                        let &(ref lock, ref condition) = &*pair;
                                        let mut available = lock.lock().unwrap();
                                        loop {
                                            let r = condition.wait_timeout(available, Duration::from_millis(500))
                                                             .unwrap();

                                            available = r.0;
                                            if *available {
                                                break
                                            }
                                        }
                                        let _ = spin.join();

                                        let mut retry = retry;
                                        if retry != -1 {
                                            retry -= 1;
                                        }

                                        self._lock(retry, func);
                                    },
                                    _ => unreachable!(),
                                }
                            },
                            Err(err) => {
                                error!("Could not put lock {}: {}",
                                       lock: self.uri(),
                                       error: err.to_string())
                            }
                        }
                    },
                    _ => {
                        unreachable!();
                    },
                }
            },
            Err(err) => {
                error!("Could not get lock {}: {}",
                       lock: self.uri(),
                       error: err.to_string())
            }
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate requests;

    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use requests::{StatusCode, Error};

    use lock::{Lock, MockableLockClient};

    #[derive(Debug,Clone)]
    struct MockLock((String, Instant));

    #[derive(Debug,Clone)]
    struct MockLockServer(Arc<Mutex<HashMap<String, MockLock>>>);
    impl MockLockServer {
        fn new() -> Self {
            MockLockServer(Arc::new(Mutex::new(HashMap::new())))
        }
    }

    #[derive(Debug,Clone)]
    struct MockClient((String, MockLockServer));
    impl MockClient {
        fn new<T: Into<String>>(client: T, server: MockLockServer) -> Self {
            MockClient((client.into(), server))
        }

        fn new_lock<T: Into<String>>(&self, lock: T, base_uri: T) -> Lock {
            let mut lock = Lock::new(lock, base_uri, 1);
            let client = Arc::new(self.clone());
            lock.client = client;
            lock
        }
    }

    impl MockableLockClient for MockClient {
        fn get_lock(&self, lock: &str) -> Result<StatusCode, Error> {
            let &MockClient((_, ref mutex)) = self;
            let locks = mutex.0.lock().unwrap();

            if locks.contains_key(lock) {
                return Ok(StatusCode::Ok);
            }
            Ok(StatusCode::NotFound)
        }

        fn put_lock(&self, lock: &str) -> Result<StatusCode, Error> {
            let &MockClient((ref client, ref mutex)) = self;
            let mut locks = mutex.0.lock().unwrap();

            match locks.get(lock) {
                Some(mocklock) => {
                    let &MockLock((ref lockclient, ref timeout)) = mocklock;
                    let elapsed = Instant::now().duration_since(*timeout);

                    if lockclient != &*client && elapsed < Duration::new(1, 0) {
                        return Ok(StatusCode::Conflict)
                    }
                },
                _ => {},
            }
            locks.insert(lock.to_string(), MockLock((client.to_string(), Instant::now())));
            Ok(StatusCode::Ok)
        }
    }

    #[test]
    fn test_locking_with_macros() {
        lock!("macro-lock", || { sleep(Duration::from_millis(250)) })
    }

    #[test]
    fn test_locking_without_retrying() {
        let server = MockLockServer::new();
        let client1 = MockClient::new("client1", server.clone());
        let client2 = MockClient::new("client2", server.clone());
        
        let lock  = client1.new_lock("good", "localhost:8080");
        let lock2 = client2.new_lock("good", "localhost:8080");

        lock.lock(|| {
            println!("DOING THE WORK");
            sleep(Duration::from_millis(250));
            println!("DONE!");
        });
        lock2.lock(|| {
            println!("DOING THE WORK");
            sleep(Duration::from_millis(250));
            println!("DONE!");
        });

        assert_eq!(lock.is_locked(), true);
        assert_eq!(lock2.is_locked(), false);
    }

    #[test]
    fn test_locking_with_retrying() {
        let server = MockLockServer::new();
        let client1 = MockClient::new("client1", server.clone());
        let client2 = MockClient::new("client2", server.clone());
        
        let lock  = client1.new_lock("good", "localhost:8080");
        let lock2 = client2.new_lock("good", "localhost:8080");

        lock.lock_with_retry(|| {
            println!("DOING THE WORK");
            sleep(Duration::from_millis(250));
            println!("DONE!");
        });
        lock2.lock_with_retry(|| {
            println!("DOING THE WORK");
            sleep(Duration::from_millis(250));
            println!("DONE!");
        });

        assert_eq!(lock.is_locked(), true);
        assert_eq!(lock2.is_locked(), true);
    }
}
