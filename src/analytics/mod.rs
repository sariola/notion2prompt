// src/analytics/mod.rs
//! Content measurement and statistics for Notion object trees.

#![allow(dead_code)]

use crate::model::{Block, BlockVisitor, Database, NotionObject, Page};
use crate::types::BlockId;

/// Quick statistics for user-facing progress messages.
///
/// Use this for lightweight summaries shown to the user (e.g., "Fetched 42 objects").
#[derive(Debug, Clone, Default)]
pub struct ContentSummary {
    pub total_objects: usize,
    pub deepest_nesting: usize,
}

/// Detailed content breakdown for diagnostics and logging.
///
/// Use this for comprehensive metrics (e.g., verbose mode, debugging).
#[derive(Debug, Clone, Default)]
pub struct ContentMeasurement {
    pub total_objects: usize,
    pub pages: usize,
    pub databases: usize,
    pub blocks: usize,
    pub deepest_nesting: usize,
    pub embedded_databases: usize,
    pub pages_with_content: usize,
    pub total_pages: usize,
}

/// Counts databases embedded within block trees (child_database blocks).
struct EmbeddedDatabaseCounter {
    count: usize,
}

impl EmbeddedDatabaseCounter {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl BlockVisitor for EmbeddedDatabaseCounter {
    type Output = ();

    fn visit_child_database(
        &mut self,
        _id: &BlockId,
        _database: &crate::model::ChildDatabaseBlock,
    ) {
        self.count += 1;
    }
}

/// Measures basic content statistics for a Notion object.
pub fn measure_content(object: &NotionObject) -> ContentSummary {
    ContentSummary {
        total_objects: total_item_count(object),
        deepest_nesting: deepest_nesting_level(object),
    }
}

/// Measures detailed content metrics for a Notion object.
pub fn measure_content_detailed(object: &NotionObject) -> ContentMeasurement {
    let mut metrics = ContentMeasurement::default();
    walk_object(&mut metrics, object, 0);
    metrics
}

/// Counts the total number of items in a Notion object tree.
pub fn total_item_count(object: &NotionObject) -> usize {
    match object {
        NotionObject::Page(page) => count_page_items(page),
        NotionObject::Database(db) => count_database_items(db),
        NotionObject::Block(block) => 1 + count_blocks(&block.children().to_vec()),
    }
}

fn count_page_items(page: &Page) -> usize {
    1 + count_blocks(&page.blocks)
}

fn count_database_items(db: &Database) -> usize {
    1 + db.pages.iter().map(count_page_items).sum::<usize>()
}

/// Counts blocks recursively.
pub fn count_blocks(blocks: &[Block]) -> usize {
    blocks.len()
        + blocks
            .iter()
            .map(|b| count_blocks(b.children()))
            .sum::<usize>()
}

/// Returns the deepest nesting level of a Notion object.
pub fn deepest_nesting_level(object: &NotionObject) -> usize {
    match object {
        NotionObject::Page(page) => blocks_max_depth(&page.blocks, 1),
        NotionObject::Database(db) => db
            .pages
            .iter()
            .map(|page| blocks_max_depth(&page.blocks, 1))
            .max()
            .unwrap_or(0),
        NotionObject::Block(block) => blocks_max_depth(&[block.clone()], 1),
    }
}

/// Gets the maximum depth of a block tree.
pub fn blocks_max_depth(blocks: &[Block], current_depth: usize) -> usize {
    blocks
        .iter()
        .map(|block| {
            if block.children().is_empty() {
                current_depth
            } else {
                blocks_max_depth(block.children(), current_depth + 1)
            }
        })
        .max()
        .unwrap_or(current_depth)
}

/// Counts databases embedded within blocks (child_database blocks in the tree).
pub fn embedded_database_count(blocks: &[Block]) -> usize {
    let mut counter = EmbeddedDatabaseCounter::new();

    for block in blocks {
        block.accept(&mut counter);
        counter.count += embedded_database_count(block.children());
    }

    counter.count
}

fn walk_object(measurement: &mut ContentMeasurement, obj: &NotionObject, depth: usize) {
    measurement.total_objects += 1;
    measurement.deepest_nesting = measurement.deepest_nesting.max(depth);

    match obj {
        NotionObject::Page(page) => {
            measurement.pages += 1;
            walk_blocks(measurement, &page.blocks, depth + 1);
        }
        NotionObject::Database(db) => {
            measurement.databases += 1;
            measurement.total_pages += db.pages.len();

            for page in &db.pages {
                measurement.pages += 1;
                measurement.total_objects += 1;

                if !page.blocks.is_empty() {
                    measurement.pages_with_content += 1;
                }

                walk_blocks(measurement, &page.blocks, depth + 2);
            }
        }
        NotionObject::Block(block) => {
            measurement.blocks += 1;
            walk_blocks(measurement, &[block.clone()], depth + 1);
        }
    }
}

fn walk_blocks(measurement: &mut ContentMeasurement, blocks: &[Block], depth: usize) {
    measurement.deepest_nesting = measurement.deepest_nesting.max(depth);

    for block in blocks {
        measurement.total_objects += 1;
        measurement.blocks += 1;

        let mut counter = EmbeddedDatabaseCounter::new();
        block.accept(&mut counter);
        measurement.embedded_databases += counter.count;

        walk_blocks(measurement, block.children(), depth + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Page, PageTitle};
    use crate::types::PageId;

    #[test]
    fn test_empty_page_stats() {
        let page = Page {
            id: PageId::parse("12345678123456781234567812345678").unwrap(),
            title: PageTitle::new("Test Page"),
            url: "https://notion.so/test".to_string(),
            blocks: vec![],
            properties: Default::default(),
            parent: None,
            archived: false,
        };

        let obj = NotionObject::Page(page);
        let stats = measure_content(&obj);

        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.deepest_nesting, 1);
    }
}
