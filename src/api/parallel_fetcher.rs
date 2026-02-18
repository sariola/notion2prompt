// src/api/parallel_fetcher.rs
//! Parallel exploration of Notion content trees using work-stealing concurrency.
//!
//! Workers explore the content tree by executing exploration steps, each of which
//! may discover new content and produce further steps to follow.

use super::concurrent_queue::{ConcurrentWorkQueue, WorkerQueue};
use super::fetch_queue::{
    CompletedStep, DiscoveredContent, ExplorationStep, FailureReason, SkipReason, StepOutcome,
};
use super::object_graph::ObjectGraph;
use super::types::*;
use crate::config::PipelineConfig;
use crate::error::{classify_database_fetch_failure, AppError, DatabaseFetchFailure};
use crate::error_recovery::retry_with_backoff;
use crate::model::{
    Block, Database, DatabaseProperty, DatabasePropertyType, DatabaseTitle, NotionObject,
    NumberFormat, Page,
};
use crate::types::{DatabaseId, NotionId, PropertyName, Warning, WarningLevel};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

/// Enhanced queue-based fetcher with parallel work-stealing and error recovery.
pub struct NotionFetcher {
    client: Arc<dyn super::NotionRepository>,
    config: PipelineConfig,
    num_workers: usize,
}

impl NotionFetcher {
    /// Creates a new fetcher with the given client and config.
    ///
    /// Default concurrency is `max(num_cpus, 4)` capped at 24 workers.
    /// Since workers are async tasks waiting on network I/O (not CPU-bound),
    /// running more workers than CPU cores is both safe and beneficial.
    pub fn new(client: Arc<dyn super::NotionRepository>, config: &PipelineConfig) -> Self {
        let num_workers = config
            .concurrency
            .unwrap_or_else(|| num_cpus::get().clamp(4, 24));
        Self {
            client,
            config: config.clone(),
            num_workers: num_workers.clamp(1, 32),
        }
    }

    /// Creates a parallel fetcher with a specific number of workers.
    #[allow(dead_code)]
    pub fn with_workers(
        client: Arc<dyn super::NotionRepository>,
        config: &PipelineConfig,
        num_workers: usize,
    ) -> Self {
        Self {
            client,
            config: config.clone(),
            num_workers: num_workers.clamp(1, 32),
        }
    }

    /// Creates a sequential fetcher (single worker) for compatibility.
    #[allow(dead_code)]
    pub fn sequential(client: Arc<dyn super::NotionRepository>, config: &PipelineConfig) -> Self {
        Self::with_workers(client, config, 1)
    }

    /// Fetches a Notion object recursively using parallel work-stealing.
    pub async fn fetch_recursive(
        &self,
        id: &NotionId,
    ) -> Result<FetchResult<NotionObject>, AppError> {
        let (queue, workers) = ConcurrentWorkQueue::new(self.num_workers);
        let initial_context = FetchContext::with_options(
            self.config.depth,
            self.config.limit,
            self.config.always_fetch_databases,
        );

        log::info!(
            "Starting recursive fetch for {} (depth: {}, limit: {}, always_fetch_databases: {})",
            id.as_str(),
            self.config.depth,
            self.config.limit,
            self.config.always_fetch_databases
        );

        // Detect object type from the original URL to skip unnecessary API calls
        let type_hint = super::types::ObjectTypeHint::from_input(&self.config.raw_input);
        log::info!("Object type hint: {:?}", type_hint);

        // Enqueue initial work
        queue.enqueue(ExplorationStep::IdentifyAndExplore {
            request: FetchRequest {
                id: id.clone(),
                objective: FetchObjective::ExploreRecursively { type_hint },
            },
            context: initial_context.clone(),
        });

        // Spawn worker tasks
        let mut join_set = JoinSet::new();
        let queue_arc = Arc::new(queue);
        let stealers = queue_arc.stealers().to_vec();

        for worker in workers {
            let queue = Arc::clone(&queue_arc);
            let client = Arc::clone(&self.client);
            let config = self.config.clone();
            let stealers = stealers.clone();

            join_set.spawn(async move {
                let worker_fetcher = ExplorationWorker::new(&*client, &config);
                run_exploration_loop(worker, &worker_fetcher, &queue, &stealers).await
            });
        }

        // Wait for all workers to complete
        while let Some(result) = join_set.join_next().await {
            result.map_err(|e| AppError::InternalError {
                message: format!("Parallel fetch worker task failed with join error: {}. This may indicate a panic in the worker thread.", e),
                source: None,
            })??;
        }

        // Collect results and build the final object
        let results = Arc::try_unwrap(queue_arc)
            .unwrap_or_else(|_| unreachable!("All workers should be done"))
            .collect_results();

        self.assemble_results(results, id, initial_context)
    }

