#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.."

rclone -v sync docs/src/ prest-db:PrestSoftware/docs/src/
