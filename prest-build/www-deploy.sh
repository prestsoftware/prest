#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.." # work from the root directory of Prest

make -C docs clean html

rsync -uav docs/build/html/ prestsoftware.com:public_html/