    /// Assembles work results into the final object tree.
    fn assemble_results(
        &self,
        results: Vec<StepOutcome>,
        root_id: &NotionId,
        initial_context: FetchContext,
    ) -> Result<FetchResult<NotionObject>, AppError> {
        let mut graph = ObjectGraph::with_capacity(results.len());
        let mut final_context = initial_context;
        let mut total_metadata = FetchMetadata::default();

        // Process all results
        for result in results {
            let (new_graph, new_context, new_metadata) = fold_into_graph(graph, result);
            graph = new_graph;
            final_context = new_context;
            total_metadata = total_metadata.merge(new_metadata);
        }

        log::debug!(
            "{} databases tracked, {} block-to-database mappings",
            graph.database_locations().len(),
            graph.child_db_block_to_database().len()
        );
        let root = graph
            .assemble(root_id)
            .map_err(|e| AppError::AssemblyFailed {
                root_id: root_id.as_str().to_string(),
                cause: format!(
                    "{}. This may indicate missing child objects or circular references.",
                    e
                ),
            })?;

        log::info!(
            "Fetch complete for {}: object tree assembled",
            root_id.as_str()
        );

        Ok(FetchResult {
            data: root,
            context: final_context,
            metadata: total_metadata,
        })
    }
}

/// Worker-specific fetcher that handles individual work items.
struct ExplorationWorker<'a> {
    client: &'a dyn super::NotionRepository,
    #[allow(dead_code)]
    config: &'a PipelineConfig,
}

impl<'a> ExplorationWorker<'a> {
    fn new(client: &'a dyn super::NotionRepository, config: &'a PipelineConfig) -> Self {
        Self { client, config }
    }

    /// Executes a single exploration step, returning the outcome and any follow-up steps.
    pub async fn execute_step(
        &self,
        item: ExplorationStep,
    ) -> Result<(StepOutcome, Vec<ExplorationStep>), AppError> {
        match item {
            ExplorationStep::IdentifyAndExplore { request, context } => {
                self.identify_and_explore(request, context).await
            }
            ExplorationStep::RetrieveChildren { parent_id, context } => {
                self.retrieve_children(parent_id, context).await
            }
            ExplorationStep::FollowReferences { block, context } => {
                self.follow_references(block, context).await
            }
            ExplorationStep::CollectRows {
                database_id,
                context,
            } => self.collect_rows(database_id, context).await,
        }
    }

