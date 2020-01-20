#!/bin/bash

set -xeuo pipefail

choco install python
choco install make
choco install pip || true

# python installation messes up paths so we can't use pip until build.sh
