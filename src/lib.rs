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
        if *crate::DEBUG {
            dbg!($e);
        }
    };
}

pub mod args;
pub mod builder;
pub mod command;
pub mod config;
pub mod db;
pub mod diff;
pub mod finder;
pub mod process;
pub mod queries;
pub mod reporters;
pub mod runner;
pub mod sqlite;
pub mod templates;
pub mod time;
pub mod util;
