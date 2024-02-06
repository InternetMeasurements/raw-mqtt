use clap::Parser;

const DEFAULT_HOST : &str = "127.0.0.1";
const DEFAULT_PORT : u16 = 1883;
const DEFAULT_TRANSPORT : &str = "tcp";
const DEFAULT_KEEP_ALIVE : u64 = 5;
const DEFAULT_QOS : u8 = 1;
const DEFAULT_PROTO_VERSION: &str = "3.1.1";
const DEFAULT_SERVER_NAME: &str = "localhost";
const DEFAULT_INSECURE: bool = false;
const DEFAULT_DEBUG: bool = false;
const DEFAULT_RATE: f64 = 0.0;
const DEFAULT_DURATION: usize = 10;

#[derive(Debug, Clone)]
pub enum Request {
    Publish,
    Subscribe
}

#[derive(Parser)]
#[command(name = "mqtt-client")]
#[command(bin_name = "mqtt-client")]
pub enum MqttCli {
    #[clap(alias = "pub")]
    Publish(PublishArgs),

    #[clap(alias = "sub")]
    Subscribe(SubscribeArgs),
}

#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {

    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,

    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    #[arg(long, default_value = DEFAULT_TRANSPORT)]
    pub transport: String,

    #[arg(short, long)]
    pub topic: String,

    #[arg(short, long, default_value_t = DEFAULT_QOS)]
    pub qos: u8,

    #[arg(short, long, default_value_t = DEFAULT_KEEP_ALIVE)]
    pub keep_alive: u64,

    #[arg(short, long, default_value_t = DEFAULT_INSECURE)]
    pub insecure: bool,

    #[arg(long, default_value = DEFAULT_PROTO_VERSION)]
    pub proto_version: String,

    #[arg(long, default_value = DEFAULT_SERVER_NAME)]
    pub server_name: String,

    #[arg(short, long, default_value_t = DEFAULT_DEBUG)]
    pub debug: bool
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
#[clap(group(clap::ArgGroup::new("payload")
    .required(true)
    .args(&["size", "message"]),
))]
pub struct PublishArgs{

    #[command(flatten)]
    pub args: Args,

    #[arg(short, long, group = "payload")]
    pub size: Option<usize>,

    #[arg(long, group = "payload")]
    pub message: Option<String>

}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct SubscribeArgs{
    #[command(flatten)]
    pub args: Args
}

#[cfg(feature = "pub_stream")]
#[derive(Parser)]
#[command(name = "mqtt-client")]
#[command(bin_name = "mqtt-client")]
pub enum MqttStreamCli{
    #[clap(alias = "pub")]
    Publish(PublishStreamArgs),

    #[clap(alias = "sub")]
    Subscribe(SubscribeArgs),

}

#[cfg(feature = "pub_stream")]
#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct PublishStreamArgs{

    #[command(flatten)]
    pub args: PublishArgs,

    #[arg(short, long, default_value_t=DEFAULT_RATE)]
    pub rate: f64,

    #[arg(long, default_value_t=DEFAULT_DURATION)]
    pub duration: usize
}