    /// Identifies an object's type and explores its contents.
    async fn identify_and_explore(
        &self,
        request: FetchRequest,
        context: FetchContext,
    ) -> Result<(StepOutcome, Vec<ExplorationStep>), AppError> {
        // Check if we should fetch this ID
        if !context.should_fetch(&request.id) {
            return Ok((
                StepOutcome::Skipped {
                    reason: SkipReason::AlreadyVisited(request.id.clone()),
                    context,
                },
                vec![],
            ));
        }

        // Mark as visited
        let context = context.with_visited(request.id.clone());

        // Fetch the object with retry — use targeted resolution for child databases
        let obj = retry_with_backoff(
            || self.resolve_by_objective(&request.id, &request.objective),
            3,
            Duration::from_millis(100),
            Duration::from_secs(5),
        )
        .await?;

        let metadata = FetchMetadata {
            items_fetched: 1,
            ..Default::default()
        };

        // Determine what additional work needs to be done based on the objective
        let mut more_work = Vec::new();

        // Extract source_id for the object graph (only set for child database fetches)
        let source_id = match &request.objective {
            FetchObjective::ResolveChildDatabase { source_block_id } => {
                Some(source_block_id.clone())
            }
            FetchObjective::ExploreRecursively { .. } => None,
        };

        match (&request.objective, &obj) {
            (FetchObjective::ExploreRecursively { .. }, NotionObject::Page(page)) => {
                log::debug!("Fetched page '{}' ({})", page.title(), page.id.as_str());
                if context.depth_remaining > 0 {
                    log::debug!(
                        "Queueing RetrieveChildren for page {} (depth_remaining: {})",
                        page.id.as_str(),
                        context.depth_remaining
                    );
                    more_work.push(ExplorationStep::RetrieveChildren {
                        parent_id: page.id.clone().into(),
                        context: context.clone(),
                    });
                }
            }
            (FetchObjective::ExploreRecursively { .. }, NotionObject::Database(db)) => {
                log::debug!(
                    "Fetched database '{}' ({}) - {} properties (exploring recursively)",
                    db.title(),
                    db.id.as_str(),
                    db.properties.len(),
                );
                if context.depth_remaining > 0 {
                    log::debug!(
                        "Queueing CollectRows for '{}' ({})",
                        db.title(),
                        db.id.as_str()
                    );
                    more_work.push(ExplorationStep::CollectRows {
                        database_id: db.id.clone().into(),
                        context: context.clone(),
                    });
                }
            }
            (FetchObjective::ResolveChildDatabase { .. }, NotionObject::Database(db)) => {
                log::debug!(
                    "Fetched child database '{}' ({}) - {} properties, {} pages",
                    db.title(),
                    db.id.as_str(),
                    db.properties.len(),
                    db.pages.len(),
                );
                // Skip CollectRows if the database already has pages
                // (linked database fallback pre-populates pages via query_rows)
                if !db.pages.is_empty() {
                    log::debug!(
                        "Child database '{}' already has {} pages (from linked db query fallback)",
                        db.title(),
                        db.pages.len(),
                    );
                } else if context.depth_remaining > 0 {
                    log::debug!(
                        "Queueing CollectRows for child database '{}' ({})",
                        db.title(),
                        db.id.as_str()
                    );
                    more_work.push(ExplorationStep::CollectRows {
                        database_id: db.id.clone().into(),
                        context: context.clone(),
                    });
                } else {
                    log::debug!(
                        "Skipping database rows for '{}' (depth_remaining: {})",
                        db.title(),
                        context.depth_remaining,
                    );
                }
            }
            (_, NotionObject::Block(block)) => {
                log::debug!(
                    "Fetched block {} ({})",
                    block.block_type(),
                    block.id().as_str()
                );
                if context.depth_remaining > 0 && block.has_children() {
                    more_work.push(ExplorationStep::FollowReferences {
                        block: Box::new(block.clone()),
                        context: context.clone(),
                    });
                }
            }
            _ => {}
        }

        // Extract child IDs to fetch - empty for now as children are handled separately
        let children_to_fetch: Vec<NotionId> = vec![];

        // Return the object and any additional work
        Ok((
            StepOutcome::Success(Box::new(CompletedStep {
                content: DiscoveredContent::Object {
                    object: obj,
                    children_to_fetch,
                    source_id,
                },
                context,
                metadata,
                warnings: vec![],
            })),
            more_work,
        ))
    }

    /// Retrieves child blocks for a parent, detects child databases, and plans follow-up work.
    async fn retrieve_children(
        &self,
        parent_id: NotionId,
        context: FetchContext,
    ) -> Result<(StepOutcome, Vec<ExplorationStep>), AppError> {
        log::debug!(
            "Processing blocks for parent {} (depth_remaining: {})",
            parent_id.as_str(),
            context.depth_remaining
        );

        // Step 1: Retrieve raw blocks from the API
        let blocks = match self.client.retrieve_children(&parent_id).await {
            Ok(blocks) => {
                log::debug!(
                    "Fetched {} blocks for parent {}",
                    blocks.len(),
                    parent_id.as_str()
                );
                blocks
            }
            Err(e) => {
                log::warn!("Failed to fetch blocks for {}: {}", parent_id.as_str(), e);
                return Ok((
                    StepOutcome::Failed {
                        reason: FailureReason::Unreachable { cause: Arc::new(e) },
                        context,
                    },
                    vec![],
                ));
            }
        };

        let metadata = FetchMetadata {
            items_fetched: blocks.len() as u32,
            ..Default::default()
        };

        // Step 2: Plan follow-up work for child databases and enrichable blocks
        let more_work = plan_deeper_exploration(&blocks, &parent_id, &context);

        Ok((
            StepOutcome::Success(Box::new(CompletedStep {
                content: DiscoveredContent::Blocks { parent_id, blocks },
                context,
                metadata,
                warnings: vec![],
            })),
            more_work,
        ))
    }

