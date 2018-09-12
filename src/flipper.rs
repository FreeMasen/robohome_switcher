use super::{yesterday, ChannelMessage, Error};
use data::{Flip, get_flips};
use mq::send;

use std::{
    sync::mpsc::{Sender, Receiver}
};

use chrono::{DateTime, Date, Local};

pub struct Flipper {
    flips: Vec<Flip>,
    current_date: Date<Local>,
    tx: Sender<ChannelMessage>,
    rx: Receiver<ChannelMessage>,
}
impl Flipper {
    pub fn new(tx: Sender<ChannelMessage>, rx: Receiver<ChannelMessage>) -> Self {
        Self {
            flips: vec![],
            current_date: yesterday().date(),
            tx,
            rx,
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        loop {
            let msg = self.rx.recv()?;
            info!(target: "robohome", "{}", msg);
            match msg {
                ChannelMessage::FlipperCheck => {
                    if self.is_out_of_date() {
                        let _ = self.tx.send(ChannelMessage::FlipperOutOfDate);
                        self.get_today()?;
                        self.tx.send(ChannelMessage::FlipperUpdated)?;
                    }
                    self.send()?;
                    self.tx.send(ChannelMessage::FlipperComplete)?;
                },
                ChannelMessage::FlipperRefresh => {
                    self.get_today()?;
                    self.prune_today();
                    self.tx.send(ChannelMessage::FlipperUpdated)?;
                },
                _ => (),
            }
        }
    }

    pub fn is_out_of_date(&self) -> bool {
        self.current_date != Local::today()
    }

    pub fn get_today(&mut self) -> Result<(), Error> {
        self.flips = get_flips()?;
        self.current_date = Local::today();
        Ok(())
    }

    pub fn prune_today(&mut self) {
        let now = Local::now();
        loop {
            let pop = if let Some(ref flip) = self.flips.last() {
                flip.time.lte(&now)
            } else {
                false
            };
            if pop {
                let _ = self.flips.pop();
            }
        }
    }
    pub fn send(&mut self) -> Result<(), Error> {
        let now = Local::now();
        while self.ready_to_send(&now) {
            let last = self.flips.pop().ok_or(Error::Other("Expected flip to exist".to_owned()))?;
            send(last.remote_id, last.switch_id, last.direction)?;
        }
        Ok(())
    }

    fn ready_to_send(&self, now: &DateTime<Local>) -> bool {
        if let Some(last) = self.flips.last() {
            last.time.lte(now)
        } else {
            false
        }
    }
}