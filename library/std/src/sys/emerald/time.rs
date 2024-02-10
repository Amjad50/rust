use crate::time::Duration;
use emerald_std::clock::ClockType;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant(Duration);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime(Duration);

pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(0));

// its a bit confusing, but `SystemTime` refers to the time since boot,
// and `RealTime` refers to the time since the unix epoch
impl Instant {
    pub fn now() -> Instant {
        let time = unsafe {
            emerald_std::clock::get_time(ClockType::SystemTime).expect("Failed to get time")
        };
        Instant(Duration::new(time.seconds, time.nanoseconds as u32))
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.0.checked_sub(other.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_sub(*other)?))
    }
}

impl SystemTime {
    pub fn now() -> SystemTime {
        let time = unsafe {
            emerald_std::clock::get_time(ClockType::RealTime).expect("Failed to get time")
        };
        SystemTime(Duration::new(time.seconds, time.nanoseconds as u32))
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.checked_sub(other.0).ok_or_else(|| other.0 - self.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_sub(*other)?))
    }
}
