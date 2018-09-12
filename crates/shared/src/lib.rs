extern crate amqp;
extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate postgres;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate uuid;
use toml::from_str;

lazy_static! {
    pub static ref CONFIG: Config = from_str(include_str!("../config.toml")).expect("Unable to deserialize config.toml");
}

pub mod data;
pub mod error;
pub mod message;

#[derive(Deserialize)]
pub struct Config {
    pub db_conn_str: String,
    pub weather_uri: String,
    weather_attempts: usize,
    pub mq_config: MqConfig,
    pub log_arg: Option<String>,
}

#[derive(Deserialize)]
pub struct MqConfig {
    host: String,
    port: u16,
    login: String,
    password: String,
}

impl<'a> Into<amqp::Options> for &'a MqConfig {
    fn into(self) -> amqp::Options {
        amqp::Options {
            host: self.host.clone(),
            port: self.port,
            login: self.login.clone(),
            password: self.password.clone(),
            vhost: String::from("/"),
            ..::std::default::Default::default()
        }
    }
}