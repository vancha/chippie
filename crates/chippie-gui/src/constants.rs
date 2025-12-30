use iced::time::Duration;

pub const APP_NAME: &str = "Chippie";
// 60 times a second (kind of, it should have been 16.667 )
pub const TICK_INTERVAL: Duration = Duration::from_millis(17);
