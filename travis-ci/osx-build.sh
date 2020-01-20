#!/bin/bash

set -euo pipefail

PATH="$HOME/.cargo/bin:$PATH"

if [ "$TRAVIS_OS_NAME" = "osx" ]; then
	PATH="$HOME/Library/Python/3.7/bin:$PATH"
fi

cd "$(dirname $0)/.." # work from the root directory of Prest

PNAME="prest-${TRAVIS_OS_NAME}-$(git describe --always)"

echo "Building ${PNAME} for ${TRAVIS_OS_NAME}..."
make

pyinstaller \
    --add-data 'version.txt:.' \
    --add-binary 'core/target/release/prest-core:.' \
    --add-data 'gui/images:images' \
    --add-data 'docs/build/html:html' \
    --add-data 'preorders-7.bin:.' \
    --osx-bundle-identifier com.prestsoftware.prest \
    --onefile \
	--name "$PNAME" \
    -i gui/images/prest.ico \
    gui/main.py
