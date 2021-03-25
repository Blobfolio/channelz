/*!
# Benchmark: `channelz`
*/

use channelz_core::encode_path;
use brunch::{
	Bench,
	benches,
};
use std::time::Duration;

benches!(
	Bench::new("channelz", "encode_path(css)")
		.timed(Duration::from_secs(2))
		.with(|| encode_path("../test/assets/core.css")),

	Bench::new("channelz", "encode_path(html)")
		.timed(Duration::from_secs(2))
		.with(|| encode_path("../test/assets/index.html")),

	Bench::new("channelz", "encode_path(svg)")
		.timed(Duration::from_secs(2))
		.with(|| encode_path("../test/assets/favicon.svg"))
);
