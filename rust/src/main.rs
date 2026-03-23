use anyhow::{Ok, Result};
use ethers::providers::{Provider, Ipc};
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

use rust::constants::Env;
use rust::strategy::event_handler;
use rust::streams::{
    stream_new_blocks, stream_pending_transactions, stream_uniswap_v2_events, Event,
};
use rust::utils::setup_logger;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup_logger()?;

    let env = Env::new();

    // Start async websocket streams
    use ethers::providers::Ipc;
    let ipc = Ipc::connect(env.rpc_sock).await?;
    let provider = Arc::new(Provider::new(ipc));

    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    set.spawn(stream_new_blocks(provider.clone(), event_sender.clone()));
    // we're not using the mempool data here, but uncomment it to use pending txs
    // set.spawn(stream_pending_transactions(
    //     provider.clone(),
    //     event_sender.clone(),
    // ));
    set.spawn(event_handler(provider.clone(), event_sender.clone()));

    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    Ok(())
}
