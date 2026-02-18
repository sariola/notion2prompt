// src/formatting/mod.rs
//! Renders Notion data structures into markdown and assembles prompts.

// Sub-modules
pub mod block_renderer;
pub mod databases;
pub mod direct_template;
mod properties;
mod pure_visitor;
mod rich_text;
mod state;

// --- Prompt Rendering (top-level entry point) ---
#[allow(unused_imports)] // Used by bin crate
pub use self::direct_template::render_prompt;

use crate::model::{Database, NotionObject};
use crate::types::NotionId;
use std::collections::HashMap;

/// Gathers all embedded databases from a NotionObject tree.
pub fn gather_embedded_databases(obj: &NotionObject) -> HashMap<NotionId, Database> {
    let mut databases = HashMap::new();
    walk_tree_for_databases(obj, &mut databases);
    log::debug!("Gathered {} databases", databases.len());
    databases
}

fn walk_tree_for_databases(obj: &NotionObject, databases: &mut HashMap<NotionId, Database>) {
    match obj {
        NotionObject::Database(db) => {
            databases.insert(NotionId::from(&db.id), db.clone());
            for page in &db.pages {
                let page_obj = NotionObject::Page(page.clone());
                walk_tree_for_databases(&page_obj, databases);
            }
        }
        NotionObject::Page(page) => {
            for block in &page.blocks {
                collect_database_from_block(block, databases);
            }
        }
        NotionObject::Block(block) => {
            collect_database_from_block(block, databases);
        }
    }
}

fn collect_database_from_block(
    block: &crate::model::Block,
    databases: &mut HashMap<NotionId, Database>,
) {
    use crate::model::Block;

    if let Block::ChildDatabase(child_db) = block {
        if let crate::model::blocks::ChildDatabaseContent::Fetched(ref db) = child_db.content {
            databases.insert(NotionId::from(&db.id), db.as_ref().clone());
        }
    }

    for child in block.children() {
        collect_database_from_block(child, databases);
    }
}
