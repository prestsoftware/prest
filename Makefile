CORE = core/target/release/prest-core
UIC_PY = $(patsubst %.ui,%.py,$(wildcard gui/uic/*.ui))

.PHONY: check test build all clean

all: build

# build everything there is to build/generate
build: version.txt $(CORE) $(UIC_PY) gui/.typecheck-ts
	make -C docs clean html

version.txt:
	bash update-version.sh

clean:
	make -C docs clean
	-rm -f $(CORE) $(UIC_PY) gui/.typecheck-ts
	(cd core; cargo clean --release)

$(CORE): core/src/*.rs core/src/*/*.rs core/Cargo.toml
	(cd core; cargo build --release --bin prest-core)

run: build
	python3 gui/main.py

test: build
	python3 -m pytest -v -m "not long"

bench: build
	python3 -m pytest -v -m benchmark

longtest: fulltest

fulltest: check
	(cd ../core; cargo test --release)
	#python3 -m pytest -v -m "not benchmark"
