use p2panda_core::PublicKey;
use p2panda_net::TopicId;
use p2panda_sync::topic_log_sync::TopicLogMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash as StdHash;
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, StdHash, Serialize, Deserialize)]
pub enum LogType {
    Snapshot,
    #[default]
    Delta,
}

#[derive(Clone, Debug, PartialEq, Eq, StdHash, Serialize, Deserialize)]
pub struct LogId(LogType, TopicId);

impl LogId {
    pub fn new(log_type: LogType, topic: &TopicId) -> Self {
        Self(log_type, *topic)
    }
}

pub type Logs<L> = HashMap<PublicKey, Vec<L>>;

#[derive(Clone, Debug)]
pub struct TopicStore {
    inner: Arc<RwLock<InnerTopicMap>>,
}

#[derive(Clone, Debug)]
struct InnerTopicMap {
    topics: HashMap<TopicId, AuthorLogs>,
}

type AuthorLogs = HashMap<PublicKey, Vec<LogId>>;

impl Default for TopicStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerTopicMap {
                topics: HashMap::new(),
            })),
        }
    }

    pub async fn add_log(&self, topic: &TopicId, public_key: &PublicKey, log_id: &LogId) {
        {
            let mut lock = self.inner.write().unwrap();
            lock.topics
                .entry(topic.clone())
                .and_modify(|author_logs| {
                    author_logs
                        .entry(*public_key)
                        .and_modify(|logs| {
                            if !logs.contains(log_id) {
                                logs.push(log_id.clone());
                            }
                        })
                        .or_insert(vec![log_id.clone()]);
                })
                .or_insert(HashMap::from([(*public_key, vec![log_id.clone()])]));
        }
    }
}

impl TopicLogMap<TopicId, LogId> for TopicStore {
    type Error = std::io::Error;

    async fn get(&self, topic: &TopicId) -> Result<Logs<LogId>, Self::Error> {
        // For persisted topics we get all logs for that topic.
        let lock = self.inner.read().unwrap();
        let topics = lock.topics.get(topic).cloned();

        match topics {
            Some(author_logs) => Ok(author_logs),
            None => Ok(HashMap::new()),
        }
    }
}
