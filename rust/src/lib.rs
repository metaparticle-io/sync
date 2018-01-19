// Copyright 2018 Christopher MacGown
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! # Metaparticle/Sync
//!
//! `metaparticle_sync` is a library for synchronization across multiple
//! containers running on different machines.
//!
//!
//!


#[macro_use]
extern crate emit;
extern crate requests;

mod election;
mod lock;

pub use self::election::{Election, Handler};
pub use self::lock::{Lock, DEFAULT_BASE_URI};
