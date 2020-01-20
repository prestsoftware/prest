#!/bin/bash

# install rust
rust_version="1.36.0-x86_64-apple-darwin"

curl -sSfL "https://static.rust-lang.org/rustup.sh" > rustup-init.sh
sh rustup-init.sh -y --default-toolchain "${rust_version}"

# install python3
python_version="3.7.4-macosx10.9"

curl -sSfL "https://www.python.or/ftp/python/3.7.4/python-${python_version}.pkg" > python3.pkg
installer -package python3.pkg -target /

pip3 install --user --no-use-pep517 pyinstaller
pip3 install --user \
    sphinx \
    sphinx-rtd-theme \
    sphinxcontrib-bibtex \
    pyqt5 \
    pytest \
    hypothesis \
    numpy \
    openpyxl
