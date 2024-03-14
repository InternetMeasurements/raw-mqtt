use crate::utility::argument_parser::{PublishArgs, SubscribeArgs};
use clap::Parser;

const DEFAULT_RATE: f64 = 0.0;
const DEFAULT_DURATION: usize = 10;
const DEFAULT_QUEUE: i64 = 1024;
const DEFAULT_NAGLE_OFF: bool = false;

#[cfg(feature = "pub_stream")]
#[derive(Parser)]
#[command(name = "mqtt-client")]
#[command(bin_name = "mqtt-client")]
pub enum MqttStreamCli {
    #[clap(alias = "pub")]
    Publish(PublishStreamArgs),

    #[clap(alias = "sub")]
    Subscribe(SubscribeArgs),
}

#[cfg(feature = "pub_stream")]
#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct PublishStreamArgs {
    #[command(flatten)]
    pub publish_args: PublishArgs,

    #[arg(short, long, default_value_t=DEFAULT_RATE)]
    pub rate: f64,

    #[arg(long, default_value_t=DEFAULT_DURATION)]
    pub duration: usize,

    #[clap(allow_hyphen_values = true)]
    #[arg(long, default_value_t=DEFAULT_QUEUE)]
    pub queue: i64,

    #[arg(long, default_value_t=DEFAULT_NAGLE_OFF)]
    pub nagle_off: bool,
}
