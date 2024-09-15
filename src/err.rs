/*!
# ChannelZ: Errors
*/

use argyle::ArgyleError;
use std::fmt;



#[expect(clippy::missing_docs_in_private_items, reason = "Self-explanatory.")]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Errors.
///
/// This is the binary's obligatory custom error type.
pub(super) enum ChannelZError {
	Argue(ArgyleError),
	Killed,
	NoEncoders,
	NoFiles,
}

impl std::error::Error for ChannelZError {}

impl fmt::Display for ChannelZError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<ArgyleError> for ChannelZError {
	#[inline]
	fn from(src: ArgyleError) -> Self { Self::Argue(src) }
}

impl ChannelZError {
	/// # As String Slice.
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Argue(e) => e.as_str(),
			Self::Killed => "The process was aborted early.",
			Self::NoEncoders => "At least one encoder needs to be enabled.",
			Self::NoFiles => "No encodeable files were found.",
		}
	}
}
