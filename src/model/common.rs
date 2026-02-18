use super::Block;
use crate::types::BlockId;
use serde::{Deserialize, Serialize};

/// Common fields for all blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockCommon {
    pub id: BlockId,
    pub children: Vec<Block>,
    pub has_children: bool,
    pub archived: bool,
}

impl BlockCommon {
    #[allow(dead_code)]
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            children: Vec::new(),
            has_children: false,
            archived: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.has_children = !children.is_empty();
        self.children = children;
        self
    }
}

impl Default for BlockCommon {
    fn default() -> Self {
        Self {
            id: BlockId::new_v4(),
            children: Vec::new(),
            has_children: false,
            archived: false,
        }
    }
}
