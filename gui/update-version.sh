#!/bin/bash

current_version="$(git describe)"
cached_version="$(cat version.txt)"

if [ "$current_version" != "$cached_version" ]; then
	echo "'$current_version' != '$cached_version'"
	echo "$current_version" > version.txt
else
	echo "up to date at $current_version"
fi
