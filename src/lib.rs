pub mod config;
pub mod drivers;

pub struct DeviceId(u32);

pub enum EventSource {
    Driver(drivers::DriverType),
}

pub enum EventType {
    ButtonPress(DeviceId),
    Disconnected(drivers::DriverType),
}

pub struct Event {
    pub src: EventSource,
    pub id: EventType,
}

pub enum Command {
    Connect,
    Disconnect,
    Reconnect,
    Enable(DeviceId),
    Disable(DeviceId),
    Display(DeviceId, String),
}
