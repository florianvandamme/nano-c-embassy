use crate::state::{DISPLAY_UPDATE, STATE};

#[embassy_executor::task]
pub async fn publish_message_task() {
    loop {
        {
            let mut state = STATE.lock().await;
            state.message = "Hello from publish task";
        }

        DISPLAY_UPDATE.signal(());
        embassy_time::Timer::after_secs(10).await;
    }
}
