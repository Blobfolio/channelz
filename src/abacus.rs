/*!
# ChannelZ: Abacus

This module helps tidy up the file sizes being passed back and forth all over
the place.
*/

use crate::{
	FLAG_BR,
	FLAG_GZ,
};
use dactyl::{
	NiceU64,
	NicePercent,
};
use fyi_msg::Msg;
use std::{
	num::NonZeroU64,
	ops::{
		Add,
		AddAssign,
	},
};



#[derive(Debug, Clone, Copy)]
/// # Encoder Totals.
///
/// This struct is used to represent the various file sizes after a single
/// encoding pass.
///
/// The brotli and gzip totals are only set if smaller than the original; if
/// `None`, they're treated as equivalent (zero savings).
pub(super) struct EncoderTotals {
	/// # Raw Size.
	raw: NonZeroU64,

	/// # Brotli Size.
	br: Option<NonZeroU64>,

	/// # Gzip Size.
	gz: Option<NonZeroU64>,
}

impl EncoderTotals {
	/// # New.
	///
	/// Return a new instance with the raw size thusly set.
	pub(super) const fn new(raw: NonZeroU64) -> Self {
		Self { raw, br: None, gz: None }
	}

	/// # Set Brotli.
	///
	/// Set the brotli size if smaller than the original.
	pub(super) fn set_br(&mut self, br: NonZeroU64) {
		if br < self.raw { self.br.replace(br); }
	}

	/// # Set Gzip.
	///
	/// Set the gzip size if smaller than the original.
	pub(super) fn set_gz(&mut self, gz: NonZeroU64) {
		if gz < self.raw { self.gz.replace(gz); }
	}
}



#[derive(Debug, Clone, Copy)]
/// # Thread Totals.
///
/// This struct is used to hold the cumulative file size totals for each worker
/// thread, and eventually the sum of those sums.
pub(super) struct ThreadTotals {
	/// # Raw Size.
	raw: u64,

	/// # Brotli Size.
	br: u64,

	/// # Gzip Size.
	gz: u64,
}

impl ThreadTotals {
	/// # New.
	///
	/// Return a default instance with all totals set to zero.
	pub(super) const fn new() -> Self {
		Self {
			raw: 0,
			br: 0,
			gz: 0,
		}
	}

	/// # Summarize.
	///
	/// Print a nice summary of the work done.
	pub(super) fn summarize(mut self, kinds: u8) {
		// What formats were we doing?
		let has_br = FLAG_BR == kinds & FLAG_BR;
		let has_gz = FLAG_GZ == kinds & FLAG_GZ;

		// Grab the totals.
		if ! has_br { self.br = 0 };
		if ! has_gz { self.gz = 0 };

		// Add commas to the numbers.
		let nice_raw = NiceU64::from(self.raw);
		let nice_len = nice_raw.len();

		// Print the raw total!
		Msg::custom("  Source", 13, &format!("{nice_raw} bytes"))
			.with_newline(true)
			.print();

		// Print the brotli total if enabled.
		if has_br {
			let nice_br = NiceU64::from(self.br);
			let mut msg = Msg::custom("  Brotli", 13, &format!(
				"{:>nice_len$} bytes",
				nice_br.as_str(),
			))
				.with_newline(true);

			if let Ok(nice_per) = NicePercent::try_from((self.raw - self.br, self.raw)) {
				msg.set_suffix(format!(" \x1b[2m(Saved {nice_per}.)\x1b[0m"));
			}

			msg.print();
		}

		// And lastly print the gzip total if enabled.
		if has_gz {
			let nice_gz = NiceU64::from(self.gz);
			let mut msg = Msg::custom("    Gzip", 13, &format!(
				"{:>nice_len$} bytes",
				nice_gz.as_str(),
			))
				.with_newline(true);

			if let Ok(nice_per) = NicePercent::try_from((self.raw - self.gz, self.raw)) {
				msg.set_suffix(format!(" \x1b[2m(Saved {nice_per}.)\x1b[0m"));
			}

			msg.print();
		}
	}
}

impl Add for ThreadTotals {
	type Output = Self;

	#[inline]
	fn add(self, other: Self) -> Self {
		Self {
			raw: self.raw + other.raw,
			br: self.br + other.br,
			gz: self.gz + other.gz,
		}
	}
}

impl AddAssign<EncoderTotals> for ThreadTotals {
	#[inline]
	fn add_assign(&mut self, len2: EncoderTotals) {
		let raw2 = len2.raw.get();
		self.raw += raw2;
		self.br += len2.br.map_or(raw2, NonZeroU64::get);
		self.gz += len2.gz.map_or(raw2, NonZeroU64::get);
	}
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_totals() {
		// Should be zero to start.
		let mut totals = ThreadTotals::new();
		assert_eq!(totals.raw, 0);
		assert_eq!(totals.br, 0);
		assert_eq!(totals.gz, 0);

		// Populate encoder totals, then add those to our sum.
		let mut enc = EncoderTotals::new(NonZeroU64::new(100).unwrap());
		enc.set_br(NonZeroU64::new(80).unwrap());
		enc.set_gz(NonZeroU64::new(90).unwrap());
		totals += enc;
		assert_eq!(totals.raw, 100);
		assert_eq!(totals.br, 80);
		assert_eq!(totals.gz, 90);

		// Do it again, but this time leave br/gz set to none.
		totals += EncoderTotals::new(NonZeroU64::new(100).unwrap());
		assert_eq!(totals.raw, 200);
		assert_eq!(totals.br, 180);
		assert_eq!(totals.gz, 190);

		// And verify that ThreadTotal can be added to itself.
		let totals = totals + totals;
		assert_eq!(totals.raw, 400);
		assert_eq!(totals.br, 360);
		assert_eq!(totals.gz, 380);
	}
}
