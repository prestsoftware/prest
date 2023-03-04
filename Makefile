CORE = prest-core/target/release/prest-core
DOCS = docs/build/html/index.html
GUI  = gui/.typecheck-ts

.PHONY: check test build all clean $(CORE) $(DOCS) $(GUI) version.txt

all: build

$(CORE): prest-core/src/*.rs prest-core/src/*/*.rs prest-core/Cargo.toml
	(cd prest-core; cargo build --release --bin prest-core)
	(cd prest-core; cargo build --release --bin plot-model)
	(cd prest-core; cargo build --release --bin list-preorders)

$(DOCS):
	poetry -C gui run -- make -C docs build/html/index.html

$(GUI):
	poetry -C gui run -- make -C gui .typecheck-ts

# build everything there is to build or generate
build: $(GUI) version.txt preorders-7.bin $(CORE) $(DOCS)
	poetry -C gui install

preorders-7.bin:
	(cd prest-core; cargo run --release --bin list-preorders)
	mv prest-core/preorders-7.bin ./

version.txt:
	bash update-version.sh

clean:
	poetry -C gui run -- make -C docs clean
	-rm -f $(UIC_PY) $(GUI)
	(cd prest-core; cargo clean --release)

run: build
	poetry -C gui run -- python gui/main.py

test: build
	(cd prest-lib; cargo test --release)
	(cd prest-core; cargo test --release)
	poetry -C gui run -- pytest -v -m "not long" gui

bench: build
	poetry -C gui run -- pytest -v -m benchmark gui

longtest: fulltest

fulltest: check
	[ "$(TRAVIS_OS_NAME)" = windows ] || poetry -C gui run -- pytest -v -m "not benchmark" gui

