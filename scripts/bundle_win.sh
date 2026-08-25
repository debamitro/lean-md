#!/bin/bash

ver=`cat version`

cargo build --release --target x86_64-pc-windows-gnu
cd target/x86_64-pc-windows-gnu/release/
zip -r ../../../lean_md-${ver}-x86_64-windows.zip lean-md.exe
cd ../../../

exit 0