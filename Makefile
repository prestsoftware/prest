CORE = core/target/release/prest-core
DOCS = docs/build/html/index.html
GUI  = gui/.typecheck-ts

.PHONY: check test build all clean $(CORE) $(DOCS) $(GUI) version.txt

all: build

$(CORE): core/src/*.rs core/src/*/*.rs core/Cargo.toml
	(cd core; cargo build --release --bin prest-core)

$(DOCS):
	make -C docs build/html/index.html

$(GUI):
	make -C gui .typecheck-ts

# build everything there is to build or generate
build: $(GUI) version.txt preorders-7.bin $(CORE) $(DOCS)

preorders-7.bin:
	(cd core; cargo run --release --bin list-preorders)
	mv core/preorders-7.bin ./

version.txt:
	bash update-version.sh

clean:
	make -C docs clean
	-rm -f $(UIC_PY) $(GUI)
	(cd core; cargo clean --release)

run: build
	python gui/main.py

test: build
	pytest -v -m "not long" gui

bench: build
	pytest -v -m benchmark gui

longtest: fulltest

fulltest: check
	(cd core; cargo test --release)
	[ "$(TRAVIS_OS_NAME)" = windows ] || pytest -v -m "not benchmark" gui

