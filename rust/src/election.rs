// Copyright 2018 Christopher MacGown
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use lock;


/// Helper macro for invoking election synchronization
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate metaparticle_sync;
///
/// #[derive(Copy,Clone)]
/// struct DatabaseMigrator;
/// impl DatabaseMigrator {
///     pub fn new() -> Self {
///         DatabaseMigrator{}
///     }
///
///     pub fn migrate(&self) {
///         // ... long-lasting work
///     }
///
///     pub fn watch(&self) {
///         // ... keep an eye on it.
///     }
/// }
///
/// fn main() {
///     let migrator = DatabaseMigrator::new();
///     elect!("database-migration",
///            || migrator.migrate(),   // This closure is invoked when leader
///            || migrator.watch());    // This closure is invoked when follower
/// }
/// ```
///
#[macro_export]
macro_rules! elect {
    ($name: tt) => ( $crate::Election::new($name,
                                           $crate::DEFAULT_BASE_URI,
                                           Box::new(|| {}),
                                           Box::new(|| {}))
    );
    ($name: tt, $leaderfn:expr, $followerfn:expr) => {{
        let mut election = elect!($name);
        election.add_handler($crate::Handler::Leader  , Box::new($leaderfn));
        election.add_handler($crate::Handler::Follower, Box::new($followerfn));
        election
    }};
    ($name: tt, $client:tt) => {{
        use std::sync::Arc;

        let mut election = elect!($name);
        election.lock.client = Arc::new($client);
        election
    }};
    ($name: tt, $client:tt, $leaderfn:expr, $followerfn:expr) => {{
        use std::sync::Arc;

        let mut election = elect!($name);
        election.add_handler($crate::Handler::Leader  , Box::new($leaderfn));
        election.add_handler($crate::Handler::Follower, Box::new($followerfn));
        election.lock.client = Arc::new($client);
        election
    }};
}


/// Metaparticle.io Election primitive.
///
/// As in the `elect!` macro example, you can create an election directly using
/// the `Election` primitive. It's preferred to use the macro, however.
///
/// # Example
///
/// ```
/// extern crate metaparticle_sync as sync;
///
/// #[derive(Copy,Clone)]
/// struct DatabaseMigrator;
/// impl DatabaseMigrator {
///     pub fn new() -> Self {
///         DatabaseMigrator{}
///     }
///
///     pub fn migrate(&self) {
///         // ... long-lasting work
///     }
///
///     pub fn watch(&self) {
///         // ... keep an eye on it.
///     }
/// }
///
/// fn main() {
///     let migrator = DatabaseMigrator::new();
///
///     // Election setup.
///     let mut elector  = sync::Election::new("database-migration",
///                                            sync::DEFAULT_BASE_URI,
///                                            Box::new(||{}),
///                                            Box::new(||{}));
///
///     elector.add_handler(sync::Handler::Leader  , Box::new(|| migrator.migrate()));
///     elector.add_handler(sync::Handler::Follower, Box::new(|| migrator.watch()));
///
///     // ... Do other stuff
///
///     elector.run();
///
/// }
/// ```

#[derive(Clone)]
pub struct Election<'a> {
    lock: lock::Lock,
    running: Arc<AtomicBool>,

    leader_fn: Arc<Box<Fn() -> () + Send + Sync + 'a>>,
    follower_fn: Arc<Box<Fn() -> () + Send + Sync + 'a>>,
}

pub enum Handler {
    Leader,
    Follower,
}

impl<'a> Election<'a> {
    pub fn new<T: Into<String>>(name: T, base_uri: T,
                                leader_fn: Box<Fn() -> () + Send + Sync + 'a>,
                                follower_fn: Box<Fn() -> () + Send + Sync + 'a>) -> Self {
        Election{
            lock: lock::Lock::new(name, base_uri, 10),
            running: Arc::new(AtomicBool::new(false)),
            leader_fn: Arc::new(leader_fn),
            follower_fn: Arc::new(follower_fn),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn run(&self) {
        let leader_fn = self.leader_fn.clone();
        let follower_fn = self.follower_fn.clone();
        self.running.store(true, Ordering::Relaxed);
        if self.is_running() {
            self.lock.lock(|| leader_fn());

            follower_fn();
        }
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn add_handler(&mut self, typ: Handler, handler: Box<Fn() -> () + Send + Sync + 'a>) {
        match typ {
            Handler::Leader   => self.leader_fn = Arc::new(handler),
            Handler::Follower => self.follower_fn = Arc::new(handler),

        };
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread::{self, sleep};
    use std::time::{Duration, Instant};

    use requests::{StatusCode, Error};

    use lock::MockableLockClient;

    use election;

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

    impl PartialEq for MockClient {
        fn eq(&self, rhs: &Self) -> bool {
            (self.0).0 == (rhs.0).0
        }
    }

    impl MockClient {
        fn new<T: Into<String>>(client: T, server: MockLockServer) -> Self {
            MockClient((client.into(), server))
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
    fn test_run_election_with_add_handler() {
        let server = MockLockServer::new();
        let client0 = MockClient::new("client0", server.clone());
        let client1 = MockClient::new("client1", server.clone());
        let client2 = MockClient::new("client2", server.clone());

        let expected = client0.clone();

        let leading: Arc<Mutex<Option<MockClient>>> = Arc::new(Mutex::new(None));
        let mut mutexes: Vec<Arc<Mutex<Option<MockClient>>>> = (0..3).map(|_| leading.clone())
                                                                     .collect();

        let clients = vec![client0, client1, client2];

        for client in clients.into_iter() {
            let mutex = mutexes.remove(0);
            let clone = client.clone();

            let mut elector = elect!("fake-lock", clone);
            elector.add_handler(election::Handler::Leader,
                                Box::new(move || {
                                     *(mutex.lock()).unwrap() = Some(client.clone());
                                     sleep(Duration::from_millis(250));
                                }));
            elector.add_handler(election::Handler::Follower,
                                Box::new(move || {
                                     sleep(Duration::from_millis(250));
                                }));

            thread::spawn(move || { elector.run() });
        }

        sleep(Duration::from_millis(500));
        assert_eq!(*leading.lock().unwrap(), Some(expected));
    }

    #[test]
    fn test_run_election() {
        let server  = MockLockServer::new();
        let client0 = MockClient::new("client0", server.clone());
        let client1 = MockClient::new("client1", server.clone());
        let client2 = MockClient::new("client2", server.clone());

        let expected = client0.clone();

        let leading: Arc<Mutex<Option<MockClient>>> = Arc::new(Mutex::new(None));
        let mut mutexes: Vec<Arc<Mutex<Option<MockClient>>>> = (0..3).map(|_| leading.clone())
                                                                     .collect();

        let clients = vec![client0, client1, client2];

        for client in clients.into_iter() {
            let mutex = mutexes.remove(0);
            let clone = client.clone();
            let elector = elect!("fake-lock", clone,
                                 move || {
                                     *(mutex.lock()).unwrap() = Some(client.clone());
                                     sleep(Duration::from_millis(250));
                                 },
                                 move || {
                                     sleep(Duration::from_millis(250));
                                 });
            thread::spawn(move || { elector.run() });
        }

        sleep(Duration::from_millis(500));
        assert_eq!(*leading.lock().unwrap(), Some(expected));
    }
}
