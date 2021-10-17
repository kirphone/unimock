pub mod logger;
pub mod database;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ports")]
pub struct Ports{

    /// Web page port
    #[structopt(long)]
    port: u32
}

impl Ports{
    pub fn get_port(&self) -> u32{
        self.port
    }
}