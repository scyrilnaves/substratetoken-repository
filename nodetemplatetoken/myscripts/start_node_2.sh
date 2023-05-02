#!/bin/bash

cd "$(dirname "$0")" # goto file directory

../target/release/node-template \
  --base-path $HOME/datas/node-2/ \
  --name Node-2 \
  --chain ../myspec/customSpecRaw.json \
  --keystore-path $HOME/datas/node-2/chains/Hackathon_PoC_testnet/keystore/ \
  --port 30333 \
  --ws-port 9945 \
  --rpc-cors=all \
  --unsafe-ws-external \
  --rpc-port 9933 \
  --unsafe-rpc-external \
  --node-key 95070113c3d34698e7849a7ba8ba9006b631370c5353c0dd440c60cdf2d69408 \
  --validator \
  --bootnodes /ip4/172.16.10.11/tcp/30333/p2p/12D3KooWAHYHHAHKHH3uDCKgKQNEVRjm3dZNEzonSJpXxQyB3fk1
