.PHONY: fuzz* fmt

fmt:
	cargo fmt && (cd fuzz/ && cargo fmt) && (cd benchmarks && cargo fmt)

fuzz_all: fuzz_ranking fuzz_unranking fuzz_bigint-previous-next fuzz_bigint-all
	:

fuzz_ranking:
	cargo fuzz run ranking --sanitizer=none -- -max_total_time=20 -max_len=9
fuzz_unranking:
	cargo fuzz run unranking --sanitizer=none -- -max_total_time=20 -max_len=128
fuzz_bigint-previous-next:
	cargo fuzz run bigint-previous-next --sanitizer=none -- -max_total_time=20 -max_len=600
fuzz_bigint-all:
	cargo fuzz run bigint-all --sanitizer=none -- -max_total_time=20 -max_len=600

