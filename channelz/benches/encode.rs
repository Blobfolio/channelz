/*!
# Benchmark: `channelz::EncodeFile`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use channelz::EncodeFile;
use std::path::PathBuf;



fn encode_all(c: &mut Criterion) {
	let mut group = c.benchmark_group("channelz::EncodeFile");

	for path in [
		PathBuf::from("../test/assets/favicon.svg"),
	].iter() {
		assert!(path.is_file(), "Invalid file: {:?}", path);
		group.bench_function(format!("{:?}.encode_all()", path), move |b| {
			b.iter(|| path.encode_all())
		});
	}

	group.finish();
}



criterion_group!(
	benches,
	encode_all,
);
criterion_main!(benches);
