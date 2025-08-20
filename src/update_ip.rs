use crate::api::MetadataRequest;
use crate::ip_onchain_runtime::ip_onchain::calls::types::create_entity::{
    MetadataFeatures, MetadataStandard,
};
use crate::ip_onchain_runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use crate::ip_onchain_runtime::runtime_types::pallet_arweave::types::TaskState;
use crate::ip_onchain_runtime::runtime_types::pallet_ip_onchain::types::{
    BitFlags, IPEntityKind, MetadataFeature, Wallet,
};
use crate::{api, calculate_flags, ip_onchain_runtime, SecretKeyFile};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::bip39::Mnemonic;
use subxt_signer::sr25519::{dev, Keypair};
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

#[derive(Serialize, Deserialize)]
struct CreateEntityFields {
    entity_kind: IPEntityKind,
    authority_id: u32,
    metadata_standard: MetadataStandard,
    flags: Vec<String>,
    authors_ids: Option<BoundedVec<u32>>,
    royalty_parts: Option<BoundedVec<Wallet<AccountId32>>>,
    related_entities_ids: Option<BoundedVec<u32>>,

    off_chain_metadata: OffChainMetadata,
    metadata_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct OffChainMetadata {
    pub title: String,
    pub bpm: u32,
    pub key: u8,
    pub scale: u8,
    pub instrument: u8,
}

pub async fn update_ip(
    node_url: &String,
    api_auth: &String,
    file: &PathBuf,
    data: &Option<String>,
    data_file: &Option<PathBuf>,
    secret_key_file: &Option<PathBuf>,
    arweave_worker_address: &Option<AccountId32>,
) -> Result<(), Box<dyn Error>> {
    // parsing a arguments
    let data = match (data, data_file) {
        (Some(data), None) => Ok(data.to_string()),
        (None, Some(data_file)) => {
            let data_file = std::fs::read_to_string(data_file)
                .map_err(|e| format!("read data_file {:?}: {e}", data_file))?;
            Ok(data_file)
        }
        _ => Err("no data file given"),
    }?;

    let req: CreateEntityFields =
        serde_json::from_str(data.as_str()).map_err(|e| format!("parsing json: {e}"))?;

    let flags: MetadataFeatures = MetadataFeatures::from(BitFlags(
        calculate_flags::<MetadataFeature>(req.flags),
        Default::default(),
    ));

    let mut sender_keypair = dev::alice();

    if let Some(secret_key_file) = secret_key_file {
        let secret_key_data = std::fs::read_to_string(secret_key_file)
            .map_err(|e| format!("read secret_key_file {:?}: {e}", secret_key_file))?;
        let secret_key: SecretKeyFile = serde_json::from_str(secret_key_data.as_str())
            .map_err(|e| format!("parsing json: {e}"))?;
        let mnemonic = Mnemonic::parse(secret_key.secret_phrase).unwrap();
        sender_keypair = Keypair::from_phrase(&mnemonic, None).unwrap();
    }

    // get rpc api client
    let api = OnlineClient::<PolkadotConfig>::from_url(node_url)
        .await
        .map_err(|e| format!("chain rpc api: {e}"))?;

    let metadata_url = match req.metadata_url {
        Some(off_chain_metadata_url) => off_chain_metadata_url,
        None => {
            // make off chain requests
            let job = api::upload_audio(file, api_auth)
                .await
                .map_err(|e| format!("upload_audio_fingerprint: {e}"))?;
            println!("fingerprint worker job id: {}", job.id);

            let fingerprint = api::wait_for_fingerprint_url(&job.id, 10, api_auth)
                .await
                .map_err(|e| format!("wait_for_fingerprint_url: {e}"))?;
            println!("fingerprint: {fingerprint}");

            let metadata_req = api::MetadataRequest {
                title: req.off_chain_metadata.title,
                bpm: req.off_chain_metadata.bpm,
                key: req.off_chain_metadata.key,
                scale: req.off_chain_metadata.scale,
                instrument: req.off_chain_metadata.instrument,
                fingerprint,
            };

            if let Some(arweave_worker_address) = arweave_worker_address {
                let metadata_url = upload_metadata_to_arweave(
                    &mut sender_keypair,
                    &api,
                    &metadata_req,
                    arweave_worker_address,
                )
                .await?;
                println!("Done! Arweave metadata url: {}", metadata_url);

                metadata_url
            } else {
                let off_chain_metadata = api::create_metadata(metadata_req, api_auth)
                    .await
                    .map_err(|e| format!("create_metadata: {e}"))?;
                println!("off chain metadata url: {}", off_chain_metadata.url);

                off_chain_metadata.url
            }
        }
    };

    let call = ip_onchain_runtime::tx().ip_onchain().create_entity(
        req.entity_kind,
        req.authority_id,
        metadata_url.into(),
        req.metadata_standard,
        flags,
        req.authors_ids,
        req.royalty_parts,
        req.related_entities_ids,
        None,
        None,
        None,
    );

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
        .map_err(|e| format!("can not finalize tx: {e}"))?;
    
    println!("wait events...");
    let events = finalized
        .fetch_events()
        .await
        .map_err(|e| format!("tx submitted, but not can not fetch events: {e}"))?;

    // check events
    if let Some(event) = events
        .find_first::<ip_onchain_runtime::ip_onchain::events::EntityAdded>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        println!("Entity added successful: {:?}", event);
    }
    Ok(())
}

