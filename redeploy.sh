#! /bin/bash
cargo build --release --package robohome_switcher
nohup ./target/release/robohome_switcher > ~/logs/robohome_switcher.log &