#!/bin/bash

PATH="$HOME/.cargo/bin:$HOME/Library/Python/3.7/bin:$PATH"

cd "$(dirname $0)/.." # work from the root directory of Prest

sign=0
preorders=0
while [ -n "$1" ]; do
    case "$1" in
        --sign) sign=1; shift;;
        --preorders) preorders=1; shift;;
        *) echo "unrecognised option: $1"; exit 1;;
    esac
done

# $1 might be undefined in the parser above
set -euo pipefail

PNAME="prest-$(git describe --always)"
echo "Building ${PNAME} for mac..."

make

preorders_opt=""
if [ "preorders" = 1 ]; then
    preorders_opt="--addbinary 'preorders-7.bin;.'"
fi

pyinstaller \
    --add-data 'version.txt:.' \
    --add-binary 'core/target/release/prest-core:.' \
    --add-data 'gui/images:images' \
    --add-data 'docs/build/html:html' \
    $preorders_opt \
    --onefile \
    -i gui/images/prest.ico \
    gui/main.py
