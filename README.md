# Mubert Parachain CLI

## Install

```bash
cargo install --git https://github.com/Mubert-Web3/mubert-cli --branch main
```

## Prerequisites

Create a test account

```bash
subkey generate --output-type json --scheme sr25519 > test_secret_key.json
```

## Examples

### create-authority

```bash
mubert-cli create-authority \
--name=test \
--kind=musician \
--secret-key-file=./test_secret_key.json
```

### upload-ip

create_entity.json example

```json
{
  "entity_kind": "Track",
  "authority_id": 0,
  "metadata_standard": "M25",
  "flags": [
    "Immutable"
  ],
  "off_chain_metadata": {
    "title": "",
    "bpm": 120,
    "key": 1,
    "scale": 0,
    "instrument": 1
  }
}
```

```bash
mubert-cli upload-ip \
--api-auth='YOUR-BEARER-TOKEN' \
--file=./music.wav \
--data-file=./examples/create_entity.json \
--secret-key-file=./test_secret_key.json
```

## upload-ip with uploading metadata to arweave
```bash
mubert-cli upload-ip \
--api-auth='YOUR-BEARER-TOKEN' \
--file=./music.wav \
--data-file=./examples/create_entity.json \
--arweave-worker-address='worker-address-who-can-upload-file-to-arweave'
```

## Update pallet api metadata

```bash
subxt metadata --pallets IPOnchain --url ws://127.0.0.1:9944 > ip_onchain_metadata.scale
```

## Move entity to your parachain

### Prerequisites

Your chain is on  ws://127.0.0.1:9945 id=4725
Chain with entity ws://127.0.0.1:9944 id=4724

* create authority on your chain
```bash
mubert-cli --node-url=ws://127.0.0.1:9945 create-authority --name=test --kind=musician
```

* create authority and entity on other chain
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 create-authority --name=test --kind=musician
```
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 upload-ip --api-auth='not-needed' --file=./music.wav --data-file=./examples/create_entity_no_upload.json
```

### Transfer

Make a request for you entity, foreign_authority_id authority id on you chain to which the entity will be transferred
```bash
mubert-cli --node-url=ws://127.0.0.1:9945 foreign-request --data='{"foreign_authority_id":0,"foreign_authority_name":"foreign_authority","entity_id":0}' --src-parachain-id=4725 --dst-parachain-id=4724
```

See a request
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 get-foreign-request --request-id=0 | jq
```
See foreign authority
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 get-authority --authority-id=1 | jq
```

On other chain authority must approve you request
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 foreign-request-approve --entity-id=0 --request-id=0 
```

After this, you can move the entity to your chain
```bash
mubert-cli --node-url=ws://127.0.0.1:9945 foreign-request-take --request-id=0 --dst-parachain-id=4724
```

See a done request
```bash
mubert-cli --node-url=ws://127.0.0.1:9944 get-foreign-request --request-id=0 | jq
```

See a new entity
```bash
mubert-cli --node-url=ws://127.0.0.1:9945 get-entity --entity-id=0 | jq 
```
