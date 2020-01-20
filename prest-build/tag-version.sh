#!/bin/bash

set -euo pipefail
cd "$(dirname $0)/.."

version_rust="$(grep '^version' core/Cargo.toml | cut -d \" -f2)"
version_git="$(git describe --always)"

echo "= Prest release tagging script ="
echo "You can press ^C anytime to cancel."
echo ""
echo "Current rust version: ${version_rust}"
echo "Current git version: ${version_git}"
echo ""
echo -n "Enter new version (omitting the 'v'): "
read version

# check CHANGELOG
if ! head -n1 CHANGELOG.md | grep "${version}" &>/dev/null; then
    echo "Version ${version} not described in CHANGELOG.md, quitting."
    exit 1
fi

# bump rust version
sed -E -i -e "s/^version = \"${version_rust}\"/version = \"${version}\"/" core/Cargo.toml

# commit
echo ""
git commit core/Cargo.toml -m "Bump core version to ${version}."

# tag, annotated
git tag -a -m '' "v${version}"

echo ""
echo "Done! Don't forget to git push --tags origin master!"
echo "Other than that, now you're free to run build/build.sh"
echo "to get your fresh build of v${version}."
