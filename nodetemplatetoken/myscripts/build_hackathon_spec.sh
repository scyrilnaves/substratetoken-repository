#!/bin/bash

NBNODES=3 # don't change

cd "$(dirname "$0")" # goto file directory

echo "Loading key file"
#load keys
chmod +x keys_file.sh
source keys_file.sh

echo "Build initial spec from local template: myspec/customSpec.json"

../target/release/node-template build-spec --disable-default-bootnode --chain local > ../myspec/customSpec.json

chainSpec=$(cat ../myspec/customSpec.json) #get file content

###################### make palletAura authorities (Sr25519 keys)
#use generated keys
palletAura_authorities="["
for (( i=0; i<$NBNODES; i++ )) # start 1 => no bootnode
do
palletAura_authorities+=$(cat <<EOF
    "${Sr25519_arr_ss58PublicKey[i]}",

EOF
)

done
palletAura_authorities=${palletAura_authorities::-1} #DON'T FORGET TO REMOVE LAST CHARACTER: ${palletAura_authorities::-1}
palletAura_authorities+="]"
palletAura_authorities=$(echo "$palletAura_authorities" | jq -c) #format json to a one line
###################### end make palletAura authorities



###################### make palletGrandpa authorities (Ed25519 keys)
#use generated keys
palletGrandpa_authorities="["
for (( i=0; i<$NBNODES; i++ )) # start 1 => no bootnode
do
palletGrandpa_authorities+=$(cat <<EOF
    [
    "${Ed25519_arr_ss58PublicKey[i]}",
    1
    ],

EOF
)

done
palletGrandpa_authorities=${palletGrandpa_authorities::-1} #DON'T FORGET TO REMOVE LAST CHARACTER: ${palletGrandpa_authorities::-1}
palletGrandpa_authorities+="]"
palletGrandpa_authorities=$(echo "$palletGrandpa_authorities" | jq -c) #format json to a one line
###################### end make palletGrandpa authorities



#edit json to replace the two arrays
#jq
chainSpec=$(echo $chainSpec | jq ".genesis.runtime.aura.authorities = ${palletAura_authorities}")
chainSpec=$(echo $chainSpec | jq ".genesis.runtime.grandpa.authorities = ${palletGrandpa_authorities}")
chainSpec=$(echo $chainSpec | jq '.name = "Hackathon PoC Chain"')
chainSpec=$(echo $chainSpec | jq '.id = "Hackathon_PoC_testnet"')

echo $chainSpec | jq > ../myspec/customSpec.json #write changes to file


#build raw chainSpec
echo "Build raw spec from ../myspec/customSpec.json"
../target/release/node-template build-spec --chain=../myspec/customSpec.json --raw --disable-default-bootnode > ../myspec/customSpecRaw.json

chainSpecRaw=$(cat ../myspec/customSpecRaw.json) #get file content 
echo $chainSpecRaw | jq | sed 's/^/      /' > ../myspec/customSpecRaw.json #write changes to file and add indentation
