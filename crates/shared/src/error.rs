#[cfg(feature = "web")]
use reqwest::Error as RError;
use postgres::Error as PError;
use amqp::AMQPError as AError;
use serde_json::Error as JError;

use std::sync::mpsc::{RecvError, SendError};

use super::message::ChannelMessage;

#[derive(Debug)]
pub enum Error {
    Db(PError),
    Req(String),
    Rabbit(AError),
    Json(JError),
    Send(SendError<ChannelMessage>),
    Rec(RecvError),
    Enum(String, i32),
    Other(String),
}

impl Error {
    pub fn other(msg: &str) -> Self {
        Error::Other(msg.to_owned())
    }
    pub fn _enum(name: &str, i: i32) -> Self {
        Error::Enum(name.to_owned(), i)
    }
}

impl ::std::error::Error for Error {
    fn cause(&self) -> Option<&::std::error::Error> {
        match self {
            Error::Db(ref p) => Some(p),
            Error::Rabbit(ref a) => Some(a),
            Error::Json(ref j) => Some(j),
            Error::Send(ref s) => Some(s),
            Error::Rec(ref r) => Some(r),
            _ => None
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Error::Db(e) => write!(f, "DB Error\n{}", e),
            Error::Req(e) => write!(f, "Web Request Error\n{}", e),
            Error::Rabbit(e) => write!(f, "RabbitMQ Error\n{}", e),
            Error::Json(e) => write!(f, "JSON Serialization Error\n{}", e),
            Error::Send(e) => write!(f, "MCSP Channel Send Error\n{}", e),
            Error::Rec(e) => write!(f, "MCSP Channel Recv Error\n{}", e),
            Error::Enum(name, idx) => write!(f, "Attempt to construct {} failed with {}, out of bounds", idx, name),
            Error::Other(s) => write!(f, "Unknown Error\n{}", s),
        }
    }
}

impl From<PError> for Error {
    fn from(other: PError) -> Self {
        Error::Db(other)
    }
}
#[cfg(feature = "web")]
impl From<RError> for Error {
    fn from(other: RError) -> Self {
        Error::Req(format!("{}", other))
    }
}

impl From<AError> for Error {
    fn from(other: AError) -> Self {
        Error::Rabbit(other)
    }
}

impl From<JError> for Error {
    fn from(other: JError) -> Self {
        Error::Json(other)
    }
}

impl From<SendError<ChannelMessage>> for Error {
    fn from(other: SendError<ChannelMessage>) -> Self {
        Error::Send(other)
    }
}

impl From<RecvError> for Error {
    fn from(other: RecvError) -> Self {
        Error::Rec(other)
    }
}