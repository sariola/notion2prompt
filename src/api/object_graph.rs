// src/api/object_graph.rs
//! Immutable graph structure for tracking Notion object relationships.
//!
//! The assembly story reads as three named steps:
//!   1. Register objects as they arrive from the API
//!   2. Assemble the tree by walking parent→child edges
//!   3. Embed databases into their ChildDatabaseBlock hosts

use crate::model::{Block, Database, NotionObject, Page};
use crate::types::NotionId;
use std::collections::HashMap;

/// Immutable graph representing parent-child relationships between Notion objects.
#[derive(Debug, Clone)]
pub struct ObjectGraph {
    /// Objects indexed by their ID
    objects: HashMap<NotionId, NotionObject>,
    /// Child relationships: parent_id -> vec of child_ids
    children: HashMap<NotionId, Vec<NotionId>>,
    /// Parent relationships: child_id -> parent_id
    parents: HashMap<NotionId, NotionId>,
    /// Database locations: database_id -> (parent_type, parent_id)
    database_locations: HashMap<NotionId, DatabaseLocation>,
    /// Maps child database block IDs to actual database IDs
    child_db_block_to_database: HashMap<NotionId, NotionId>,
}

/// Tracks where a database was found in the object tree
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseLocation {
    /// Type of parent that contains this database
    pub parent_type: DatabaseParentType,
    /// ID of the parent object
    pub parent_id: NotionId,
    /// Path from root to this database (for debugging)
    pub path: Vec<NotionId>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DatabaseParentType {
    /// Database is a direct child of a page
    PageChild,
    /// Database is referenced by a ChildDatabaseBlock in a page
    ChildDatabaseBlock,
    /// Database is embedded within another block
    BlockChild,
}

impl Default for ObjectGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectGraph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self::with_capacity(128)
    }

    /// Creates a new graph with capacity hints.
    pub fn with_capacity(expected_objects: usize) -> Self {
        Self {
            objects: HashMap::with_capacity(expected_objects),
            children: HashMap::with_capacity(expected_objects / 2),
            parents: HashMap::with_capacity(expected_objects),
            database_locations: HashMap::with_capacity(expected_objects / 10),
            child_db_block_to_database: HashMap::with_capacity(expected_objects / 20),
        }
    }

    // --- Registration: objects arrive from the API ---

    /// Adds an object to the graph.
    pub fn with_object(self, object: NotionObject) -> Self {
        self.with_object_from_source(object, None)
    }

    /// Adds an object to the graph with optional source ID (for child database blocks).
    pub fn with_object_from_source(
        self,
        object: NotionObject,
        source_id: Option<NotionId>,
    ) -> Self {
        let id = object.id();
        log::debug!(
            "Registering {} '{}'",
            object.object_type_name(),
            id.as_str()
        );
        self.register_database_origin(&object, &id, &source_id)
            .store_object(id, object)
    }

    /// Records where a database came from — either via a child block or directly.
    fn register_database_origin(
        self,
        object: &NotionObject,
        id: &NotionId,
        source_id: &Option<NotionId>,
    ) -> Self {
        let db = match object {
            NotionObject::Database(db) => db,
            _ => return self,
        };

        let mut graph = self;

        // If fetched via a child database block, record that mapping
        if let Some(block_id) = source_id {
            log::debug!(
                "Database '{}' fetched via child block {}",
                db.title().as_plain_text(),
                block_id.as_str()
            );
            graph
                .child_db_block_to_database
                .insert(block_id.clone(), id.clone());
        }

        // Wire up parent-child relationship if already tracked from a ChildDatabaseBlock
        let needs_relationship = graph
            .database_locations
            .get(id)
            .is_some_and(|loc| loc.parent_type == DatabaseParentType::ChildDatabaseBlock);

        if needs_relationship {
            if let Some(location) = graph.database_locations.get(id) {
                let parent_id = location.parent_id.clone();
                graph = graph.with_relationship(parent_id, id.clone());
            }
        } else if !graph.database_locations.contains_key(id) {
            graph.database_locations.insert(
                id.clone(),
                DatabaseLocation {
                    parent_type: DatabaseParentType::PageChild,
                    parent_id: id.clone(),
                    path: vec![],
                },
            );
        }

        graph
    }

    /// Stores the object in the graph by ID.
    fn store_object(mut self, id: NotionId, object: NotionObject) -> Self {
        self.objects.insert(id, object);
        self
    }

    /// Adds a parent-child relationship.
    pub fn with_relationship(self, parent_id: NotionId, child_id: NotionId) -> Self {
        let mut children = self.children;
        let mut parents = self.parents;

        children
            .entry(parent_id.clone())
            .or_default()
            .push(child_id.clone());
        parents.insert(child_id, parent_id);

        Self {
            objects: self.objects,
            children,
            parents,
            database_locations: self.database_locations,
            child_db_block_to_database: self.child_db_block_to_database,
        }
    }

    /// Adds blocks as children of a parent, tracking any ChildDatabaseBlocks found.
    pub fn with_blocks(self, parent_id: NotionId, blocks: Vec<Block>) -> Self {
        log::debug!(
            "Adding {} blocks to parent {}",
            blocks.len(),
            parent_id.as_str()
        );

        let mut graph = self;

        for block in blocks.into_iter() {
            if let Block::ChildDatabase(ref child_db) = &block {
                let db_id: NotionId = child_db.common.id.clone().into();
                graph.database_locations.insert(
                    db_id.clone(),
                    DatabaseLocation {
                        parent_type: DatabaseParentType::ChildDatabaseBlock,
                        parent_id: parent_id.clone(),
                        path: vec![parent_id.clone()],
                    },
                );
            }

            let child_id: NotionId = block.id().clone().into();
            graph = graph
                .with_object(NotionObject::Block(block))
                .with_relationship(parent_id.clone(), child_id);
        }

        graph
    }

    /// Adds database rows as children.
    pub fn with_rows(self, database_id: NotionId, pages: Vec<Page>) -> Self {
        pages.into_iter().fold(self, |graph, row| {
            let child_id: NotionId = row.id.clone().into();
            graph
                .with_object(NotionObject::Page(row))
                .with_relationship(database_id.clone(), child_id)
        })
    }

    // --- Assembly: walk edges to build the tree ---

    /// Assembles the complete object tree starting from a root ID.
    pub fn assemble(&self, root_id: &NotionId) -> Result<NotionObject, String> {
        self.assemble_recursive(root_id, &mut Vec::new())
    }

    /// Gets database location information.
    #[allow(dead_code)]
    pub fn get_database_location(&self, database_id: &NotionId) -> Option<&DatabaseLocation> {
        self.database_locations.get(database_id)
    }

    /// Gets all tracked database locations.
    pub fn database_locations(&self) -> &HashMap<NotionId, DatabaseLocation> {
        &self.database_locations
    }

    /// Gets all child database block to database mappings.
    pub fn child_db_block_to_database(&self) -> &HashMap<NotionId, NotionId> {
        &self.child_db_block_to_database
    }

    /// Recursively assembles objects with cycle detection.
    fn assemble_recursive(
        &self,
        id: &NotionId,
        stack: &mut Vec<NotionId>,
    ) -> Result<NotionObject, String> {
        if stack.contains(id) {
            return Err(format!("Cycle detected at ID: {}", id.as_str()));
        }

        let object = self
            .objects
            .get(id)
            .ok_or_else(|| format!("Object not found: {}", id.as_str()))?;

        stack.push(id.clone());

        let result = if let Some(child_ids) = self.children.get(id) {
            let children: Vec<NotionObject> = child_ids
                .iter()
                .map(|child_id| self.assemble_recursive(child_id, stack))
                .collect::<Result<Vec<_>, _>>()?;

            self.assemble_with_children(object.clone(), children)
        } else {
            Ok(object.clone())
        };

        stack.pop();

        result
    }

    // --- Nesting: embed databases into their ChildDatabaseBlock hosts ---

    /// Assembles a parent with its children, embedding databases into blocks.
    fn assemble_with_children(
        &self,
        mut parent: NotionObject,
        children: Vec<NotionObject>,
    ) -> Result<NotionObject, String> {
        log::debug!(
            "Assembling {} children for {} '{}'",
            children.len(),
            parent.object_type_name(),
            parent.id()
        );

        match &mut parent {
            NotionObject::Page(page) => {
                self.attach_children_to_page(page, children);
            }
            NotionObject::Database(db) => {
                let rows = extract_pages(children);
                if !rows.is_empty() {
                    db.pages = rows;
                }
            }
            NotionObject::Block(block) => {
                self.attach_children_to_block(block, children);
            }
        }
        Ok(parent)
    }

    /// Partitions children by kind, then embeds databases into their host blocks.
    fn attach_children_to_page(&self, page: &mut Page, children: Vec<NotionObject>) {
        let (blocks, mut databases) = self.partition_by_kind(children);

        log::debug!(
            "Separated into {} blocks and {} databases for page '{}'",
            blocks.len(),
            databases.len(),
            page.title().as_str()
        );

        page.blocks = embed_databases(blocks, &mut databases, &self.child_db_block_to_database);
    }

    /// Partitions children by kind, then embeds databases into their host blocks.
    fn attach_children_to_block(&self, block: &mut Block, children: Vec<NotionObject>) {
        let (child_blocks, mut databases) = self.partition_by_kind(children);

        if !databases.is_empty() || !child_blocks.is_empty() {
            let enriched = embed_databases(
                child_blocks,
                &mut databases,
                &self.child_db_block_to_database,
            );
            if !enriched.is_empty() {
                *block.children_mut() = enriched;
            }
        }
    }

    /// Partitions a mixed list of children into (blocks, embeddable databases).
    fn partition_by_kind(
        &self,
        children: Vec<NotionObject>,
    ) -> (Vec<Block>, HashMap<NotionId, Database>) {
        let mut blocks = Vec::new();
        let mut databases = HashMap::new();

        for child in children {
            match child {
                NotionObject::Block(block) => blocks.push(block),
                NotionObject::Database(database) => {
                    let db_id: NotionId = database.id.clone().into();
                    let is_embeddable = self
                        .child_db_block_to_database
                        .values()
                        .any(|mapped_id| mapped_id == &db_id);

                    if is_embeddable {
                        // Ensure a ChildDatabaseBlock exists in the blocks list
                        let block_exists = blocks.iter().any(|b| {
                            if let Block::ChildDatabase(cdb) = b {
                                cdb.common.id.as_str() == db_id.as_str()
                            } else {
                                false
                            }
                        });

                        if !block_exists {
                            blocks.push(recreate_child_database_block(
                                &database,
                                &db_id,
                                &self.child_db_block_to_database,
                            ));
                        }

                        databases.insert(db_id, database);
                    } else {
                        log::warn!(
                            "Direct database child '{}' not embeddable — no matching ChildDatabaseBlock",
                            database.title().as_plain_text()
                        );
                    }
                }
                _ => {}
            }
        }

        (blocks, databases)
    }
}

