use crate::Pot;


impl Pot<f32> for f32 {
    fn read(&self) -> f32 {
        *self
    }
}


pub struct TimedSawtoothPot {}
impl Default for TimedSawtoothPot { fn default() -> Self { Self {} } }

impl Pot<f32> for TimedSawtoothPot {
    fn read(&self) -> f32 {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("time moved backwards");
        ts.subsec_millis() as f32 / 1000.0
    }
}
