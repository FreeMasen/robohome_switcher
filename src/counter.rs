use super::{ChannelMessage, Error,};
use std::{
    sync::mpsc::Sender,
    time::Duration,
    thread::sleep,
};
pub struct Counter {
    sender: Sender<ChannelMessage>,
}

impl Counter {
    pub fn new(sender: Sender<ChannelMessage>) -> Self {
        Self {
            sender,
        }
    }
    pub fn run(self) -> Result<(), Error> {
        loop {
            self.sender.send(ChannelMessage::Tick)?;
            let sleep_for = Duration::from_millis(60 * 1000 as u64);
            sleep(sleep_for);
        }
    }
}