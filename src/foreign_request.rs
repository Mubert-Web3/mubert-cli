use crate::{
    ip_onchain_runtime::{
        self,
        runtime_types::{
            staging_xcm::v5::{
                junction::Junction, junctions::Junctions, location::Location, Instruction, Xcm,
            },
            xcm::{
                double_encoded::DoubleEncoded,
                v3::{OriginKind, WeightLimit},
                VersionedLocation, VersionedXcm,
            },
        },
    },
    SecretKeyFile,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};
use subxt::{tx::Payload, utils::to_hex, OnlineClient, PolkadotConfig};
use subxt_signer::{
    bip39::Mnemonic,
    sr25519::{dev, Keypair},
};

#[derive(Serialize, Deserialize)]
struct SendForeignRequest {
    foreign_authority_id: u64,
    foreign_authority_name: String,

    entity_id: u32,
}
pub async fn foreign_request_to(
    node_url: &String,
    data: &Option<String>,
    data_file: &Option<PathBuf>,
    secret_key_file: &Option<PathBuf>,
    src_parachain_id: u32,
    dst_parachain_id: u32,
) -> Result<(), Box<dyn Error>> {
    let data = match (data, data_file) {
        (Some(data), None) => Ok(data.to_string()),
        (None, Some(data_file)) => {
            let data_file = std::fs::read_to_string(data_file)
                .map_err(|e| format!("read data_file {:?}: {e}", data_file))?;
            Ok(data_file)
        }
        _ => Err("no data file given"),
    }?;

    let req: SendForeignRequest =
        serde_json::from_str(data.as_str()).map_err(|e| format!("parsing json: {e}"))?;

    let call = ip_onchain_runtime::tx()
        .ip_onchain()
        .foreign_authority_request(
            req.foreign_authority_id,
            req.foreign_authority_name.into(),
            req.entity_id,
            Location {
                parents: 1,
                interior: Junctions::X1([Junction::Parachain(src_parachain_id)]),
            },
        );

    let mut sender_keypair = dev::alice();

    if let Some(secret_key_file) = secret_key_file {
        let secret_key_data = std::fs::read_to_string(secret_key_file)
            .map_err(|e| format!("read secret_key_file {:?}: {e}", secret_key_file))?;
        let secret_key: SecretKeyFile = serde_json::from_str(secret_key_data.as_str())
            .map_err(|e| format!("parsing json: {e}"))?;
        let mnemonic = Mnemonic::parse(secret_key.secret_phrase).unwrap();
        sender_keypair = Keypair::from_phrase(&mnemonic, None).unwrap();
    }

    let api = OnlineClient::<PolkadotConfig>::from_url(node_url)
        .await
        .map_err(|e| format!("chain rpc api: {e}"))?;

    let res = call.encode_call_data(&api.metadata())?;
    println!("to_hex: {}", to_hex(&res));

    let message = Xcm(vec![
        Instruction::UnpaidExecution {
            weight_limit: WeightLimit::Unlimited,
            check_origin: None,
        },
        Instruction::Transact {
            origin_kind: OriginKind::SovereignAccount,
            call: DoubleEncoded { encoded: res },
            fallback_max_weight: None,
        },
    ]);

    let xcm_call = ip_onchain_runtime::tx().polkadot_xcm().send(
        VersionedLocation::V5(Location {
            parents: 1,
            interior: Junctions::X1([Junction::Parachain(dst_parachain_id)]),
        }),
        VersionedXcm::V5(message),
    );

    let res = xcm_call.encode_call_data(&api.metadata())?;
    println!("to_hex: {}", to_hex(&res));

    println!("Submitting transaction...");
    let tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&xcm_call, &sender_keypair)
        .await
        .map_err(|e| format!("can not submit tx: {e}"))?;

    println!("wait finalization...");
    let finalized = tx_progress
        .wait_for_finalized()
        .await
        .map_err(|e| format!("tx submitted, but not finalize: {e}"))?;

    println!("wait events...");
    let events = finalized
        .fetch_events()
        .await
        .map_err(|e| format!("tx submitted, but not can not fetch events: {e}"))?;

    // check events
    if let Some(event) = events
        .find_first::<ip_onchain_runtime::polkadot_xcm::events::Sent>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        println!("xcm sent successful: {:?}", event);
    }

    Ok(())
}

