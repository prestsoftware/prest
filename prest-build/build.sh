#!/bin/bash

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

PNAME="prest-$(git describe)"
echo "Building ${PNAME}..."

die() {
	echo "$@"
	exit 1
}

remote() {
	echo "$1" | ssh win7 bash
}

# clean up possible old builds on win7
remote "rm -rf prest-build; mkdir prest-build"


##-- UI --##

# rebuild the docs
make -C docs clean html

# build all files
make

# run all tests
# - core tests
# - mypy typecheck
# - all GUI tests (including the long ones)
make fulltest \
	|| die "GUI test suite failed"

# send build files to win7
scp -r gui/* win7:prest-build/
scp -r version.txt docs/build/html preorders-7.bin win7:prest-build/


##-- CORE --##

# core was tested in the GUI fulltest above

# compile core
(cd core; ../build/build-core-win.sh) \
	|| die "could not compile core"

# copy core to win7
scp core/target/x86_64-pc-windows-gnu/release/prest-core.exe win7:prest-build

preorders_opt=""
if [ "$preorders" = 1 ]; then
	preorders_opt="--add-binary 'preorders-7.bin;.'"
fi

# build UI on win7
remote "
	cd prest-build;
	pyinstaller \
		--add-binary 'version.txt;.' \
		--add-binary 'prest-core.exe;.' \
		--add-binary 'images;images' \
		--add-binary 'html;html' \
		$preorders_opt \
		--onefile \
		-i images/prest.ico \
		main.py
"

#		--add-binary '../docs/build/qthelp/Prest.qhc;.'
#		--add-binary '../docs/build/qthelp/Prest.qch;.'

## -- Copy back -- ##

# copy files from win7
mkdir -p dist/
scp win7:"prest-build/dist/main.exe" "dist/$PNAME-unsigned.exe"
chmod 644 "dist/$PNAME-unsigned.exe"

if [ "$sign" = 1 ]; then
	build/sign.sh
fi
