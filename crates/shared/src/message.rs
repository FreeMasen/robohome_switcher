
#[derive(Clone, Debug)]
pub enum ChannelMessage {
    FlipperCheck,
    FlipperRefresh,
    FlipperOutOfDate,
    FlipperComplete,
    FlipperUpdated,
    MqUpdateFlip,
    Error(String),
    Stop,
    Tick,
}

impl ::std::fmt::Display for ChannelMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ChannelMessage::FlipperCheck => write!(f, "FL OUT FlipperCheck"),
            ChannelMessage::FlipperRefresh => write!(f, "FL OUT FlipperRefresh"),
            ChannelMessage::FlipperOutOfDate => write!(f, "FL IN FlipperOutOfDate"),
            ChannelMessage::FlipperComplete => write!(f, "FL IN FlipperComplete"),
            ChannelMessage::FlipperUpdated => write!(f, "FL IN FlipperUpdated"),
            ChannelMessage::MqUpdateFlip => write!(f, "MQ IN MqUpdateFlip"),
            ChannelMessage::Error(msg) => write!(f, "?? IN MqError: {}", msg),
            ChannelMessage::Stop => write!(f, "?? OUT Stop"),
            ChannelMessage::Tick => write!(f, "CT IN Tick"),
        }
    }
}