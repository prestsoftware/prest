#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.."

rclone -v sync prest-db:PrestSoftware/docs/src/ docs/src/
rm docs/src/_static/images/PrestLogoCrop.pdf
rm docs/src/_static/images/prest-screen.png
