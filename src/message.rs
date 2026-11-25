use crate::state::{DISPLAY_UPDATE, STATE};
use core::fmt::Write;

#[embassy_executor::task]
pub async fn publish_message_task() {
    let mut i = 1;

    loop {
        {
            let mut state = STATE.lock().await;

            state.message.clear();
            write!(state.message, "Hello from executor with task number {i}").unwrap();
        }

        DISPLAY_UPDATE.signal(());
        i = i + 1;

        embassy_time::Timer::after_secs(10).await;
    }
}