pub async fn foreign_request_approve(
    node_url: &String,
    secret_key_file: &Option<PathBuf>,
    entity_id: u32,
    request_id: u32,
) -> Result<(), Box<dyn Error>> {
    let call = ip_onchain_runtime::tx()
        .ip_onchain()
        .foreign_authority_request_approve(entity_id, request_id);

    let mut sender_keypair = dev::alice();

    if let Some(secret_key_file) = secret_key_file {
        let secret_key_data = std::fs::read_to_string(secret_key_file)
            .map_err(|e| format!("read secret_key_file {:?}: {e}", secret_key_file))?;
        let secret_key: SecretKeyFile = serde_json::from_str(secret_key_data.as_str())
            .map_err(|e| format!("parsing json: {e}"))?;
        let mnemonic = Mnemonic::parse(secret_key.secret_phrase).unwrap();
        sender_keypair = Keypair::from_phrase(&mnemonic, None).unwrap();
    }

    let api = OnlineClient::<PolkadotConfig>::from_url(node_url)
        .await
        .map_err(|e| format!("chain rpc api: {e}"))?;

    let res = call.encode_call_data(&api.metadata())?;
    println!("to_hex: {}", to_hex(&res));

    println!("Submitting transaction...");
    let tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&call, &sender_keypair)
        .await
        .map_err(|e| format!("can not submit tx: {e}"))?;

    println!("wait finalization...");
    let finalized = tx_progress
        .wait_for_finalized()
        .await
        .map_err(|e| format!("tx submitted, but not finalize: {e}"))?;

    println!("wait events...");
    let events = finalized
        .fetch_events()
        .await
        .map_err(|e| format!("tx submitted, but not can not fetch events: {e}"))?;

    // check events
    if let Some(event) = events
        .find_first::<ip_onchain_runtime::ip_onchain::events::EntityWraped>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        println!("Authority added successful: {:?}", event);
    }

    Ok(())
}

pub async fn foreign_request_take(
    node_url: &String,
    secret_key_file: &Option<PathBuf>,
    request_id: u32,
    dst_parachain_id: u32,
) -> Result<(), Box<dyn Error>> {
    let call = ip_onchain_runtime::tx()
        .ip_onchain()
        .foreign_authority_request_take(
            request_id,
            Location {
                parents: 1,
                interior: Junctions::X1([Junction::Parachain(dst_parachain_id)]),
            },
        );

    let mut sender_keypair = dev::alice();

    if let Some(secret_key_file) = secret_key_file {
        let secret_key_data = std::fs::read_to_string(secret_key_file)
            .map_err(|e| format!("read secret_key_file {:?}: {e}", secret_key_file))?;
        let secret_key: SecretKeyFile = serde_json::from_str(secret_key_data.as_str())
            .map_err(|e| format!("parsing json: {e}"))?;
        let mnemonic = Mnemonic::parse(secret_key.secret_phrase).unwrap();
        sender_keypair = Keypair::from_phrase(&mnemonic, None).unwrap();
    }

    let api = OnlineClient::<PolkadotConfig>::from_url(node_url)
        .await
        .map_err(|e| format!("chain rpc api: {e}"))?;

    let res = call.encode_call_data(&api.metadata())?;
    println!("to_hex: {}", to_hex(&res));

    let message = Xcm(vec![
        Instruction::UnpaidExecution {
            weight_limit: WeightLimit::Unlimited,
            check_origin: None,
        },
        Instruction::Transact {
            origin_kind: OriginKind::SovereignAccount,
            call: DoubleEncoded { encoded: res },
            fallback_max_weight: None,
        },
    ]);

    let xcm_call = ip_onchain_runtime::tx().polkadot_xcm().send(
        VersionedLocation::V5(Location {
            parents: 1,
            interior: Junctions::X1([Junction::Parachain(dst_parachain_id)]),
        }),
        VersionedXcm::V5(message),
    );

    let res = xcm_call.encode_call_data(&api.metadata())?;
    println!("to_hex: {}", to_hex(&res));

    println!("Submitting transaction...");
    let tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&xcm_call, &sender_keypair)
        .await
        .map_err(|e| format!("can not submit tx: {e}"))?;

    println!("wait finalization...");
    let finalized = tx_progress
        .wait_for_finalized()
        .await
        .map_err(|e| format!("tx submitted, but not finalize: {e}"))?;

    println!("wait events...");
    let events = finalized
        .fetch_events()
        .await
        .map_err(|e| format!("tx submitted, but not can not fetch events: {e}"))?;

    // check events
    if let Some(event) = events
        .find_first::<ip_onchain_runtime::polkadot_xcm::events::Sent>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        println!("xcm sent successful: {:?}", event);
    }

    Ok(())
}

pub async fn foreign_request(node_url: &String, request_id: u32) -> Result<(), Box<dyn Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(node_url)
        .await
        .map_err(|e| format!("chain rpc api: {e}"))?;

    let query = ip_onchain_runtime::storage()
        .ip_onchain()
        .foreigns_requests(request_id);

    let details = api
        .storage()
        .at_latest()
        .await?
        .fetch(&query)
        .await?
        .ok_or("foreign_request not found")?;

    let data = serde_json::to_string(&details).unwrap();

    println!("{data}");

    Ok(())
}