    /// Follows references discovered within a block.
    async fn follow_references(
        &self,
        block: Box<Block>,
        context: FetchContext,
    ) -> Result<(StepOutcome, Vec<ExplorationStep>), AppError> {
        let mut metadata = FetchMetadata::default();
        let mut more_work = Vec::new();

        // Extract links and queue them for fetching
        let links = extract_links_from_block(&block);
        if !links.is_empty() {
            metadata.links_found.extend(links.clone());
            more_work.extend(exploration_steps_for_references(&links, &context));
        }

        // Fetch children if needed
        if block.has_children() && context.depth_remaining > 0 {
            more_work.push(ExplorationStep::RetrieveChildren {
                parent_id: block.id().clone().into(),
                context: context.clone().with_decremented_depth(),
            });
        }

        // Return success without re-adding the block to the graph
        // The block has already been added when its parent was processed
        Ok((
            StepOutcome::Success(Box::new(CompletedStep {
                content: DiscoveredContent::Object {
                    object: NotionObject::Block(*block),
                    children_to_fetch: vec![],
                    source_id: None,
                },
                context,
                metadata,
                warnings: vec![],
            })),
            more_work,
        ))
    }

    /// Collects rows from a database.
    async fn collect_rows(
        &self,
        database_id: NotionId,
        context: FetchContext,
    ) -> Result<(StepOutcome, Vec<ExplorationStep>), AppError> {
        log::debug!(
            "Querying database rows for {} (depth_remaining: {}, items_remaining: {})",
            database_id.as_str(),
            context.depth_remaining,
            context.items_remaining
        );

        let rows = match self.client.query_rows(&database_id).await {
            Ok(rows) => {
                log::debug!(
                    "Queried database {} - {} rows",
                    database_id.as_str(),
                    rows.len()
                );
                rows
            }
            Err(e) => {
                log::warn!("Failed to query database {}: {}", database_id.as_str(), e);
                return Ok((
                    StepOutcome::Failed {
                        reason: FailureReason::Unreachable { cause: Arc::new(e) },
                        context,
                    },
                    vec![],
                ));
            }
        };

        let updated_context = context.with_items_used(rows.len() as u32);
        let metadata = FetchMetadata {
            items_fetched: rows.len() as u32,
            ..Default::default()
        };

        Ok((
            StepOutcome::Success(Box::new(CompletedStep {
                content: DiscoveredContent::Rows {
                    database_id,
                    pages: rows,
                },
                context: updated_context,
                metadata,
                warnings: vec![],
            })),
            vec![],
        ))
    }

