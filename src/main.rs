use std::{str::from_utf8, sync::LazyLock};

use crate::{
    extensions::TestAppExtensions,
    operation_store::OperationStore,
    topic_store::{LogType, TopicStore},
    topic_sync_manager::TopicSyncManager,
};
use p2panda_core::{Body, Hash, Header, PrivateKey};
use p2panda_discovery::address_book::memory::MemoryStore as MemoryAddressBook;
use p2panda_net::{MdnsDiscoveryMode, Network, NetworkBuilder, TopicId, events::NetworkEvent};

use p2panda_sync::{
    managers::topic_sync_manager::TopicSyncManagerConfig, topic_log_sync::TopicLogSyncEvent,
};
use rand_chacha::rand_core::SeedableRng;
use tokio::sync::mpsc;

mod extensions;
mod operation_store;
mod topic_store;
mod topic_sync_manager;

static RELAY_URL: LazyLock<iroh::RelayUrl> = LazyLock::new(|| {
    "https://euc1-1.relay.n0.iroh-canary.iroh.link"
        .parse()
        .expect("valid relay URL")
});

#[tokio::main]
async fn main() {
    println!("Test app for p2panda-net-next");

    let network_id = Hash::new(b"p2panda-next-test");
    let private_key = PrivateKey::new();
    let topic_store = TopicStore::new();
    let operation_store = OperationStore::new();

    println!("Setting up Network ID: {:?}", hex::encode(network_id));
    let network = setup_network(&private_key, &network_id, &topic_store, &operation_store)
        .await
        .unwrap();

    println!("Subscribing to network events for debug reasons...");
    let network_events_rx = network.events().await.unwrap();
    tokio::spawn(async move {
        println!("Spawning task to handle network events...");
        let mut network_events_rx = network_events_rx;
        while let Ok(event) = network_events_rx.recv().await {
            match event {
                NetworkEvent::Transport(status) => {
                    println!("* Network event: Transport status changed: {:?}", status);
                }
                NetworkEvent::Relay(status) => {
                    println!("* Network event: Relay status changed: {:?}", status);
                }
                NetworkEvent::Discovery(_discovery_event) => {
                    // println!("* Discovery event: {:?}", discovery_event);
                }
            }
        }
    });

    let topic_id: TopicId = Hash::new(b"example-topic").into();

    println!("Subscribing to topic: {:?}", hex::encode(topic_id));
    let stream = match network.stream(topic_id, true).await {
        Ok(result) => {
            println!("+-- successfully created stream");
            result
        }
        Err(error) => {
            eprintln!("!-- Failed to create stream, aborting: {:?}", error);
            return;
        }
    };

    let mut topic_rx = stream.subscribe().await.unwrap();
    let topic_tx = stream;
    println!("+-- successfully created subscription");

    tokio::spawn(async move {
        println!("Spawning task to handle incoming messages...");
        while let Ok(event) = topic_rx.recv().await {
            match event.event() {
                TopicLogSyncEvent::Operation(operation) => {
                    match unpack_operation(operation.as_ref().to_owned()) {
                        (header, body, _raw_header) => {
                            println!(
                                "* Received operation on topic {:?} with header: {:?}",
                                topic_id, header
                            );
                            println!(
                                "  -- body raw bytes: {:?}",
                                from_utf8(&body.unwrap().to_bytes())
                            );
                        }
                    }
                }
                _ => {
                    // TODO: Handle sync events
                }
            }
        }
    });

    // Listen for text input via the terminal.
    let (line_tx, mut line_rx) = mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    // Sign and encode each line of text input and broadcast it on the chat topic.
    while let Some(text) = line_rx.recv().await {
        println!("User typed line: {}", text.trim_end());

        let result = operation_store
            .create_operation(
                &private_key,
                LogType::Snapshot,
                topic_id,
                Some(text.as_bytes()),
            )
            .await;

        match result {
            Ok(operation) => {
                println!(
                    "Created operation: header = {:?}, body = {:?}",
                    operation.header, operation.body
                );

                topic_tx.publish(operation).await.unwrap();
            }
            Err(error) => {
                eprintln!("Failed to create operation: {:?}", error);
            }
        }
    }
}

fn input_loop(line_tx: mpsc::Sender<String>) -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx
            .blocking_send(buffer.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to send line"))?;
        buffer.clear();
    }
}

async fn setup_network(
    private_key: &PrivateKey,
    network_id: &Hash,
    topic_store: &TopicStore,
    operation_store: &OperationStore,
) -> Option<Network<TopicSyncManager>> {
    let address_book = MemoryAddressBook::new(rand_chacha::ChaCha20Rng::from_os_rng());

    // if let Err(error) = address_book.insert_node_info(BOOTSTRAP_NODE.clone()).await {
    //     error!("Failed to add bootstrap node to the address book: {error}");
    // }

    let sync_conf = TopicSyncManagerConfig {
        store: operation_store.clone_inner(),
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

type OperationWithRawHeader = (Header<TestAppExtensions>, Option<Body>, Vec<u8>);

fn unpack_operation(
    operation: p2panda_core::Operation<TestAppExtensions>,
) -> OperationWithRawHeader {
    let p2panda_core::Operation::<TestAppExtensions> { header, body, .. } = operation;

    (header.clone(), body, header.to_bytes())
}

// async fn send_message(topic_tx: &TopicSyncManager, message: String) {
//     let body = Body::from(message.into_bytes());
//     let header = Header::<TestAppExtensions>::new(body.hash());

//     let operation = p2panda_core::Operation {
//         header: header.clone(),
//         body: Some(body),
//         extensions: TestAppExtensions {},
//     };

//     match topic_tx.publish_operation(operation).await {
//         Ok(_) => {
//             println!("Successfully sent message with header: {:?}", header);
//         }
//         Err(error) => {
//             eprintln!("Failed to send message: {:?}", error);
//         }
//     }
// }
