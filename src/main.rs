use std::sync::LazyLock;

use crate::{
    extensions::TestAppExtensions,
    topic_store::{LogId, TopicStore},
    topic_sync_manager::TopicSyncManager,
};
use p2panda_core::{Hash, PrivateKey};
use p2panda_discovery::address_book::memory::MemoryStore as MemoryAddressBook;
use p2panda_net::{MdnsDiscoveryMode, Network, NetworkBuilder};
use p2panda_store::MemoryStore;
use p2panda_sync::managers::topic_sync_manager::TopicSyncManagerConfig;
use rand_chacha::rand_core::SeedableRng;

mod extensions;
mod topic_store;
mod topic_sync_manager;

static RELAY_URL: LazyLock<iroh::RelayUrl> = LazyLock::new(|| {
    "https://euc1-1.relay.n0.iroh-canary.iroh.link"
        .parse()
        .expect("valid relay URL")
});

fn main() {
    println!("Test app for p2panda-net-next");

    let network_id = Hash::new(b"p2panda-next-test");
    let private_key = PrivateKey::new();
    let topic_store = TopicStore::new();

    let network = setup_network(&private_key, &network_id, &topic_store);
}

async fn setup_network(
    private_key: &PrivateKey,
    network_id: &Hash,
    topic_store: &TopicStore,
) -> Option<Network<TopicSyncManager>> {
    let address_book = MemoryAddressBook::new(rand_chacha::ChaCha20Rng::from_os_rng());

    // if let Err(error) = address_book.insert_node_info(BOOTSTRAP_NODE.clone()).await {
    //     error!("Failed to add bootstrap node to the address book: {error}");
    // }

    let memory_store = MemoryStore::<LogId, TestAppExtensions>::new();

    let sync_conf = TopicSyncManagerConfig {
        store: memory_store,
        topic_map: topic_store.clone(),
    };
    let network = NetworkBuilder::new(network_id.into())
        .private_key(private_key.clone())
        .mdns(MdnsDiscoveryMode::Active)
        .relay(RELAY_URL.clone())
        .build(address_book, sync_conf)
        .await;

    if let Err(error) = network {
        eprintln!("Failed to startup network: {}", error);
        None
    } else {
        network.ok()
    }
}
