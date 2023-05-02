#!/bin/bash

cd "$(dirname "$0")" # goto file directory

#purge if necessary
# ../target/release/node-template purge-chain --base-path $HOME/datas/node-1 --chain ../myspec/customSpecRaw.json
#OR Force delete with:
rm -rf $HOME/datas/node-1/*;
################only this once
mkdir -p $HOME/datas/
#insert Sr25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-1 \
  --chain ../myspec/customSpecRaw.json \
  --key-type aura \
  --scheme Sr25519 \
  --suri "0x15d3e38d1d81b1e2ab96fb192d75f7791fcded85e34dcc3e236475bc41af8885";
#insert Ed25519 key
../target/release/node-template key insert \
  --base-path $HOME/datas/node-1 \
  --chain ../myspec/customSpecRaw.json \
  --key-type gran \
  --scheme Ed25519 \
  --suri "0x15d3e38d1d81b1e2ab96fb192d75f7791fcded85e34dcc3e236475bc41af8885";
