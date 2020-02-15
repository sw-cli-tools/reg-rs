#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use std::env;

lazy_static! {
    static ref DEBUG: bool = env::args().any(|s| s.starts_with("-d"));
}
macro_rules! md {
    // maybe-debug
    ( $e:expr ) => {
        if *super::DEBUG {
            dbg!($e);
        }
    };
}

pub mod args;
pub mod builder;
pub mod config;
pub mod db;
pub mod process;
pub mod report;
pub mod runner;
pub mod sqlite;
pub mod time;