    /// Resolves an object using the appropriate strategy for the fetch objective.
    ///
    /// For child databases, tries database first (skip the wasted page attempt).
    /// When the database fetch fails, classifies the failure reason and enriches
    /// the fallback block with that classification (e.g., LinkedDatabase vs Inaccessible).
    /// For general exploration, uses the default page → database → block order.
    async fn resolve_by_objective(
        &self,
        id: &NotionId,
        objective: &FetchObjective,
    ) -> Result<NotionObject, AppError> {
        match objective {
            FetchObjective::ResolveChildDatabase { .. } => {
                // Try database first — this is what we expect for child databases
                match self.client.retrieve_database(id).await {
                    Ok(db) => Ok(NotionObject::Database(db)),
                    Err(e) => {
                        let failure = classify_database_fetch_failure(&e);
                        match &failure {
                            DatabaseFetchFailure::LinkedDatabase => {
                                log::info!(
                                    "Child database {} is a linked database — \
                                     attempting query_rows() fallback.",
                                    id.as_str()
                                );

                                // Try querying rows — the query endpoint works for linked databases
                                match self.client.query_rows(id).await {
                                    Ok(rows) => {
                                        log::info!(
                                            "Successfully queried {} rows from linked database {}",
                                            rows.len(),
                                            id.as_str()
                                        );
                                        let schema = infer_schema_from_pages(&rows);
                                        let db = Database {
                                            id: DatabaseId::parse(id.as_str())
                                                .unwrap_or_else(|_| DatabaseId::new_v4()),
                                            title: DatabaseTitle::new(vec![]),
                                            url: String::new(),
                                            pages: rows,
                                            properties: schema,
                                            parent: None,
                                            archived: false,
                                        };
                                        return Ok(NotionObject::Database(db));
                                    }
                                    Err(query_err) => {
                                        // The error often reveals the source database ID.
                                        if let Some(source_id) =
                                            extract_source_database_id(&query_err)
                                        {
                                            // When query_rows(block_id) returns object_not_found
                                            // with the source database ID, the API already resolved
                                            // the linked DB and tried to access the source — retrying
                                            // with that same ID won't help.
                                            if is_not_found_error(&query_err) {
                                                log::info!(
                                                    "Linked database {} references source database {} \
                                                     which is not shared with the integration",
                                                    id.as_str(),
                                                    source_id.as_str()
                                                );
                                            } else {
                                                // Transient or other error — worth retrying with
                                                // the source ID directly.
                                                log::info!(
                                                    "Extracted source database ID {} from error, retrying query_rows()",
                                                    source_id.as_str()
                                                );
                                                match self.client.query_rows(&source_id).await {
                                                    Ok(rows) => {
                                                        log::info!(
                                                            "Successfully queried {} rows from source database {}",
                                                            rows.len(),
                                                            source_id.as_str()
                                                        );
                                                        let schema = infer_schema_from_pages(&rows);
                                                        let db = Database {
                                                            id: DatabaseId::parse(id.as_str())
                                                                .unwrap_or_else(|_| {
                                                                    DatabaseId::new_v4()
                                                                }),
                                                            title: DatabaseTitle::new(vec![]),
                                                            url: String::new(),
                                                            pages: rows,
                                                            properties: schema,
                                                            parent: None,
                                                            archived: false,
                                                        };
                                                        return Ok(NotionObject::Database(db));
                                                    }
                                                    Err(retry_err) => {
                                                        log::warn!(
                                                            "Retry with source database {} also failed: {}",
                                                            source_id.as_str(),
                                                            retry_err
                                                        );
                                                    }
                                                }
                                            }
                                        } else {
                                            log::warn!(
                                                "query_rows() failed for linked database {}: {}",
                                                id.as_str(),
                                                query_err
                                            );
                                        }
                                        // Fall through to the existing block fallback
                                    }
                                }
                            }
                            other => {
                                log::warn!(
                                    "Child database fetch failed for {}: {}",
                                    id.as_str(),
                                    other
                                );
                            }
                        }

                        // Fall back to block, enriched with the failure classification
                        match self.client.retrieve_block(id).await {
                            Ok(block) => Ok(enrich_with_failure_reason(block, failure)),
                            Err(e) => Err(AppError::InvalidId(format!(
                                "Could not resolve child database {}: {}",
                                id.as_str(),
                                e
                            ))),
                        }
                    }
                }
            }
            FetchObjective::ExploreRecursively { ref type_hint } => {
                use super::types::ObjectTypeHint;
                match type_hint {
                    ObjectTypeHint::Database => {
                        // URL hints this is a database — try database first
                        log::debug!(
                            "Speculative typing: trying database first for {}",
                            id.as_str()
                        );
                        if let Ok(db) = self.client.retrieve_database(id).await {
                            return Ok(NotionObject::Database(db));
                        }
                        // Fall back to default resolution order
                        self.client.resolve_object(id).await
                    }
                    ObjectTypeHint::Unknown => self.client.resolve_object(id).await,
                }
            }
        }
    }
}

/// Enriches a child database block with the classified reason its database couldn't be fetched.
///
/// This is a pure function: given a block and a failure classification, it returns
/// a NotionObject with the appropriate ChildDatabaseContent variant stamped in.
fn enrich_with_failure_reason(mut block: Block, failure: DatabaseFetchFailure) -> NotionObject {
    use crate::model::blocks::ChildDatabaseContent;

    if let Block::ChildDatabase(ref mut cdb) = block {
        cdb.content = match failure {
            DatabaseFetchFailure::LinkedDatabase => ChildDatabaseContent::LinkedDatabase,
            DatabaseFetchFailure::PermissionDenied { reason } => {
                ChildDatabaseContent::Inaccessible { reason }
            }
            DatabaseFetchFailure::NotFound => ChildDatabaseContent::Inaccessible {
                reason: "database not found".to_string(),
            },
            DatabaseFetchFailure::Other { cause } => {
                ChildDatabaseContent::Inaccessible { reason: cause }
            }
        };
    }
    NotionObject::Block(block)
}

