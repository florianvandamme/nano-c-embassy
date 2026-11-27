use heapless::String;

use crate::state::{DISPLAY_UPDATE, STATE};

pub async fn publish_debug_message(message: &str) {
    {
        let mut state = STATE.lock().await;

        if state.messages.is_full() {
            state.messages.remove(0);
        }

        let mut line = String::<64>::new();

        if line.push_str(message).is_ok() {
            let _ = state.messages.push(line);
        }

        DISPLAY_UPDATE.signal(());
    }
}
