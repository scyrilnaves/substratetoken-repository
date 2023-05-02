#!/bin/bash

cd "$(dirname "$0")" # goto file directory

../target/release/node-template \
  --base-path $HOME/datas/node-3/ \
  --name Node-3 \
  --chain ../myspec/customSpecRaw.json \
  --keystore-path $HOME/datas/node-3/chains/Hackathon_PoC_testnet/keystore/ \
  --port 30333 \
  --ws-port 9945 \
  --rpc-cors=all \
  --unsafe-ws-external \
  --rpc-port 9933 \
  --unsafe-rpc-external \
  --node-key e59a7c2db8925536c204b991dda1d96d19bb449e6dfda5e1cb8d87faf55b3653 \
  --validator \
  --bootnodes /ip4/172.16.10.11/tcp/30333/p2p/12D3KooWAHYHHAHKHH3uDCKgKQNEVRjm3dZNEzonSJpXxQyB3fk1
