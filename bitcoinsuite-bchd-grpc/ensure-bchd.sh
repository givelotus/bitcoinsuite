#!/bin/bash
set -ex

DL_DIR="../downloads"
DL_BCHD_FOLDER="$DL_DIR/bchd"

if [ -d "$DL_BCHD_FOLDER" ]; then
    pushd $DL_BCHD_FOLDER
    git pull --ff-only
else
    git clone https://github.com/gcash/bchd $DL_BCHD_FOLDER
    pushd $DL_BCHD_FOLDER
fi

go build
popd
