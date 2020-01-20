#!/bin/bash

set -euo pipefail

PIP="python3 -m pip"

$PIP install --upgrade pip
$PIP install --user pyinstaller
$PIP install --user -r gui/requirements.txt
