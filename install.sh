#!/bin/bash
# This is script shows how you can install the usermgmt
# tool on a Mac with M1 chip using a prebuilt release
# from GitHub. 
#
# Author: Dominik Wagner, 2022

version=v0.4.1
arch=aarch64
target_dir=/Users/$USER/usermgmt

echo "Installing usermgmt version $version for $arch architecture"

mkdir -p $target_dir
wget https://github.com/th-nuernberg/usermgmt/releases/download/${version}/usermgmt-${arch}-apple-darwin.tar.gz || exit 1

tar -xf usermgmt-${arch}-apple-darwin.tar.gz -C $target_dir || exit 1

rm usermgmt-${arch}-apple-darwin.tar.gz

if [[ ":$PATH:" == *":$target_dir:"* ]]; then
  echo "Not setting path"
else
  echo "Adding $target_dir to PATH"
  if [[ -f "/Users/$USER/.zshrc" ]]; then
     echo 'export PATH=$PATH:'$target_dir >> ~/.zshrc
  fi  
  if [[ -f "/Users/$USER/.bashrc" ]]; then
     echo 'export PATH=$PATH:'$target_dir >> ~/.bashrc
  fi
fi

