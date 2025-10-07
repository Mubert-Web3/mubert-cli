use clap::{CommandFactory, Parser, Subcommand};
use mubert_cli::ip_onchain_runtime::ip_onchain::calls::types::create_authority;

use subxt::utils::AccountId32;

#[derive(Parser)]
#[command(
    name = "mubert-cli",
    about = "Mubert cli",
    long_about = None,
    version = "0.0.1",
    author = "Mubert"
)]
struct Cli {
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    node_url: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    UploadIP {
        #[arg(long)]
        api_auth: String,
        #[arg(short = 'f', long)]
        file: std::path::PathBuf,
        #[arg(short = 'd', long, help = "data as plain json")]
        data: Option<String>,
        #[arg(short = 'j', long)]
        data_file: Option<std::path::PathBuf>,
        #[arg(short = 's', long)]
        secret_key_file: Option<std::path::PathBuf>,
        #[arg(long)]
        arweave_worker_address: Option<AccountId32>,
    },
    CreateAuthority {
        #[arg(short = 'n', long)]
        name: String,
        #[arg(value_enum, short = 'k', long)]
        kind: create_authority::AuthorityKind,
        #[arg(short = 's', long)]
        secret_key_file: Option<std::path::PathBuf>,
    },
    ForeignRequest {
        #[arg(long, help = "data as plain json")]
        data: Option<String>,
        #[arg(short = 'j', long)]
        data_file: Option<std::path::PathBuf>,
        #[arg(long)]
        secret_key_file: Option<std::path::PathBuf>,
        #[arg(short = 's', long, help = "foreign location parachain id")]
        src_parachain_id: u32,
        #[arg(short = 'd', long, help = "parachain id where source entity exists")]
        dst_parachain_id: u32,
    },
    ForeignRequestApprove {
        #[arg(long)]
        secret_key_file: Option<std::path::PathBuf>,
        #[arg(short = 'e', long)]
        entity_id: u32,
        #[arg(short = 'r', long)]
        request_id: u32,
    },
    ForeignRequestTake {
        #[arg(long)]
        secret_key_file: Option<std::path::PathBuf>,
        #[arg(short = 'r', long)]
        request_id: u32,
        #[arg(short = 'd', long, help = "parachain id where source entity exists")]
        dst_parachain_id: u32,
    },
    GetForeignRequest {
        #[arg(short = 'r', long)]
        request_id: u32,
    },
    GetEntity {
        #[arg(short = 'e', long)]
        entity_id: u32,
    },
    GetAuthority {
        #[arg(short = 'a', long)]
        authority_id: u32,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.command.is_some() {
        match &cli.command {
            Some(Commands::UploadIP {
                api_auth,
                file,
                data,
                data_file,
                secret_key_file,
                arweave_worker_address,
            }) => {
                mubert_cli::update_ip::update_ip(
                    &cli.node_url,
                    api_auth,
                    file,
                    data,
                    data_file,
                    secret_key_file,
                    arweave_worker_address,
                )
                .await?;
            }
            Some(Commands::CreateAuthority {
                name,
                kind,
                secret_key_file,
            }) => {
                mubert_cli::create_authority::create_authority(
                    &cli.node_url,
                    name,
                    kind.clone(),
                    secret_key_file,
                )
                .await?;
            }
            Some(Commands::ForeignRequest {
                data,
                data_file,
                secret_key_file,
                src_parachain_id,
                dst_parachain_id,
            }) => {
                mubert_cli::foreign_request::foreign_request_to(
                    &cli.node_url,
                    data,
                    data_file,
                    secret_key_file,
                    *src_parachain_id,
                    *dst_parachain_id,
                )
                .await?;
            }
            Some(Commands::ForeignRequestApprove {
                secret_key_file,
                entity_id,
                request_id,
            }) => {
                mubert_cli::foreign_request::foreign_request_approve(
                    &cli.node_url,
                    secret_key_file,
                    *entity_id,
                    *request_id,
                )
                .await?;
            }
            Some(Commands::ForeignRequestTake {
                secret_key_file,
                request_id,
                dst_parachain_id,
            }) => {
                mubert_cli::foreign_request::foreign_request_take(
                    &cli.node_url,
                    secret_key_file,
                    *request_id,
                    *dst_parachain_id,
                )
                .await?;
            }
            Some(Commands::GetForeignRequest { request_id }) => {
                mubert_cli::foreign_request::foreign_request(&cli.node_url, *request_id).await?;
            }
            Some(Commands::GetEntity { entity_id }) => {
                mubert_cli::update_ip::get_entity(&cli.node_url, *entity_id).await?;
            }
            Some(Commands::GetAuthority { authority_id }) => {
                mubert_cli::create_authority::get_authority(&cli.node_url, *authority_id).await?;
            }
            None => {
                Cli::command().print_help().unwrap();
            }
        };
    }

    Ok(())
}
