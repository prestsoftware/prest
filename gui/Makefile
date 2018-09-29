CORE = ../core/target/release/prest-core
UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))
QHC = ../docs/build/qthelp/Prest.qhc

.PHONY: check test build all clean check-version

all: build

# build everything there is to build/generate
build: check-version $(UIC_PY) $(CORE) $(QHC)

$(QHC)p: $(shell find ../docs/src -name \*.py) version.txt
	make -C ../docs qthelp

$(QHC): $(QHC)p
	qcollectiongenerator $<

viewhelp: $(QHC)
	assistant -collectionFile $(QHC)

check-version:
	bash update-version.sh

clean:
	-rm -rf ../docs/build
	-rm -f $(CORE) $(UIC_PY)
	(cd ../core; cargo clean --release)

$(CORE): ../core/src/*.rs ../core/src/*/*.rs ../core/Cargo.toml
	(cd ../core; cargo build --release --bin prest-core)

uic/%.py: uic/%.ui
	pyuic5 $< -o $@

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy --platform win32 --ignore-missing-imports --check-untyped-defs --strict-optional main.py
	touch .typecheck-ts

# override the timestamp and recheck anyway
check: build
	mypy --platform win32 --ignore-missing-imports --check-untyped-defs --strict-optional main.py
	touch .typecheck-ts

run: .typecheck-ts build
	python3 main.py

test: build
	python3 -m pytest -v -m "not long"

bench: build
	python3 -m pytest -v -m benchmark

longtest: fulltest

fulltest: check
	(cd ../core; cargo test --release)
	python3 -m pytest -v -m "not benchmark"
