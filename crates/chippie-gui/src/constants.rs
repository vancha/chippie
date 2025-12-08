use iced::time::Duration;

pub const APP_NAME: &str = "Chippie";
// 60 times a second (kind of, it should have been 16.667 )
pub const TICK_INTERVAL: Duration = Duration::from_millis(17);
/// How many cycles the cpu advances for every frame. This decides how fast the cpu will run
pub const CYCLES_PER_FRAME: usize = 5;
