use super::{ChannelMessage, Error};

use std::{
    sync::mpsc::{Sender, Receiver, channel}
};
pub struct Supervisor {
    incoming: Receiver<ChannelMessage>,
    flip_ch: Sender<ChannelMessage>,
}

impl Supervisor {
    pub fn new() -> (Self, Sender<ChannelMessage>, Receiver<ChannelMessage>) {
        let (flip_ch, flip_rx) = channel();
        let (tx, incoming) = channel();
        let ret = Self {
            incoming,
            flip_ch,
        };
        (ret, tx, flip_rx)
    }
    pub fn run(self) -> Result<(), Error> {
        loop {
            let msg = self.incoming.recv()?;
            info!(target: "robohome", "{}", msg);
            match msg {
                ChannelMessage::Tick => self.flip_ch.send(ChannelMessage::FlipperCheck)?,
                ChannelMessage::MqUpdateFlip => self.flip_ch.send(ChannelMessage::FlipperRefresh)?,
                ChannelMessage::Error(msg) => return Err(Error::Other(msg)),
                _ => (),
            }
        }
    }
}