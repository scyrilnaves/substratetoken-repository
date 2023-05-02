#!/bin/bash

cd "$(dirname "$0")" # goto file directory

#purge if necessary
# ../target/release/node-template purge-chain --base-path $HOME/datas/node-2 --chain ../myspec/customSpecRaw.json
#OR Force delete with:
rm -rf $HOME/datas/node-2/*;
################only this once
mkdir -p $HOME/datas/
#insert Sr25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-2 \
  --chain ../myspec/customSpecRaw.json \
  --key-type aura \
  --scheme Sr25519 \
  --suri "0x95070113c3d34698e7849a7ba8ba9006b631370c5353c0dd440c60cdf2d69408";
#insert Ed25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-2 \
  --chain ../myspec/customSpecRaw.json \
  --key-type gran \
  --scheme Ed25519 \
  --suri "0x95070113c3d34698e7849a7ba8ba9006b631370c5353c0dd440c60cdf2d69408";
