pub mod config;
pub mod drivers;

pub struct DeviceId(u32);

pub enum EventSource {
    Driver(drivers::DriverType),
}

pub enum EventType {
    ButtonPress(u32),
}

pub struct Event {
    pub src: EventSource,
    pub id: EventType,
}

pub enum Command {
    Connect,
    Disconnect,
    Reconnect,
    Enable(u32),
    Disable(u32),
    Display(u32, String),
}
