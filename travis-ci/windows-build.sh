#!/bin/bash

set -xeuo pipefail

cd "$(dirname $0)/.." # work from the root directory of Prest

PNAME="prest-${TRAVIS_OS_NAME}-$(git describe --always)"

export PATH="/c/python39:/c/python39/Scripts:/c/Users/travis/AppData/Roaming/Python/Python39/Scripts:$PATH"

# try this
PIP=/c/python39/Scripts/pip
$PIP install --user --upgrade pip
$PIP install --user pyinstaller
$PIP install --user -r gui/requirements.txt

echo "Building ${PNAME} for ${TRAVIS_OS_NAME}..."
make

pyinstaller \
    --add-data 'version.txt;.' \
    --add-binary 'core/target/release/prest-core.exe;.' \
    --add-data 'gui/images;images' \
    --add-data 'docs/build/html;html' \
    --add-data 'preorders-7.bin;.' \
    --onefile \
	--name "${PNAME}.exe" \
    -i gui/images/prest.ico \
    gui/main.py
