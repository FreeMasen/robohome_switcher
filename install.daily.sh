echo building updater
cargo build --release --package robohome_daily_updater
echo moving updater
scp ./target/release/robohome_daily_updater rfm@192.168.0.199:~/scripts/
echo copying update script
scp ./robohome_daily rfm@192.168.0.199:~/scripts/