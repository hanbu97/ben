use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "Ben", about = "Currently only support on Linux")]
pub struct CmdParams {
    pub command: String,
    /// Set monitor interval
    #[structopt(short = "i", long = "interval", default_value = "1.0")]
    pub interval: f32,
    /// Use exactly interval, if true will ignore time spent during moniting
    #[structopt(short = "e", long)]
    pub exact: bool,
    /// Monitor mode: gpu, mem
    #[structopt(short = "m", long)]
    pub mode: Vec<String>,
    /// Save
    #[structopt(
        short = "o",
        long = "output",
        default_value = "./ben.log",
        parse(from_os_str)
    )]
    pub output: PathBuf,
}