/// Runs the exploration loop for a single worker.
async fn run_exploration_loop(
    worker_queue: WorkerQueue,
    fetcher: &ExplorationWorker<'_>,
    global_queue: &ConcurrentWorkQueue,
    stealers: &[crossbeam::deque::Stealer<super::fetch_queue::PrioritizedWorkItem>],
) -> Result<(), AppError> {
    let mut consecutive_empty_attempts = 0;
    const MAX_EMPTY_ATTEMPTS: u32 = 10;

    loop {
        // Try to get work
        let work_item = match worker_queue.dequeue(stealers) {
            Some(item) => {
                consecutive_empty_attempts = 0;
                item
            }
            None => {
                // Check if there's still pending work globally
                if !global_queue.has_pending_work() {
                    log::debug!("No pending work, worker exiting");
                    break;
                }

                consecutive_empty_attempts += 1;

                // Brief sleep to avoid busy waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                // After several attempts, check one more time
                if consecutive_empty_attempts >= MAX_EMPTY_ATTEMPTS {
                    if !global_queue.has_pending_work() {
                        log::debug!(
                            "No work after {} attempts, worker exiting",
                            MAX_EMPTY_ATTEMPTS
                        );
                        break;
                    }
                    consecutive_empty_attempts = 0;
                }

                continue;
            }
        };

        log::debug!("Processing work item: {:?}", work_item.priority());

        // Process the work item
        match fetcher.execute_step(work_item).await {
            Ok((result, more_work)) => {
                // Queue additional work BEFORE marking this item complete
                if !more_work.is_empty() {
                    log::debug!("Queueing {} additional work items", more_work.len());
                    global_queue.enqueue_multiple(more_work);
                }

                match &result {
                    StepOutcome::Success(_) => {}
                    StepOutcome::Skipped { reason, .. } => {
                        log::debug!("Work item skipped: {}", reason);
                    }
                    StepOutcome::Failed { reason, .. } => {
                        log::warn!("Work item failed: {}", reason);
                    }
                }

                // Store the result
                global_queue.store_result(result);

                // Mark this work item as completed
                global_queue.mark_completed();
            }
            Err(e) => {
                log::warn!("Error processing work item: {}", e);
                global_queue.store_result(StepOutcome::Failed {
                    reason: FailureReason::Unprocessable { cause: Arc::new(e) },
                    context: FetchContext::new(0, 0),
                });

                // Mark as completed even on failure
                global_queue.mark_completed();
            }
        }
    }

    Ok(())
}

/// Folds a step outcome into the growing object graph.
fn fold_into_graph(
    graph: ObjectGraph,
    result: StepOutcome,
) -> (ObjectGraph, FetchContext, FetchMetadata) {
    match result {
        StepOutcome::Success(success) => {
            let mut metadata = success.metadata;
            metadata.warnings.extend(success.warnings);
            (
                register_content(graph, success.content),
                success.context,
                metadata,
            )
        }
        StepOutcome::Skipped { reason, context } => {
            log::debug!("Work item skipped: {}", reason);
            (graph, context, FetchMetadata::default())
        }
        StepOutcome::Failed { reason, context } => {
            log::warn!("Work item failed: {}", reason);
            let mut metadata = FetchMetadata::default();
            metadata.warnings.push(Warning {
                level: WarningLevel::Warning,
                message: reason.to_string(),
                context: None,
            });
            (graph, context, metadata)
        }
    }
}

