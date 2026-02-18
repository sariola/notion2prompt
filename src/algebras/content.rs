//! Content retrieval algebra for Notion API.
//!
//! This module defines the [`NotionContent`] capability trait for retrieving
//! content from the Notion API. The trait is intentionally focused on core
//! operations without implementation details like HTTP, retries, or caching.

use crate::model::{Block, Database, Page};
use crate::types::NotionId;
use async_trait::async_trait;

use super::error::FetchError;

/// Content retrieval capability for Notion API.
///
/// This trait represents the ability to fetch content from Notion. It is
/// **object-safe** and can be used as `dyn NotionContent`.
///
/// # Laws
///
/// All implementations must satisfy these laws:
///
/// - **L1 (Idempotency)**: Fetching the same ID twice returns identical content.
///   ```text
///   retrieve_page(id) == p1
///   retrieve_page(id) == p2
///   assert_eq!(p1, p2)
///   ```
///
/// - **L2 (Children-Consistency)**: `retrieve_children` returns children of the given parent.
///   ```text
///   retrieve_children(parent) == children
///   for child in children:
///       assert(child.parent == parent)
///   ```
///
/// - **L3 (Database-Rows)**: `query_rows` returns pages that belong to the database.
///   ```text
///   retrieve_database(db_id) == db
///   query_rows(db_id) == rows
///   for row in rows:
///       assert(row.parent.database_id == db_id)
///   ```
///
/// - **L4 (Block-Identity)**: A retrieved block's ID matches the requested ID.
///   ```text
///   retrieve_block(id) == block
///   assert(block.id == id)
///   ```
///
/// # Object Safety
///
/// This trait is object-safe and can be used as `Arc<dyn NotionContent>`.
/// Methods use concrete domain types (`Page`, `Database`, `Block`) rather than
/// generics to enable dynamic dispatch.
#[async_trait]
pub trait NotionContent: Send + Sync {
    // -----------------------------------------------------------------------
    // Core retrieval operations
    // -----------------------------------------------------------------------

    /// Retrieve a page by its ID.
    ///
    /// Returns the page with its metadata, properties, and content.
    /// The page's blocks are typically fetched separately via `retrieve_children`.
    async fn retrieve_page(&self, id: &NotionId) -> Result<Page, FetchError>;

    /// Retrieve a database by its ID.
    ///
    /// Returns the database with its schema (properties, title).
    /// The database's rows (pages) are typically fetched separately via `query_rows`.
    async fn retrieve_database(&self, id: &NotionId) -> Result<Database, FetchError>;

    /// Retrieve a single block by its ID.
    ///
    /// Returns the block with its content. The block's children are typically
    /// fetched separately via `retrieve_children`.
    async fn retrieve_block(&self, id: &NotionId) -> Result<Block, FetchError>;

    /// Retrieve the children blocks of a parent.
    ///
    /// The parent can be a page, block, or database. Returns all direct
    /// children (not nested grandchildren).
    async fn retrieve_children(&self, parent: &NotionId) -> Result<Vec<Block>, FetchError>;

    /// Query the rows of a database.
    ///
    /// Returns all pages (rows) in the database, respecting pagination.
    /// Each row includes its properties and content.
    async fn query_rows(&self, database: &NotionId) -> Result<Vec<Page>, FetchError>;
}

// ==============================================================================
// Extension Trait for NotionContent
// ==============================================================================

/// Extension trait with typed convenience methods for [`NotionContent`].
///
/// This trait provides higher-level operations built from the base trait methods.
/// It is **NOT object-safe** due to generic methods but provides a blanket
/// implementation for all `NotionContent` types.
#[async_trait]
pub trait NotionContentExt: NotionContent {
    /// Retrieve a page and all its descendants recursively.
    ///
    /// This is a convenience method that combines `retrieve_page` with
    /// `retrieve_children` to build a complete page tree.
    async fn retrieve_page_recursive(
        &self,
        id: &NotionId,
        max_depth: u8,
    ) -> Result<Page, FetchError> {
        let mut page = self.retrieve_page(id).await?;
        fetch_children_recursive(self, &mut page.blocks, max_depth).await?;
        Ok(page)
    }

    /// Retrieve a database with all its rows.
    ///
    /// This is a convenience method that combines `retrieve_database` with
    /// `query_rows` to get a complete database.
    async fn retrieve_database_with_rows(&self, id: &NotionId) -> Result<Database, FetchError> {
        let mut database = self.retrieve_database(id).await?;
        database.pages = self.query_rows(id).await?;
        Ok(database)
    }

