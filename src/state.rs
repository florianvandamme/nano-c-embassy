use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, signal::Signal};

pub struct State<'a> {
    pub message: &'a str,
}

pub static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State { message: "" });
pub static DISPLAY_UPDATE: Signal<ThreadModeRawMutex, ()> = Signal::new();