/// Registers discovered content into the object graph.
fn register_content(graph: ObjectGraph, content: DiscoveredContent) -> ObjectGraph {
    match content {
        DiscoveredContent::Object {
            object, source_id, ..
        } => {
            log::debug!("Adding object to graph: {:?}", object.id());
            graph.with_object_from_source(object, source_id)
        }
        DiscoveredContent::Blocks { parent_id, blocks } => {
            log::debug!(
                "Adding {} blocks to parent {}",
                blocks.len(),
                parent_id.as_str()
            );
            graph.with_blocks(parent_id, blocks)
        }
        DiscoveredContent::Rows { database_id, pages } => {
            log::debug!(
                "Adding {} rows to database {}",
                pages.len(),
                database_id.as_str()
            );
            graph.with_rows(database_id, pages)
        }
    }
}

// --- Error classification helpers for linked database retry ---

/// Checks whether an error is an `object_not_found` response from the Notion API.
fn is_not_found_error(error: &AppError) -> bool {
    match error {
        AppError::NotionClient(crate::error::NotionClientError::NotionApi { code, .. }) => {
            code == "object_not_found"
        }
        AppError::NotionService { code, .. } => code.is_not_found(),
        _ => false,
    }
}

/// Extracts a source database ID from a Notion API error message.
///
/// When querying a linked database, the Notion API often returns an error
/// containing the actual source database ID, e.g.:
///   "Could not find database with ID: 8e2801e8-1705-4f25-ae28-7572a069c873"
///
/// This function extracts that ID so we can retry the query against the source.
fn extract_source_database_id(error: &AppError) -> Option<NotionId> {
    let message = match error {
        AppError::NotionClient(crate::error::NotionClientError::NotionApi { message, .. }) => {
            message
        }
        AppError::NotionService { message, .. } => message,
        _ => return None,
    };

    // Pattern: "Could not find database with ID: <uuid>"
    let prefix = "Could not find database with ID: ";
    let start = message.find(prefix)? + prefix.len();
    let id_str = message[start..].split('.').next()?.trim();
    NotionId::parse(id_str).ok()
}

// --- Schema inference for linked databases ---

/// Infers database schema (property definitions) from queried page properties.
///
/// When `retrieve_database()` fails for linked databases, we can still query
/// the rows. This function reconstructs the schema by examining the property
/// types present in the returned pages.
fn infer_schema_from_pages(pages: &[Page]) -> HashMap<PropertyName, DatabaseProperty> {
    let mut schema: HashMap<PropertyName, DatabaseProperty> = HashMap::new();
    for page in pages {
        for (name, value) in &page.properties {
            if schema.contains_key(name) {
                continue;
            }
            if let Some(db_prop_type) = property_type_value_to_db_type(&value.type_specific_value) {
                schema.insert(
                    name.clone(),
                    DatabaseProperty {
                        id: name.clone(),
                        name: name.clone(),
                        property_type: db_prop_type,
                    },
                );
            }
        }
    }
    schema
}

/// Maps a `PropertyTypeValue` variant to its corresponding `DatabasePropertyType`.
fn property_type_value_to_db_type(
    value: &crate::model::PropertyTypeValue,
) -> Option<DatabasePropertyType> {
    use crate::model::PropertyTypeValue;
    use DatabasePropertyType as DPT;

    Some(match value {
        PropertyTypeValue::Title { .. } => DPT::Title,
        PropertyTypeValue::RichText { .. } => DPT::RichText,
        PropertyTypeValue::Number { .. } => DPT::Number {
            format: NumberFormat::Number,
        },
        PropertyTypeValue::Select { .. } => DPT::Select { options: vec![] },
        PropertyTypeValue::MultiSelect { .. } => DPT::MultiSelect { options: vec![] },
        PropertyTypeValue::Status { .. } => DPT::Status { options: vec![] },
        PropertyTypeValue::Date { .. } => DPT::Date,
        PropertyTypeValue::People { .. } => DPT::People,
        PropertyTypeValue::Files { .. } => DPT::Files,
        PropertyTypeValue::Checkbox { .. } => DPT::Checkbox,
        PropertyTypeValue::Url { .. } => DPT::Url,
        PropertyTypeValue::Email { .. } => DPT::Email,
        PropertyTypeValue::PhoneNumber { .. } => DPT::PhoneNumber,
        PropertyTypeValue::Formula { .. } => DPT::Formula {
            expression: String::new(),
        },
        PropertyTypeValue::Relation { .. } => DPT::Relation {
            database_id: String::new(),
            synced_property_name: None,
            synced_property_id: None,
        },
        PropertyTypeValue::Rollup { .. } => DPT::Rollup {
            relation_property_name: String::new(),
            relation_property_id: String::new(),
            rollup_property_name: String::new(),
            rollup_property_id: String::new(),
            function: String::new(),
        },
        PropertyTypeValue::CreatedTime { .. } => DPT::CreatedTime,
        PropertyTypeValue::CreatedBy { .. } => DPT::CreatedBy,
        PropertyTypeValue::LastEditedTime { .. } => DPT::LastEditedTime,
        PropertyTypeValue::LastEditedBy { .. } => DPT::LastEditedBy,
        // UniqueID and Verification don't have DatabasePropertyType equivalents
        _ => return None,
    })
}

