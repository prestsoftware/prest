#!/bin/bash

set -euo pipefail

PATH="$HOME/.cargo/bin:$HOME/Library/Python/3.7/bin:$PATH"
cd "$(dirname $0)/.." # work from the root directory of Prest

PNAME="prest-$(git describe --always)"

echo "Building ${PNAME} for OSX..."
make

pyinstaller \
    --add-data 'version.txt:.' \
    --add-binary 'core/target/release/prest-core:.' \
    --add-data 'gui/images:images' \
    --add-data 'docs/build/html:html' \
    --add-data 'preorders-7.bin:.' \
    --osx-bundle-identifier com.prestsoftware.prest \
    --onefile \
    -i gui/images/prest.ico \
    gui/main.py
