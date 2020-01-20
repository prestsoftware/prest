#!/bin/bash

current_version="$(git describe --always)"

if [ -f version.txt ] ; then
	cached_version="$(cat version.txt)"

	if [ "$current_version" != "$cached_version" ]; then
		echo "'$current_version' != '$cached_version'"
		echo "$current_version" > version.txt
	else
		echo "up to date at $current_version"
	fi
else
	echo "version.txt not found"
	echo "$current_version" > version.txt
fi
