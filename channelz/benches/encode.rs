/*!
# Benchmark: `fyi_msg::Msg`
*/

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use channelz::encode;
use std::path::PathBuf;



fn encode_br(c: &mut Criterion) {
	let mut group = c.benchmark_group("channelz::encode");

	for path in [
		// PathBuf::from("../test/assets/core.css"),
		PathBuf::from("../test/assets/favicon.svg"),
	].iter() {
		// The file should exist.
		assert!(path.is_file());

		let stub = path.to_str().expect("It's fine.");
		let data = std::fs::read(&path).expect("It's fine.");

		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"encode_br({:?})",
				path
			)),
			&(stub, data),
			|b, (stub, data)| {
				b.iter(||
					encode::encode_br(stub, data)
				);
			}
		);
	}
}

fn encode_gz(c: &mut Criterion) {
	let mut group = c.benchmark_group("channelz::encode");

	for path in [
		PathBuf::from("../test/assets/core.css"),
		PathBuf::from("../test/assets/favicon.svg"),
	].iter() {
		// The file should exist.
		assert!(path.is_file());

		let stub = path.to_str().expect("It's fine.");
		let data = std::fs::read(&path).expect("It's fine.");

		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"encode_gz({:?})",
				path
			)),
			&(stub, data),
			|b, (stub, data)| {
				b.iter(||
					encode::encode_gz(stub, data)
				);
			}
		);
	}
}

criterion_group!(
	benches,
	encode_br,
	encode_gz,
);
criterion_main!(benches);
