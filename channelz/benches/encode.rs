/*!
# Benchmark: `channelz::EncodeFile`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use std::path::PathBuf;



fn encode_path(c: &mut Criterion) {
	let mut group = c.benchmark_group("channelz::encode_path");

	for path in [
		PathBuf::from("../test/assets/favicon.svg"),
	].iter() {
		assert!(path.is_file(), "Invalid file: {:?}", path);
		group.bench_function(format!("{:?}", path), move |b| {
			b.iter(|| channelz::encode_path(path))
		});
	}

	group.finish();
}



criterion_group!(
	benches,
	encode_path,
);
criterion_main!(benches);