/// Extracts pages from a list of NotionObjects.
fn extract_pages(objects: Vec<NotionObject>) -> Vec<Page> {
    objects
        .into_iter()
        .filter_map(|obj| match obj {
            NotionObject::Page(page) => Some(page),
            _ => None,
        })
        .collect()
}

/// Recreates a ChildDatabaseBlock when the original was overwritten in the graph.
fn recreate_child_database_block(
    database: &Database,
    db_id: &NotionId,
    block_to_db: &HashMap<NotionId, NotionId>,
) -> Block {
    let original_block_id = block_to_db
        .iter()
        .find(|(_, v)| *v == db_id)
        .map(|(k, _)| k.as_str())
        .unwrap_or(db_id.as_str());

    Block::ChildDatabase(crate::model::blocks::ChildDatabaseBlock {
        common: crate::model::BlockCommon {
            id: crate::types::BlockId::parse(original_block_id).unwrap_or_else(|e| {
                log::error!(
                    "Invalid BlockId '{}': {}. Using default UUID.",
                    original_block_id,
                    e
                );
                crate::types::BlockId::new_v4()
            }),
            children: Vec::new(),
            has_children: false,
            archived: false,
        },
        title: database.title().as_plain_text(),
        content: crate::model::blocks::ChildDatabaseContent::NotFetched,
    })
}

