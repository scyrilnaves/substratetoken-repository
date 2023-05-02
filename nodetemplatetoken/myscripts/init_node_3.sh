#!/bin/bash

cd "$(dirname "$0")" # goto file directory

#purge if necessary
# ../target/release/node-template purge-chain --base-path $HOME/datas/node-3 --chain ../myspec/customSpecRaw.json
#OR Force delete with:
rm -rf $HOME/datas/node-3/*;
################only this once
mkdir -p $HOME/datas/
#insert Sr25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-3 \
  --chain ../myspec/customSpecRaw.json \
  --key-type aura \
  --scheme Sr25519 \
  --suri "0xe59a7c2db8925536c204b991dda1d96d19bb449e6dfda5e1cb8d87faf55b3653";
#insert Ed25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-3 \
  --chain ../myspec/customSpecRaw.json \
  --key-type gran \
  --scheme Ed25519 \
  --suri "0xe59a7c2db8925536c204b991dda1d96d19bb449e6dfda5e1cb8d87faf55b3653";