// --- Pure work generation functions ---

/// Plans deeper exploration for a set of retrieved blocks.
///
/// For each block, decides whether to queue a database fetch (for child databases)
/// or a reference-following pass (for blocks with children or links).
fn plan_deeper_exploration(
    blocks: &[Block],
    parent_id: &NotionId,
    context: &FetchContext,
) -> Vec<ExplorationStep> {
    let mut work = Vec::new();

    for block in blocks {
        match block {
            Block::ChildDatabase(child_db) => {
                log::debug!(
                    "Child database detected: '{}' ({}, parent: {})",
                    child_db.title,
                    child_db.common.id.as_str(),
                    parent_id.as_str()
                );

                if context.always_fetch_databases || context.depth_remaining > 0 {
                    let db_id: NotionId = child_db.common.id.clone().into();
                    let block_id: NotionId = child_db.common.id.clone().into();

                    log::debug!(
                        "Queueing database fetch for '{}' ({})",
                        child_db.title,
                        db_id.as_str()
                    );

                    work.push(ExplorationStep::IdentifyAndExplore {
                        request: FetchRequest {
                            id: db_id,
                            objective: FetchObjective::ResolveChildDatabase {
                                source_block_id: block_id,
                            },
                        },
                        context: context.clone().with_decremented_depth(),
                    });
                } else {
                    log::warn!(
                        "Skipping database fetch for '{}' ({}) - depth exhausted",
                        child_db.title,
                        child_db.common.id.as_str()
                    );
                }
            }
            _ => {
                if context.depth_remaining > 0 && (block.has_children() || has_links(block)) {
                    log::debug!(
                        "Queueing FollowReferences for {} ({})",
                        block.block_type(),
                        block.id().as_str()
                    );
                    work.push(ExplorationStep::FollowReferences {
                        block: Box::new(block.clone()),
                        context: context.clone().with_decremented_depth(),
                    });
                }
            }
        }
    }

    work
}

/// Produces exploration steps for discovered references.
fn exploration_steps_for_references(
    links: &[DiscoveredLink],
    context: &FetchContext,
) -> Vec<ExplorationStep> {
    if context.depth_remaining == 0 {
        return vec![];
    }

    links
        .iter()
        .filter(|link| context.should_fetch(&link.id))
        .map(|link| ExplorationStep::IdentifyAndExplore {
            request: FetchRequest {
                id: link.id.clone(),
                objective: FetchObjective::ExploreRecursively {
                    type_hint: super::types::ObjectTypeHint::Unknown,
                },
            },
            context: context.clone().with_decremented_depth(),
        })
        .collect()
}

// Helper functions

/// Checks if a block has links.
fn has_links(block: &Block) -> bool {
    // TODO: Check rich text for links in each block variant

    // Check specific block types
    matches!(
        block,
        Block::LinkToPage { .. } | Block::ChildDatabase { .. }
    )
}

/// Extracts links from a block's content.
fn extract_links_from_block(block: &Block) -> Vec<DiscoveredLink> {
    let mut links = Vec::new();

    // TODO: Extract from rich text content in each block variant

    // Extract from specific block types
    match block {
        Block::LinkToPage(block) => {
            links.push(DiscoveredLink {
                id: block.page_id.clone().into(),
                link_type: LinkType::Page,
                origin: LinkOrigin::LinkToPageBlock,
            });
        }
        Block::ChildDatabase(_) => {
            // Child databases are handled separately in process_blocks
            // to avoid duplicate fetching
        }
        _ => {}
    }

    links
}
