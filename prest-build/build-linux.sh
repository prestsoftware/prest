#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.." # work from the root directory of Prest

(cd gui;
pyinstaller \
	--add-binary '../version.txt:.' \
	--add-binary '../core/target/release/prest-core:.' \
	--add-binary 'images:images' \
	--add-binary '../docs/build/html:html' \
	--onefile \
	-n prest-$(cat ../version.txt) \
	-i 'images/prest.ico' \
	--distpath '../dist/' \
	main.py
)
