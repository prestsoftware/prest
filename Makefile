CORE = core/target/release/prest-core
DOCS = docs/build/html/index.html
GUI  = gui/.typecheck-ts

.PHONY: check test build all clean $(CORE) $(DOCS) $(GUI)

all: build

$(CORE): core/src/*.rs core/src/*/*.rs core/Cargo.toml
	(cd core; cargo build --release --bin prest-core)

$(DOCS):
	make -C docs build/html/index.html

$(GUI):
	make -C gui .typecheck-ts

# build everything there is to build/generate
build: version.txt $(CORE) $(DOCS) $(GUI)

version.txt:
	bash update-version.sh

clean:
	make -C docs clean
	-rm -f $(UIC_PY) $(GUI)
	(cd core; cargo clean --release)

run: build
	python3 gui/main.py

test: build
	python3 -m pytest -v -m "not long" gui

bench: build
	python3 -m pytest -v -m benchmark gui

longtest: fulltest

fulltest: check
	(cd core; cargo test --release)
	python3 -m pytest -v -m "not benchmark" gui
