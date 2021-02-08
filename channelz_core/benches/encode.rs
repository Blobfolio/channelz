/*!
# Benchmark: `channelz`
*/

use channelz_core::encode_path;
use fyi_bench::{
	Bench,
	benches,
};
use std::{
	path::PathBuf,
	time::Duration,
};

benches!(
	Bench::new("channelz", "encode_path(css)")
		.timed(Duration::from_secs(2))
		.with_setup_ref(PathBuf::from("../test/assets/core.css"), |p| encode_path(p)),

	Bench::new("channelz", "encode_path(html)")
		.timed(Duration::from_secs(2))
		.with_setup_ref(PathBuf::from("../test/assets/index.html"), |p| encode_path(p)),

	Bench::new("channelz", "encode_path(min.js)")
		.timed(Duration::from_secs(10))
		.with_setup_ref(PathBuf::from("../test/assets/core.min.js"), |p| encode_path(p)),

	Bench::new("channelz", "encode_path(svg)")
		.timed(Duration::from_secs(2))
		.with_setup_ref(PathBuf::from("../test/assets/favicon.svg"), |p| encode_path(p))
);
