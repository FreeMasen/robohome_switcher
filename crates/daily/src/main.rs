extern crate env_logger;
#[macro_use]
extern crate log;

extern crate robohome_shared;

use env_logger::{Builder, Target};

use robohome_shared::{data::*, error::Error};

fn main() -> Result<(), Error> {
    init_logging();
    let (sunrise, sunset) = get_daily_info()?;
    let ct = save_daily_info(sunrise, sunset)?;
    info!(target: "robohome:info", "saved daily info with {} times", ct);
    Ok(())
}

fn init_logging() {
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    if let Ok(arg) = ::std::env::var("RUST_LOG") {
        builder.parse(&arg);
    }
    builder.init();
}