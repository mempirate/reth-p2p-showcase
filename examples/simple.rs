use std::sync::Arc;
use tokio_stream::StreamExt;

use reth_network::{config::rng_secret_key, NetworkConfig, NetworkEvents, NetworkManager};
use reth_p2p::init_tracing;
use reth_primitives::mainnet_nodes;
use reth_provider::test_utils::NoopProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    // Generate a random ECDSA private key.
    let secret = rng_secret_key();

    // Create the network builder with the secret key, which allows you to configure the network.
    // The defaults will do for now.
    let builder = NetworkConfig::builder(secret)
        .disable_dns_discovery()
        .boot_nodes(mainnet_nodes());

    // Build the config. The configuration needs a client to interact with the chain (to respond to block header
    // and block bodies requests), but for now we'll just use a no-op client.
    let config = builder.build(Arc::new(NoopProvider::default()));
    let network = NetworkManager::new(config).await?;

    // Get a handle to the network manager
    let network_handle = network.handle().clone();
    // Subscribe to network events
    let mut network_events = network_handle.event_listener();
    println!("Starting network manager...");
    // Spawn the network manager task. This will start the network manager
    // and all the subcomponents necessary for the devp2p stack.
    tokio::spawn(network);

    while let Some(net_event) = network_events.next().await {
        println!("Received network event: {:?}", net_event);
    }

    Ok(())
}
