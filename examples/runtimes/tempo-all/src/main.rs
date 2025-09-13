use std::path::PathBuf;

use anyhow::Result;
use res_azkeyvault::AzKeyVault;
use res_mongodb::MongoDb;
use res_nats::Nats;
use runtime::{AddResource, Cli, Command, Parser, ResourceBuilder, Runtime};
use wasi_blobstore_mdb::Blobstore;
use wasi_http::Http;
use wasi_keyvalue_nats::KeyValue;
use wasi_messaging_nats::Messaging;
use wasi_otel::Otel;
use wasi_vault_az::Vault;

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Run { wasm } => init_runtime(wasm).await,
        #[cfg(feature = "compile")]
        Command::Compile { wasm, output } => runtime::compile(&wasm, output),
    }
}

async fn init_runtime(wasm: PathBuf) -> Result<()> {
    // create resources (in parallel)
    let (mongodb, az_secret, nats) = tokio::join!(MongoDb::new(), AzKeyVault::new(), Nats::new());
    let nats = nats?;

    Runtime::new(wasm)
        .register(Otel)
        .register(Http)
        .register(Blobstore.resource(mongodb?)?)
        .register(KeyValue.resource(nats.clone())?)
        .register(Vault.resource(az_secret?)?)
        .register(Messaging.resource(nats)?)
        .await
}
