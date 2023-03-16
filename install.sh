#!/bin/bash
# This is script shows how you can install the usermgmt
# tool on a Mac with M1 using a prebuilt release
# from GitHub. 
#
# Author: Dominik Wagner, 2022

version=v0.4.8
arch=aarch64
target_dir=/Users/$USER/usermgmt

echo "Installing usermgmt version $version for $arch architecture"

mkdir -p $target_dir
wget https://github.com/th-nuernberg/usermgmt/releases/download/${version}/usermgmt-${arch}-apple-darwin.tar.gz || exit 1

tar -xf usermgmt-${arch}-apple-darwin.tar.gz -C $target_dir || exit 1

rm usermgmt-${arch}-apple-darwin.tar.gz
