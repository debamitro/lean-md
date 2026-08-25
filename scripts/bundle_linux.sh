#!/bin/bash

ver=`cat version`

cargo zigbuild --release --target x86_64-unknown-linux-gnu
cd target/x86_64-unknown-linux-gnu/release/
zip -r ../../../lean_md-${ver}-x86_64-linux.zip lean-md
cd ../../../

exit 0