use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, signal::Signal};
use heapless::String;

pub struct State {
    pub message: String<64>,
}

pub static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State {
    message: String::new(),
});
pub static DISPLAY_UPDATE: Signal<ThreadModeRawMutex, ()> = Signal::new();
