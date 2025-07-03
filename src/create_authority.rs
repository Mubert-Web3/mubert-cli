use crate::ip_onchain_runtime::ip_onchain::calls::types::create_authority;
use crate::{ip_onchain_runtime, SecretKeyFile};
use std::error::Error;
use std::path::PathBuf;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::bip39::Mnemonic;
use subxt_signer::sr25519::{dev, Keypair};

pub async fn create_authority(
    node_url: &String,
    name: &String,
    kind: create_authority::AuthorityKind,
    secret_key_file: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let call = ip_onchain_runtime::tx()
        .ip_onchain()
        .create_authority(name.clone().into(), kind);

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

    // make transaction and wait for event
    let tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&call, &sender_keypair)
        .await
        .map_err(|e| format!("can not submit tx: {e}"))?;
    let finalized = tx_progress.wait_for_finalized().await?;
    let events = finalized.fetch_events().await
        .map_err(|e| format!("tx submitted, but not validated: {e}"))?;
    
    // check events
    if let Some(event) = events
        .find_first::<ip_onchain_runtime::ip_onchain::events::AuthorityAdded>()
        .map_err(|e| format!("tx submitted, but event not found: {e}"))?
    {
        println!("Authority added successful: {:?}", event);
    }
    Ok(())
}
