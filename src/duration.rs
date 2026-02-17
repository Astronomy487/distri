#[derive(Clone, Copy)]
pub struct Duration {
	milliseconds: u32
}

impl std::ops::Add for Duration {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Self {
			milliseconds: self.milliseconds + other.milliseconds
		}
	}
}

impl std::fmt::Display for Duration {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let total_seconds = self.milliseconds / 1000;
		let minutes = (total_seconds / 60) % 60;
		let hours = (total_seconds / 3600) % 60;
		let seconds = total_seconds % 60;
		if hours > 0 {
			write!(fmt, "{}h{:02}m{:02}s", hours, minutes, seconds)
		} else {
			write!(fmt, "{}m{:02}s", minutes, seconds)
		}
	}
}

impl Duration {
	pub fn from_milliseconds(milliseconds: u32) -> Duration {
		Self {
			milliseconds
		}
	}
}