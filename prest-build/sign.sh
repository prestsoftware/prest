#!/bin/bash

cd "$(dirname $0)/.."

version=$(cat version.txt)

signcode \
	-spc keys/pem/cert.spc \
	-v keys/pem/key.pvk \
	-a sha1 \
	-$ individual \
	-n "Prest $(cat version.txt)" \
	-i "https://prestsoftware.com" \
	-t "http://timestamp.verisign.com/scripts/timstamp.dll" \
	-tr 10 \
	dist/prest-${version}-unsigned.exe

mv dist/prest-${version}-unsigned.exe     dist/prest-${version}.exe
mv dist/prest-${version}-unsigned.exe.bak dist/prest-${version}-unsigned.exe
