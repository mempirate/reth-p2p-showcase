use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio_stream::StreamExt;

use reth_discv4::{DiscoveryUpdate, Discv4, Discv4ConfigBuilder, NodeRecord};
use reth_network::{config::rng_secret_key, NetworkConfig, NetworkManager};
use reth_network_api::{PeerKind, Peers};
use reth_p2p::init_tracing;
use reth_primitives::mainnet_nodes;
use reth_provider::test_utils::NoopProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    // Generate a random ECDSA private key.
    let secret = rng_secret_key();

    // Disable the default discovery services.
    let builder = NetworkConfig::builder(secret)
        .disable_dns_discovery()
        .disable_discv4_discovery()
        .boot_nodes(mainnet_nodes());

    // Build the config. The configuration needs a client to interact with the chain (to respond to block header
    // and block bodies requests), but for now we'll just use a no-op client.
    let config = builder.build(Arc::new(NoopProvider::default()));
    let network = NetworkManager::new(config).await?;

    let peer_id = *network.peer_id();

    // Get a handle to the network manager
    let network_handle = network.handle().clone();
    println!("Starting network manager...");
    // Spawn the network manager task. This will start the network manager
    // and all the subcomponents necessary for the devp2p stack.
    tokio::spawn(network);

    // Build local node record
    let disc_addr = SocketAddr::from_str("0.0.0.0:30303").unwrap();
    let local_enr = NodeRecord {
        id: peer_id,
        address: disc_addr.ip(),
        tcp_port: disc_addr.port(),
        udp_port: disc_addr.port(),
    };

    // Create the discv4 config
    let discv4_config = Discv4ConfigBuilder::default()
        // Decrease lookup interval to 5 seconds (from 20 sec default)
        .lookup_interval(Duration::from_secs(5))
        // Decrease ban duration to 30 minutes
        .ban_duration(Some(Duration::from_secs(30 * 60)))
        .add_boot_nodes(mainnet_nodes())
        .build();

    let (_discv4, mut service) = Discv4::bind(disc_addr, local_enr, secret, discv4_config).await?;
    let mut disc_updates = service.update_stream();

    // Spawn the discv4 service
    let _handle = service.spawn();

    while let Some(disc_event) = disc_updates.next().await {
        match disc_event {
            DiscoveryUpdate::Added(enr) | DiscoveryUpdate::DiscoveredAtCapacity(enr) => {
                println!("Discovered new node: {:?}", enr);
                // Evaluate if we want to connect to peer
                if custom_peer_eval_func(&enr) {
                    network_handle.add_peer(enr.id, enr.tcp_addr());
                }
            }
            DiscoveryUpdate::Removed(id) => {
                network_handle.remove_peer(id, PeerKind::Basic);
            }
            _ => {}
        }
    }

    Ok(())
}

// This function could be used to add some custom peer evaluation logic,
// like a ping below n milliseconds.
fn custom_peer_eval_func(_enr: &NodeRecord) -> bool {
    true
}
