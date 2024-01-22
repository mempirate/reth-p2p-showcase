use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use reth_network::{
    config::rng_secret_key, NetworkConfig, NetworkEvents, NetworkManager, PeersConfig,
};
use reth_p2p::init_tracing;
use reth_primitives::mainnet_nodes;
use reth_provider::test_utils::NoopProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    // Generate a random ECDSA private key.
    let secret = rng_secret_key();

    // // Create a peer configuration with max 10 outbound and 10 inbound peers.
    let peer_config = PeersConfig::default()
        .with_max_outbound(10)
        .with_max_inbound(10);

    // Add the peer configuration here.
    let builder = NetworkConfig::builder(secret)
        .disable_dns_discovery()
        .boot_nodes(mainnet_nodes())
        .peer_config(peer_config);

    // Build the config. The configuration needs a client to interact with the chain (to respond to block header
    // and block bodies requests), but for now we'll just use a no-op client.
    let config = builder.build(Arc::new(NoopProvider::default()));
    let mut network = NetworkManager::new(config).await?;

    // Create the channels for receiving eth messages
    let (eth_tx, mut eth_rx) = mpsc::channel(32);
    let (transaction_tx, mut transaction_rx) = mpsc::unbounded_channel();

    network.set_eth_request_handler(eth_tx);
    network.set_transactions(transaction_tx);

    let network_handle = network.handle().clone();
    let mut network_events = network_handle.event_listener();
    println!("Starting network manager...");
    tokio::spawn(network);

    loop {
        tokio::select! {
            Some(tx_event) = transaction_rx.recv() => {
                println!("New transaction event: {:?}", tx_event);
            }

            Some(eth_req) = eth_rx.recv() => {
                println!("New eth protocol request: {:?}", eth_req);
            }

            Some(net_event) = network_events.next() => {
                println!("New network event: {:?}", net_event);
            }
        }
    }
}
