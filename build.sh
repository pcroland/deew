#!/bin/bash
pyinstaller.exe -F deew.py --icon logo/icon.ico
version=$(python -c 'from version import prog_version; print(prog_version)')
mkdir -p release
mv dist/deew.exe release
cp config.toml.example release
cd release
zip -r "../deew_$version.zip" *
cd ..
rm -rf build dist release deew.spec