/// Embeds databases into their corresponding ChildDatabaseBlocks.
///
/// Walks the block tree. For each ChildDatabaseBlock, looks up the matching
/// database by ID (via the block→database mapping) and embeds it.
fn embed_databases(
    blocks: Vec<Block>,
    databases: &mut HashMap<NotionId, Database>,
    block_to_db_mapping: &HashMap<NotionId, NotionId>,
) -> Vec<Block> {
    log::debug!(
        "Embedding {} databases into {} blocks",
        databases.len(),
        blocks.len()
    );

    let mut embedded_count = 0;
    let total_databases = databases.len();

    let result: Vec<Block> = blocks
        .into_iter()
        .map(|block| {
            embed_database_if_present(block, databases, block_to_db_mapping, &mut embedded_count)
        })
        .collect();

    log::debug!(
        "Embedding complete: {}/{} databases embedded",
        embedded_count,
        total_databases
    );

    if !databases.is_empty() {
        log::warn!(
            "Unmatched databases after embedding: [{}]",
            databases
                .keys()
                .map(|k| format!("'{}'", k.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    result
}

/// Embeds a database into a block if it's a ChildDatabaseBlock,
/// then recurses into children.
fn embed_database_if_present(
    mut block: Block,
    databases: &mut HashMap<NotionId, Database>,
    block_to_db_mapping: &HashMap<NotionId, NotionId>,
    embedded_count: &mut usize,
) -> Block {
    if let Block::ChildDatabase(ref mut child_db) = &mut block {
        let block_id: NotionId = child_db.common.id.clone().into();
        let db_id = block_to_db_mapping.get(&block_id).unwrap_or(&block_id);

        if let Some(database) = databases.remove(db_id) {
            log::debug!(
                "Embedded '{}' into block {}",
                database.title().as_plain_text(),
                block_id.as_str()
            );
            child_db.content =
                crate::model::blocks::ChildDatabaseContent::Fetched(Box::new(database));
            *embedded_count += 1;
        }
    }

    // Recurse into children
    if block.has_children() && !block.children().is_empty() {
        let children = block.children().clone();
        let enriched = embed_databases(children, databases, block_to_db_mapping);
        block.set_children(enriched);
    }

    block
}
