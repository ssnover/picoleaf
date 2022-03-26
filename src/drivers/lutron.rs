use crate::{
    config::CasetaConfig,
    drivers::{Driver, DriverType},
    Command, DeviceId, Event, EventSource, EventType,
};
use async_channel::{Receiver, Sender};

pub struct CasetaDriver {
    cmd_rx: Receiver<Command>,
}

impl CasetaDriver {
    pub async fn new(
        cmd_rx: Receiver<Command>,
        evt_tx: Sender<Event>,
        cfg: &CasetaConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let certs = casita::Certs::new(
            cfg.ca_cert_path.clone().into(),
            cfg.cert_path.clone().into(),
            cfg.key_path.clone().into(),
        )?;
        let mut client = casita::Client::new(certs, cfg.address.clone()).await;
        client.connect().await?;
        let msg_rx = client
            .subscribe()
            .expect("Successfully connected, but could not subscribe to Caseta client");
        tokio::spawn(CasetaDriver::event_receiver_context(msg_rx, evt_tx));
        Ok(CasetaDriver { cmd_rx })
    }

    async fn handle_cmd(&self, _cmd: &Command) {}

    async fn event_receiver_context(
        msg_rx: Receiver<casita::client::Message>,
        tx: Sender<Event>,
    ) -> Result<(), ()> {
        loop {
            if let Ok(msg) = msg_rx.recv().await {
                match msg {
                    casita::client::Message::Raw(json_value) => {
                        log::debug!("Got raw leap message: {}", json_value);
                    }
                    casita::client::Message::Decoded(msg) => {
                        log::info!("Got decoded leap message: {:?}", msg);
                    }
                }
            }
            // read from the leap socket, look for events
            if let Err(err) = tx
                .send(Event {
                    src: EventSource::Driver(DriverType::Lutron),
                    id: EventType::ButtonPress(DeviceId(0)),
                })
                .await
            {
                eprintln!("Unable to report event due to: {}", err);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(())
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
