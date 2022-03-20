/// Design: Drivers should have a handle which allows sending abstract commands and a handle from which the caller can receive event notices.
///
/// These two handles need to be separate in order to allow different owners to call them.
///
pub mod lutron;

use crate::{config::Config, Command, Event};
use async_channel::{Receiver, Sender};
use lutron::CasetaDriver;

#[async_trait::async_trait]
pub trait Driver {
    async fn run(&self) -> Result<(), ()>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum DriverType {
    Lutron = 1,
    Nanoleaf = 2,
    IpcSocket = 3,
}

pub fn start_driver(driver: DriverType, cfg: &Config) -> (Sender<Command>, Receiver<Event>) {
    // create channels, tokio spawn
    let (cmd_tx, cmd_rx) = async_channel::unbounded::<Command>();
    let (evt_tx, evt_rx) = async_channel::unbounded::<Event>();

    let driver: Box<dyn Driver + Send + Sync> = match driver {
        DriverType::Lutron => Box::new(CasetaDriver::new(cmd_rx, evt_tx, &cfg.caseta)),
        _ => {
            unimplemented!()
        }
    };

    tokio::spawn(async move { driver.run().await });

    (cmd_tx, evt_rx)
}
