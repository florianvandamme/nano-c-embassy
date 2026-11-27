use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, signal::Signal};
use heapless::{String, Vec};

pub struct State {
    pub messages: Vec<String<64>, 5>,
}

pub static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State {
    messages: Vec::new(),
});
pub static DISPLAY_UPDATE: Signal<ThreadModeRawMutex, ()> = Signal::new();
