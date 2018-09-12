extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate amqp;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate robohome_shared;

use chrono::{DateTime, Local, Duration};

mod counter;
mod flipper;
mod mq;
mod supervisor;

use counter::Counter;
use flipper::Flipper;
use supervisor::Supervisor;

use robohome_shared::{data, message::ChannelMessage, error::Error, CONFIG};

fn main() -> Result<(), Error> {
    init_logging();
    let (boss, tx, flip_rx) = Supervisor::new();
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    let _fl_handle = ::std::thread::Builder::new().name("Flipper".to_owned()).spawn(move || {
        let f = Flipper::new(tx1, flip_rx);
        if let Err(e) = f.run() {
            error!(target: "robohome", "Exiting flipper thread with error\n{}", e);
        } else {
            info!(target: "robohome", "Exiting flipper thread")
        }
    });
    let _mq_handle = ::std::thread::Builder::new().name("MQ".to_owned()).spawn(move || {
        if let Err(e) = mq::listen(tx2) {
            error!(target: "robohome", "Exiting mq thread with error\n{}", e);
        } else {
            info!(target: "robohome", "Exiting mq thread");
        }
    });
    let _count_handle = ::std::thread::Builder::new().name("Counter".to_owned()).spawn(move || {
        let c = Counter::new(tx3);
        if let Err(e) = c.run() {
            error!(target: "robohome", "Exiting counter thread with error\n{}", e);
        } else {
            info!(target: "robohome", "Exiting counter thread");
        }
    });
    boss.run()
}

pub fn yesterday() -> DateTime<Local> {
    let today = Local::now();
    let day = Duration::days(1);
    today - day
}

use env_logger::{Builder, Target};

fn init_logging() {
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    if let Ok(arg) = ::std::env::var("RUST_LOG") {
        builder.parse(&arg);
    }
    builder.init();
}