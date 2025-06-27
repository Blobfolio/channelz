/*!
# ChannelZ: Abacus

This module helps tidy up the file sizes being passed back and forth all over
the place.
*/

use crate::Flags;
use dactyl::{
	NiceU64,
	NicePercent,
};
use fyi_msg::{
	fyi_ansi::dim,
	AnsiColor,
	Msg,
};
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
	pub(super) fn summarize(self, kinds: Flags) {
		// Print the original raw total with commas in all the right places.
		let nice_raw = NiceU64::from(self.raw);
		let nice_len = nice_raw.len();
		Msg::new(("  Source", AnsiColor::LightMagenta), format!("{nice_raw} bytes"))
			.with_newline(true)
			.print();

		// Now do the same for each of the (enabled) encoded variants.
		let encoded: [(u64, &str, bool); 2] = [
			(self.br,  "  Brotli", kinds.contains(Flags::Brotli)),
			(self.gz,  "    Gzip", kinds.contains(Flags::Gzip)),
		];

		for (total, label, enabled) in encoded {
			if ! enabled || total == 0 { continue; }

			let nice = NiceU64::from(total);
			let mut msg = Msg::new((label, AnsiColor::LightMagenta), format!(
				"{:>nice_len$} bytes",
				nice.as_str(),
			))
				.with_newline(true);

			if total < self.raw {
				let nice_per = NicePercent::from((self.raw - total, self.raw));
				msg.set_suffix(format!(dim!(" (Saved {}.)"), nice_per));
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
