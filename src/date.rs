#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
// fields are lexicographically ordered! magic
pub struct Date {
	pub year: u32,
	pub month: u32, // 1 = january
	pub day: u32
}
impl Date {
	pub fn from(yyyy_mm_dd: &str) -> Date {
		let bad_format = || -> ! {
			panic!("Date must be in YYYY-MM-DD format: \"{}\"", yyyy_mm_dd);
		};
		let bad_date = || -> ! {
			panic!("Date {} is invalid", yyyy_mm_dd);
		};
		if yyyy_mm_dd.len() != 10 {
			bad_format();
		}
		let year = yyyy_mm_dd[0..4].parse().unwrap_or_else(|_| bad_format());
		let month = yyyy_mm_dd[5..7].parse().unwrap_or_else(|_| bad_format());
		let day = yyyy_mm_dd[8..10].parse().unwrap_or_else(|_| bad_format());
		if !(1..=12).contains(&month) {
			bad_date();
		}
		let days_in_each_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
		if day < 1 || day > days_in_each_month[(month - 1) as usize] {
			bad_date();
		}
		let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
		if !is_leap && month == 2 && day == 29 {
			bad_date();
		}
		assert!(year > 1582, "Date {} possibly predates the Gregorian calendar; I'm not doing all that", yyyy_mm_dd);
		Date { year, month, day }
	}
	pub fn weekday_name(&self) -> &'static str {
		const LEADING_VALUES: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
		let year = if (self.month as i32) < 3 {
			self.year - 1
		} else {
			self.year
		};
		match (year + year / 4 - year / 100
			+ year / 400
			+ LEADING_VALUES[(self.month - 1) as usize]
			+ self.day)
			% 7
		{
			0 => "Sun",
			1 => "Mon",
			2 => "Tue",
			3 => "Wed",
			4 => "Thu",
			5 => "Fri",
			6 => "Sat",
			_ => unreachable!()
		}
	}
	pub fn to_rfc822(&self) -> String {
		format!(
			"{}, {:02} {} {} 17:00:00 GMT",
			self.weekday_name(),
			self.day,
			match self.month {
				1 => "Jan",
				2 => "Feb",
				3 => "Mar",
				4 => "Apr",
				5 => "May",
				6 => "Jun",
				7 => "Jul",
				8 => "Aug",
				9 => "Sep",
				10 => "Oct",
				11 => "Nov",
				12 => "Dec",
				_ => unreachable!()
			},
			self.year
		)
	}
	pub fn today() -> Date {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_else(|_| std::time::Duration::from_secs(0))
			.as_secs();
		let days = now / 86400;
		let z_value = days as i64 + 719_468;
		let era = (if z_value >= 0 {
			z_value
		} else {
			z_value - 146_096
		}) / 146_097;
		let doe = z_value - era * 146_097;
		let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
		let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
		let mp = (5 * doy + 2) / 153;
		let day = doy - (153 * mp + 2) / 5 + 1;
		let month = mp + if mp < 10 { 3 } else { -9 };
		let year = yoe + era * 400 + i64::from(month <= 2);
		Date {
			year: year.try_into().expect("Could not calculate today's date"),
			month: month.try_into().expect("Could not calculate today's date"),
			day: day.try_into().expect("Could not calculate today's date")
		}
	}

	pub fn now_rfc822() -> String {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_else(|_| std::time::Duration::from_secs(0))
			.as_secs();
		let secs = now % 60;
		let mins = (now / 60) % 60;
		let hours = (now / 3600) % 24;
		let days = now / 86400;
		let weekday = (4 + days % 7) % 7;
		let weekday_name = match weekday {
			0 => "Sun",
			1 => "Mon",
			2 => "Tue",
			3 => "Wed",
			4 => "Thu",
			5 => "Fri",
			6 => "Sat",
			_ => unreachable!()
		};
		let today = Date::today();
		let month_name = match today.month {
			1 => "Jan",
			2 => "Feb",
			3 => "Mar",
			4 => "Apr",
			5 => "May",
			6 => "Jun",
			7 => "Jul",
			8 => "Aug",
			9 => "Sep",
			10 => "Oct",
			11 => "Nov",
			12 => "Dec",
			_ => unreachable!()
		};
		format!(
			"{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
			weekday_name, today.day, month_name, today.year, hours, mins, secs
		)
	}
	pub fn to_iso8601(&self) -> String {
		format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
	}
	pub fn to_display(&self) -> String {
		format!(
			"{} {} {}",
			match self.month {
				1 => "Jan",
				2 => "Feb",
				3 => "Mar",
				4 => "Apr",
				5 => "May",
				6 => "Jun",
				7 => "Jul",
				8 => "Aug",
				9 => "Sep",
				10 => "Oct",
				11 => "Nov",
				12 => "Dec",
				_ => unreachable!()
			},
			self.day,
			self.year
		)
	}
	pub fn birthday(&self) -> bool {
		self.month == 12 && self.day == 5
	}
}
