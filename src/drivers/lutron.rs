use crate::{
    config::CasetaConfig,
    drivers::{Driver, DriverType},
    Command, Event, EventSource, EventType,
};
use async_channel::{Receiver, Sender};

pub struct CasetaDriver {
    cmd_rx: Receiver<Command>,
}

impl CasetaDriver {
    pub fn new(cmd_rx: Receiver<Command>, evt_tx: Sender<Event>, cfg: &CasetaConfig) -> Self {
        tokio::spawn(CasetaDriver::event_receiver_context(evt_tx));
        CasetaDriver { cmd_rx }
    }

    async fn handle_cmd(&self, _cmd: &Command) {
    }

    async fn event_receiver_context(tx: Sender<Event>) -> Result<(), ()> {
        loop {
            // read from the leap socket, look for events
            if let Err(err) = tx
                .send(Event {
                    src: EventSource::Driver(DriverType::Lutron),
                    id: EventType::ButtonPress(0),
                })
                .await
            {
                eprintln!("Unable to report event due to: {}", err);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

#[async_trait::async_trait]
impl Driver for CasetaDriver {
    async fn run(&self) -> Result<(), ()> {
        while let Ok(cmd) = self.cmd_rx.recv().await {
            self.handle_cmd(&cmd).await;
        }

        Ok(())
    }
}
