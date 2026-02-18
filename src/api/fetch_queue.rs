// src/api/fetch_queue.rs
//! Work items for exploring a Notion content tree iteratively.

use super::types::{FetchContext, FetchMetadata, FetchObjective, FetchRequest};
use crate::error::AppError;
use crate::model::{Block, NotionObject, Page};
use crate::types::{NotionId, Warning};
use std::cmp::Ordering;
use std::sync::Arc;

/// Priority levels for work items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Work item with priority for the exploration queue.
#[derive(Debug, Clone)]
pub struct PrioritizedWorkItem {
    pub priority: WorkPriority,
    pub sequence: usize, // To maintain FIFO order for same priority
    pub item: ExplorationStep,
}

impl PartialEq for PrioritizedWorkItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PrioritizedWorkItem {}

impl PartialOrd for PrioritizedWorkItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedWorkItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier sequence
        match self.priority.cmp(&other.priority).reverse() {
            Ordering::Equal => self.sequence.cmp(&other.sequence),
            other => other,
        }
    }
}

/// A single step in the exploration of a Notion content tree.
#[derive(Debug, Clone)]
pub enum ExplorationStep {
    /// Identify an object's type and explore its contents
    IdentifyAndExplore {
        request: FetchRequest,
        context: FetchContext,
    },
    /// Retrieve child blocks for a parent object
    RetrieveChildren {
        parent_id: NotionId,
        context: FetchContext,
    },
    /// Follow references discovered within a block
    FollowReferences {
        block: Box<Block>,
        context: FetchContext,
    },
    /// Collect rows from a database
    CollectRows {
        database_id: NotionId,
        context: FetchContext,
    },
}

impl ExplorationStep {
    /// Determines the priority of this exploration step.
    pub fn priority(&self) -> WorkPriority {
        match self {
            ExplorationStep::IdentifyAndExplore { request, .. } => match &request.objective {
                FetchObjective::ResolveChildDatabase { .. } => WorkPriority::Critical,
                FetchObjective::ExploreRecursively { .. } => WorkPriority::Normal,
            },
            // Database queries are high priority
            ExplorationStep::CollectRows { .. } => WorkPriority::High,
            // Child block retrieval is normal priority
            ExplorationStep::RetrieveChildren { .. } => WorkPriority::Normal,
            // Reference following is low priority
            ExplorationStep::FollowReferences { .. } => WorkPriority::Low,
        }
    }
}

/// Result of processing an exploration step.
#[derive(Debug, Clone)]
pub enum StepOutcome {
    /// Step completed with discovered content
    Success(Box<CompletedStep>),
    /// Step was skipped (not an error)
    Skipped {
        reason: SkipReason,
        context: FetchContext,
    },
    /// Step failed
    Failed {
        reason: FailureReason,
        context: FetchContext,
    },
}

/// Why a fetch step was skipped.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SkipReason {
    AlreadyVisited(NotionId),
    DepthExhausted,
    ItemLimitReached,
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::AlreadyVisited(id) => write!(f, "ID {} already visited", id),
            SkipReason::DepthExhausted => write!(f, "maximum recursion depth reached"),
            SkipReason::ItemLimitReached => write!(f, "item limit reached"),
        }
    }
}

/// Why an exploration step failed.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FailureReason {
    Unreachable { cause: Arc<AppError> },
    ParseFailed { cause: Arc<AppError> },
    Unprocessable { cause: Arc<AppError> },
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::Unreachable { cause } => write!(f, "unreachable: {}", cause),
            FailureReason::ParseFailed { cause } => write!(f, "parse failed: {}", cause),
            FailureReason::Unprocessable { cause } => {
                write!(f, "unprocessable: {}", cause)
            }
        }
    }
}

/// A completed exploration step — the discovered content plus metadata.
#[derive(Debug, Clone)]
pub struct CompletedStep {
    pub content: DiscoveredContent,
    pub context: FetchContext,
    pub metadata: FetchMetadata,
    pub warnings: Vec<Warning>,
}

/// Content discovered during an exploration step.
#[derive(Debug, Clone)]
pub enum DiscoveredContent {
    /// A single object was discovered
    Object {
        object: NotionObject,
        #[allow(dead_code)]
        children_to_fetch: Vec<NotionId>,
        /// For databases fetched via child_database blocks, this is the block ID
        source_id: Option<NotionId>,
    },
    /// Child blocks were retrieved for a parent
    Blocks {
        parent_id: NotionId,
        blocks: Vec<Block>,
    },
    /// Database rows were collected
    Rows {
        database_id: NotionId,
        pages: Vec<Page>,
    },
}
