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
subxt metadata --pallets IPOnchain --url ws://127.0.0.1:34299 > ip_onchain_metadata.scale
```