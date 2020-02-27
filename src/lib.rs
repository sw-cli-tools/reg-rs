//#[macro_use]
//extern crate clap;
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

pub const DEFAULT_STATUS_PORT: u16 = 4111;
pub const STATUS_BANNER: &str = "RTT1 Status Server";

pub mod args;
pub mod builder;
pub mod command;
pub mod config;
pub mod db;
pub mod diff;
pub mod finder;
pub mod logging;
pub mod process;
pub mod queries;
pub mod reporters;
pub mod runner;
pub mod sqlite;
pub mod status;
pub mod templates;
pub mod time;
pub mod util;