    /// Check if an ID refers to a valid, accessible object.
    ///
    /// Attempts to fetch the object and returns true if successful.
    async fn exists(&self, id: &NotionId) -> bool {
        // Try page first, then database, then block
        if self.retrieve_page(id).await.is_ok() {
            return true;
        }
        if self.retrieve_database(id).await.is_ok() {
            return true;
        }
        if self.retrieve_block(id).await.is_ok() {
            return true;
        }
        false
    }
}

// Helper function for recursive fetching
async fn fetch_children_recursive<C: NotionContent + ?Sized>(
    content: &C,
    blocks: &mut [Block],
    depth: u8,
) -> Result<(), FetchError> {
    if depth == 0 {
        return Ok(());
    }

    for block in blocks.iter_mut() {
        if block.has_children() {
            let block_id = NotionId::from(block.id());
            let children = content.retrieve_children(&block_id).await?;
            block.set_children(children);

            // Recursively fetch grandchildren
            let child_blocks = block.children_mut();
            Box::pin(fetch_children_recursive(content, child_blocks, depth - 1)).await?;
        }
    }
    Ok(())
}

// Blanket implementation for all NotionContent
#[async_trait]
impl<T: NotionContent + ?Sized> NotionContentExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BlockId, DatabaseId, PageId};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// In-memory mock NotionContent for testing.
    ///
    /// Pre-load pages, databases, and blocks for deterministic tests.
    pub struct MockNotionContent {
        pages: Arc<RwLock<HashMap<String, Page>>>,
        databases: Arc<RwLock<HashMap<String, Database>>>,
        blocks: Arc<RwLock<HashMap<String, Block>>>,
        children: Arc<RwLock<HashMap<String, Vec<Block>>>>,
        rows: Arc<RwLock<HashMap<String, Vec<Page>>>>,
    }

    impl MockNotionContent {
        /// Create a new empty mock content store.
        pub fn new() -> Self {
            Self {
                pages: Arc::new(RwLock::new(HashMap::new())),
                databases: Arc::new(RwLock::new(HashMap::new())),
                blocks: Arc::new(RwLock::new(HashMap::new())),
                children: Arc::new(RwLock::new(HashMap::new())),
                rows: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        /// Add a page to the mock store.
        pub async fn add_page(&self, page: Page) {
            let mut pages = self.pages.write().await;
            pages.insert(page.id.as_str().to_string(), page);
        }

        /// Add a database to the mock store.
        #[allow(dead_code)]
        pub async fn add_database(&self, database: Database) {
            let mut databases = self.databases.write().await;
            databases.insert(database.id.as_str().to_string(), database);
        }

        /// Add a block to the mock store.
        pub async fn add_block(&self, block: Block) {
            let mut blocks = self.blocks.write().await;
            blocks.insert(block.id().as_str().to_string(), block);
        }

        /// Add children for a parent.
        pub async fn add_children(&self, parent: &NotionId, children: Vec<Block>) {
            let mut all_children = self.children.write().await;
            all_children.insert(parent.as_str().to_string(), children);
        }

        /// Add rows for a database.
        pub async fn add_rows(&self, database: &NotionId, rows: Vec<Page>) {
            let mut all_rows = self.rows.write().await;
            all_rows.insert(database.as_str().to_string(), rows);
        }
    }

    impl Default for MockNotionContent {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl NotionContent for MockNotionContent {
        async fn retrieve_page(&self, id: &NotionId) -> Result<Page, FetchError> {
            let pages = self.pages.read().await;
            pages
                .get(id.as_str())
                .cloned()
                .ok_or_else(|| FetchError::NotFound {
                    id: id.as_str().to_string(),
                })
        }

        async fn retrieve_database(&self, id: &NotionId) -> Result<Database, FetchError> {
            let databases = self.databases.read().await;
            databases
                .get(id.as_str())
                .cloned()
                .ok_or_else(|| FetchError::NotFound {
                    id: id.as_str().to_string(),
                })
        }

        async fn retrieve_block(&self, id: &NotionId) -> Result<Block, FetchError> {
            let blocks = self.blocks.read().await;
            blocks
                .get(id.as_str())
                .cloned()
                .ok_or_else(|| FetchError::NotFound {
                    id: id.as_str().to_string(),
                })
        }

        async fn retrieve_children(&self, parent: &NotionId) -> Result<Vec<Block>, FetchError> {
            let children = self.children.read().await;
            Ok(children.get(parent.as_str()).cloned().unwrap_or_default())
        }

        async fn query_rows(&self, database: &NotionId) -> Result<Vec<Page>, FetchError> {
            let rows = self.rows.read().await;
            Ok(rows.get(database.as_str()).cloned().unwrap_or_default())
        }
    }

    // ========================================================================
    // Law Tests
    // ========================================================================

    /// L1: Idempotency - Fetching same ID twice returns identical content
    #[tokio::test]
    async fn law_l1_idempotency() {
        let content = MockNotionContent::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // Create and add a test page
        let page = Page {
            id: PageId::parse("550e8400e29b41d4a716446655440000").unwrap(),
            title: crate::model::PageTitle::new("Test"),
            url: "https://notion.so/test".to_string(),
            blocks: vec![],
            properties: HashMap::new(),
            parent: None,
            archived: false,
        };
        content.add_page(page).await;

        // Fetch twice — both return cloned data from the store
        let first = content.retrieve_page(&id).await.unwrap();
        let second = content.retrieve_page(&id).await.unwrap();

        assert_eq!(first, second);
    }

    /// L2: Children-Consistency - retrieve_children returns children of the given parent
    #[tokio::test]
    async fn law_l2_children_consistency() {
        let content = MockNotionContent::new();
        let parent_id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // Create children with the correct parent
        let child1 = Block::Paragraph(crate::model::ParagraphBlock {
            common: crate::model::BlockCommon {
                id: BlockId::parse("550e8400e29b41d4a716446655440001").unwrap(),
                children: vec![],
                has_children: false,
                archived: false,
            },
            content: crate::model::TextBlockContent::default(),
        });

        content.add_children(&parent_id, vec![child1.clone()]).await;

        // Retrieve children
        let children = content.retrieve_children(&parent_id).await.unwrap();

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), child1.id());
    }

    /// L3: Database-Rows - query_rows returns pages that belong to the database
    #[tokio::test]
    async fn law_l3_database_rows() {
        let content = MockNotionContent::new();
        let db_id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // Create a row page with this database as parent
        let row = Page {
            id: PageId::parse("550e8400e29b41d4a716446655440001").unwrap(),
            title: crate::model::PageTitle::new("Row"),
            url: "https://notion.so/row".to_string(),
            blocks: vec![],
            properties: HashMap::new(),
            parent: Some(crate::model::Parent::Database {
                database_id: DatabaseId::parse("550e8400e29b41d4a716446655440000").unwrap(),
            }),
            archived: false,
        };

        content.add_rows(&db_id, vec![row.clone()]).await;

        // Query rows
        let rows = content.query_rows(&db_id).await.unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, row.id);
        assert!(matches!(
            rows[0].parent,
            Some(crate::model::Parent::Database { .. })
        ));
    }

    /// L4: Block-Invariant - A retrieved block's ID is consistent
    #[tokio::test]
    async fn law_l4_block_id_invariant() {
        let content = MockNotionContent::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        let block = Block::Paragraph(crate::model::ParagraphBlock {
            common: crate::model::BlockCommon {
                id: BlockId::parse("550e8400e29b41d4a716446655440000").unwrap(),
                children: vec![],
                has_children: false,
                archived: false,
            },
            content: crate::model::TextBlockContent::default(),
        });

        content.add_block(block.clone()).await;

        // Retrieve block
        let retrieved = content.retrieve_block(&id).await.unwrap();

        // ID should match
        assert_eq!(retrieved.id().as_str(), id.as_str());
    }

    // ========================================================================
    // Extension Trait Tests
    // ========================================================================

    #[tokio::test]
    async fn exists_returns_true_for_valid_objects() {
        let content = MockNotionContent::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        let page = Page {
            id: PageId::parse("550e8400e29b41d4a716446655440000").unwrap(),
            title: crate::model::PageTitle::new("Test"),
            url: "https://notion.so/test".to_string(),
            blocks: vec![],
            properties: HashMap::new(),
            parent: None,
            archived: false,
        };
        content.add_page(page).await;

        assert!(content.exists(&id).await);
    }

    #[tokio::test]
    async fn exists_returns_false_for_invalid_objects() {
        let content = MockNotionContent::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        assert!(!content.exists(&id).await);
    }
}
