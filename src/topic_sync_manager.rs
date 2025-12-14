use p2panda_net::TopicId;
use p2panda_store::MemoryStore;

use crate::{
    extensions::TestAppExtensions,
    topic_store::{LogId, TopicStore},
};

pub type TopicSyncManager = p2panda_sync::managers::topic_sync_manager::TopicSyncManager<
    TopicId,
    MemoryStore<LogId, TestAppExtensions>,
    TopicStore,
    LogId,
    TestAppExtensions,
>;
