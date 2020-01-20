#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.."

licenses="$(cd core; cargo license-hound 2)"

if echo "$licenses" | grep null; then
	echo '!!! SOME LICENSE IS NOT COMPATIBLE !!!' >&2
	exit 1
fi

echo "$licenses" \
	| jq -r '
		.[]
		| (
			"\n----\n\n"
			+ .package_name
			+ " version "
			+ .version
			+ "\n"
			+ .conclusion.Ok.full_license_document
		)
	'