async fn upload_metadata_to_arweave(
    sender_keypair: &mut Keypair,
    api: &OnlineClient<PolkadotConfig>,
    metadata_req: &MetadataRequest,
    arweave_worker_address: &AccountId32,
) -> Result<String, Box<dyn Error>> {
    println!("Starting arweave metadata upload");

    let data = serde_json::to_string(&metadata_req).unwrap();
    let call = ip_onchain_runtime::tx()
        .arweave()
        .create_task(arweave_worker_address.clone(), BoundedVec::from(data));

    println!("Submitting transaction...");
    let submited = api
        .tx()
        .sign_and_submit_then_watch_default(&call, sender_keypair)
        .await
        .map_err(|e| format!("can not submit tx: {e}"))?;

    println!("wait finalization...");
    let finalized = submited
        .wait_for_finalized()
        .await
        .map_err(|e| format!("tx submitted, but not finalized: {e}"))?;

    println!("wait events...");
    let events = finalized
        .fetch_events()
        .await
        .map_err(|e| format!("tx submitted, but not can not fetch events: {e}"))?;
    
    let task_id = match events
        .find_first::<ip_onchain_runtime::arweave::events::TaskAdded>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        Some(e) => {
            println!("Task added successful: task_id={:?}", e.task_id);
            e.task_id
        }
        None => return Err("unexpected event".into()),
    };

    let tasks_query = ip_onchain_runtime::storage().arweave().tasks(task_id);

    println!("Waiting for worker done task: may take a 5 min to validate");

    let result = Retry::spawn(FixedInterval::from_millis(6_000), || async {
        println!("Get task state: task_id={}", task_id);

        let tasks_details = api
            .storage()
            .at_latest()
            .await
            .unwrap()
            .fetch(&tasks_query)
            .await
            .unwrap()
            .ok_or("no task")
            .unwrap();

        println!("state: {:?}", tasks_details.state);
        if tasks_details.state != TaskState::Validate {
            println!("retry");
            return Err("task not in validate state, waiting");
        };

        if let Some(tx_hash) = tasks_details.tx_hash {
            let metadata_url = format!(
                "https://arweave.net/{}",
                String::from_utf8(tx_hash.0).unwrap()
            );

            return Ok(metadata_url);
        };

        Err("task in unexpected state")
    })
    .await;

    Ok(result.expect("can not get tx_hash from task"))
}
