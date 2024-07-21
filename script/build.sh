#!/usr/bin/env bash

if [ ! -d ./.git ]; then
    echo 'This script should be run from the project root!'
    exit 1
fi

bin_dir="$PWD/bin"
target_dir="$PWD/target/release"

target_triple="$(rustup target list | grep 'installed')"
target_triple=${target_triple%' (installed)'}

echo -n 'Compiling... '

cargo build --release &> /dev/null || exit $?

echo -ne 'Done.\nCopying... '

if [ ! -d "$bin_dir" ]; then
    mkdir "$bin_dir"
elif [ -n "$(ls -A "$bin_dir")" ]; then
    rm --interactive=once "$bin_dir"/*
fi

function copy() {
    local binary="$1"
    local version

    version="$(eval "$target_dir/$binary -V" | sed "s/$binary //")"

    local semver_suffix="-$version+$target_triple"

    cp "$target_dir/$binary" "$bin_dir/$binary$semver_suffix"
}

copy 'geolocate-cli'
copy 'geolocate-data'

echo -ne 'Done.\nStripping... '

strip "$bin_dir"/*

echo 'Done.'

escaped_bin_dir="$(echo "$bin_dir" | sed 's/\//\\\//g')"
checksums="$(sha256sum "$bin_dir"/* | sed "s/$escaped_bin_dir\///")"

echo -ne '\nChecksums'

if [ -n "$(which clip.exe)" ]; then
    echo -n "$checksums" | clip.exe
    echo -n ' (also sent to clipboard)'
elif [ -n "$(which xclip)" ]; then
    echo -n "$checksums" | xclip -selection clipboard
    echo -n ' (also sent to clipboard)'
fi

echo -e ":\n$checksums"
echo -e "\nFiles located within: '$bin_dir'"
