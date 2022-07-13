.PHONY: deps build run check test clippy coverage benchmark flamegraph compare_benchmarks_deps compare_benchmarks docs clean compare_vm_output

TEST_DIR=cairo_programs
TEST_FILES:=$(wildcard $(TEST_DIR)/*.cairo)
COMPILED_TESTS:=$(patsubst $(TEST_DIR)/%.cairo, $(TEST_DIR)/%.json, $(TEST_FILES))
CAIRO_MEM:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.memory, $(COMPILED_TESTS))
CAIRO_TRACE:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.trace, $(COMPILED_TESTS))
CLEO_MEM:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.cleopatra.memory, $(COMPILED_TESTS))
CLEO_TRACE:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.cleopatra.trace, $(COMPILED_TESTS))

BENCH_DIR=cairo_programs/benchmarks
BENCH_FILES:=$(wildcard $(BENCH_DIR)/*.cairo)
COMPILED_BENCHES:=$(patsubst $(BENCH_DIR)/%.cairo, $(BENCH_DIR)/%.json, $(BENCH_FILES))

$(TEST_DIR)/%.json: $(TEST_DIR)/%.cairo
	cairo-compile $< --output $@

$(TEST_DIR)/%.cleopatra.memory: $(TEST_DIR)/%.json build
	./target/release/cleopatra-run $< --memory_file $@

$(TEST_DIR)/%.cleopatra.trace: $(TEST_DIR)/%.json build
	./target/release/cleopatra-run $< --trace_file $@

$(TEST_DIR)/%.memory: $(TEST_DIR)/%.json
	cairo-run --layout all --program $< --memory_file $@

$(TEST_DIR)/%.trace: $(TEST_DIR)/%.json
	cairo-run --layout all --program $< --trace_file $@

$(BENCH_DIR)/%.json: $(BENCH_DIR)/%.cairo
	cairo-compile $< --output $@
deps:
	cargo install --version 1.1.0 cargo-criterion
	cargo install --version 0.6.1 flamegraph
	pyenv install pypy3.7-7.3.9
	pyenv global pypy3.7-7.3.9
	pip install cairo_lang
	pyenv install 3.7.12
	pyenv global 3.7.12
	pip install cairo_lang

build:
	cargo build --release

run:
	cargo run

check:
	cargo check

test: $(COMPILED_TESTS) $(CAIRO_TRACE) $(CAIRO_MEM)
	cargo test

clippy:
	cargo clippy  -- -D warnings

coverage:
	docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin

benchmark: $(COMPILED_BENCHES)
	cargo criterion --bench criterion_benchmark
	@echo 'Report: target/criterion/reports/index.html'

flamegraph:
	cargo flamegraph --root --bench criterion_benchmark -- --bench

compare_benchmarks: $(COMPILED_BENCHES)
	cd bench && ./run_benchmarks.sh
 
compare_trace_memory: $(CLEO_TRACE) $(CAIRO_TRACE) $(CLEO_MEM) $(CAIRO_MEM)
	cd tests; ./compare_vm_state.sh trace memory

compare_trace: $(CLEO_TRACE) $(CAIRO_TRACE)
	cd tests; ./compare_vm_state.sh trace

compare_memory: $(CLEO_MEM) $(CAIRO_MEM)
	cd tests; ./compare_vm_state.sh memory

docs:
	cargo doc --verbose --release --locked --no-deps

clean:
	rm -f $(TEST_DIR)/*.json
	rm -f $(TEST_DIR)/*.memory
	rm -f $(TEST_DIR)/*.trace
	rm -f $(BENCH_DIR)/*.json
