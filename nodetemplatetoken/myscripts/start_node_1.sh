#!/bin/bash

cd "$(dirname "$0")" # goto file directory

#Node libp2p ID: 12D3KooWAHYHHAHKHH3uDCKgKQNEVRjm3dZNEzonSJpXxQyB3fk1
../target/release/node-template \
  --base-path $HOME/datas/node-1/ \
  --name Node-1 \
  --chain ../myspec/customSpecRaw.json \
  --keystore-path $HOME/datas/node-1/chains/Hackathon_PoC_testnet/keystore/ \
  --port 30333 \
  --ws-port 9945 \
  --rpc-cors=all \
  --unsafe-ws-external \
  --rpc-port 9933 \
  --unsafe-rpc-external \
  --node-key 15d3e38d1d81b1e2ab96fb192d75f7791fcded85e34dcc3e236475bc41af8885 \
  --validator
