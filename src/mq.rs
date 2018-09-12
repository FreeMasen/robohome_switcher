use amqp::{Session, Table, Basic,
            protocol::basic::{BasicProperties, Deliver},
            Consumer, Channel};
use std::{
    default::Default,
    sync::mpsc::Sender,
};
use data::SwitchState;
use serde_json::to_vec;
use super::{
    CONFIG,
    Error,
    ChannelMessage
};

pub fn send(remote_id: i32, switch_id: i32, direction: SwitchState) -> Result<(), Error> {
    let direction = direction.for_db();
    let msg = Message {
        switch_id: switch_id as u16,
        direction: direction as u8,
    };
    let msg = to_vec(&msg)?;
    let mut sess = get_session()?;
    let mut ch = sess.open_channel(1)?;
    let queue_name = "switches";
    let binding_key = remote_id.to_string();
    let _ex = ch.exchange_declare(queue_name, "topic", false, false, false, false, false, Table::new())?;
    let _queue = ch.queue_declare(queue_name, false, false, false, false, false, Table::new())?;
    let _bind = ch.queue_bind(queue_name, queue_name, &binding_key, false, Table::new())?;
    let props: BasicProperties = Default::default();
    ch.basic_publish(queue_name, &binding_key, true, false, props, msg)?;
    Ok(())
}

fn get_session() -> Result<Session, Error> {
    let s = Session::new((&CONFIG.mq_config).into())?;
    Ok(s)
}

pub fn listen(sender: Sender<ChannelMessage>) -> Result<(), Error> {
    let l = MqListener::new(sender);
    let mut session = get_session()?;
    let mut ch = session.open_channel(2)?;
    let exchange_name = "switches";
    let queue_name = "refresh";
    let _ex = ch.exchange_declare(exchange_name, "topic", false, false, false, false, false, Table::new())?;
    let _queue_decl = ch.queue_declare(queue_name, false, false, false, false, false, Table::new())?;
    let _bind = ch.queue_bind(queue_name, exchange_name, "update", false, Table::new())?;
    ch.basic_prefetch(10)?;
    let _consumer_name = ch.basic_consume(l, queue_name, "update", false, false, false, false, Table::new());
    ch.start_consuming();
    Ok(())
}

pub struct MqListener {
    sender: Sender<ChannelMessage>,
}

impl MqListener {
    pub fn new(sender: Sender<ChannelMessage>) -> Self {
        Self {
            sender,
        }
    }

    fn send_error(&mut self, msg: &str) {
        let msg = ChannelMessage::Error(msg.to_owned());
        match self.sender.send(msg) {
            Err(e) => {
                eprintln!("Catastrophic error when sending msg\n{}", e);
            },
            _ => (),
        }
    }

    fn send_reset(&mut self) {
        match self.sender.send(ChannelMessage::MqUpdateFlip) {
            Err(e) => eprintln!("Catastrophic error when sending msg\n{}", e),
            _ => (),
        }
    }
}

impl Consumer for MqListener {
    fn handle_delivery(&mut self, ch: &mut Channel, method: Deliver, _: BasicProperties, body: Vec<u8>) {
        if let Ok(ref msg) = String::from_utf8(body) {
            info!(target: "robohome", "new mq message {}",msg);
            if msg == "update" {
                self.send_reset();
            } else {
                self.send_error(&format!("Unknown message content from MQ router {}", msg));
            }
        } else {
            self.send_error("failed to decode utf-8")
        }
        match ch.basic_ack(method.delivery_tag, false) {
            Ok(_) => (),
            Err(e) => self.send_error(&format!("Unable to send ack to MQ router\n{}", e)),
        }
    }
}




#[derive(Deserialize, Serialize)]
pub struct Message {
    switch_id: u16,
    direction: u8,
}
