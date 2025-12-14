use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestAppExtensions {
    // /// If flag is true we can remove all previous operations in this log.
    // ///
    // /// This usually indicates that a "snapshot" has been inserted into the body of this operation,
    // /// containing all required state to reconstruct the full version including all previous edits
    // /// of this topic.
    // ///
    // /// In our case of a text-editor, this would be the encoded payload of a state-based CRDT.
    // #[serde(
    //     rename = "p",
    //     skip_serializing_if = "PruneFlag::is_not_set",
    //     default = "PruneFlag::default"
    // )]
    // pub prune_flag: PruneFlag,

    // /// Operations can be organised in separate logs. With a "log id" we can declare where this
    // /// operation belongs to.
    // ///
    // /// We organise two logs per author per topic, one for "short lived" / ephemeral deltas
    // /// (small text changes) and one for persisted snapshots (full topic history). These are two
    // /// distinct "log types".
    // #[serde(rename = "t")]
    // pub log_type: LogType,

    // /// Identifier of the topic this operation relates to.
    // #[serde(rename = "d")]
    // pub topic: TopicId,
}
