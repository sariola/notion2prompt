// benches/api_bench.rs
//! Benchmarks for API performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use notion2prompt::api::parser::{parse_block, parse_database_properties};
use notion2prompt::performance::zero_copy::{StringView, ZeroCopyParser};
use serde_json::json;

fn create_sample_block_json(text_length: usize) -> serde_json::Value {
    let text = "a".repeat(text_length);
    json!({
        "object": "block",
        "id": "12345678-1234-1234-1234-123456789abc",
        "parent": {
            "type": "page_id",
            "page_id": "87654321-4321-4321-4321-cba987654321"
        },
        "created_time": "2024-01-01T00:00:00.000Z",
        "last_edited_time": "2024-01-01T00:00:00.000Z",
        "created_by": {
            "object": "user",
            "id": "11111111-1111-1111-1111-111111111111"
        },
        "last_edited_by": {
            "object": "user",
            "id": "11111111-1111-1111-1111-111111111111"
        },
        "has_children": false,
        "archived": false,
        "type": "paragraph",
        "paragraph": {
            "rich_text": [{
                "type": "text",
                "text": {
                    "content": text,
                    "link": null
                },
                "annotations": {
                    "bold": false,
                    "italic": false,
                    "strikethrough": false,
                    "underline": false,
                    "code": false,
                    "color": "default"
                },
                "plain_text": text,
                "href": null
            }],
            "color": "default"
        }
    })
}

fn create_sample_database_json(num_properties: usize) -> serde_json::Value {
    let mut properties = serde_json::Map::new();

    for i in 0..num_properties {
        let prop_name = format!("Property_{}", i);
        let prop_type = match i % 5 {
            0 => json!({
                "id": format!("prop_{}", i),
                "name": prop_name,
                "type": "title",
                "title": {}
            }),
            1 => json!({
                "id": format!("prop_{}", i),
                "name": prop_name,
                "type": "number",
                "number": {
                    "format": "number"
                }
            }),
            2 => json!({
                "id": format!("prop_{}", i),
                "name": prop_name,
                "type": "select",
                "select": {
                    "options": [
                        {"id": "opt1", "name": "Option 1", "color": "red"},
                        {"id": "opt2", "name": "Option 2", "color": "blue"}
                    ]
                }
            }),
            3 => json!({
                "id": format!("prop_{}", i),
                "name": prop_name,
                "type": "checkbox",
                "checkbox": {}
            }),
            _ => json!({
                "id": format!("prop_{}", i),
                "name": prop_name,
                "type": "rich_text",
                "rich_text": {}
            }),
        };
        properties.insert(prop_name, prop_type);
    }

    json!({
        "object": "database",
        "id": "12345678-1234-1234-1234-123456789abc",
        "created_time": "2024-01-01T00:00:00.000Z",
        "last_edited_time": "2024-01-01T00:00:00.000Z",
        "title": [{
            "type": "text",
            "text": {
                "content": "Test Database",
                "link": null
            },
            "plain_text": "Test Database"
        }],
        "properties": properties,
        "parent": {
            "type": "page_id",
            "page_id": "87654321-4321-4321-4321-cba987654321"
        },
        "archived": false
    })
}

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    // Test different text sizes
    let text_sizes = vec![(100, "small"), (1000, "medium"), (10000, "large")];

    for (size, name) in text_sizes {
        let json = create_sample_block_json(size);
        let json_str = json.to_string();

        group.bench_with_input(
            BenchmarkId::new("parse_block_serde", name),
            &json,
            |b, json| {
                b.iter(|| parse_block(black_box(json.clone())));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_block_zero_copy", name),
            &json_str,
            |b, json_str| {
                b.iter(|| {
                    let parser = ZeroCopyParser::new(black_box(json_str));
                    // Simulate parsing with zero-copy
                    let view = parser.parse_string_field("id");
                    match view {
                        StringView::Borrowed(s) => s,
                        _ => "",
                    }
                });
            },
        );
    }

    // Test database parsing with different property counts
    let property_counts = vec![(10, "small"), (50, "medium"), (100, "large")];

    for (count, name) in property_counts {
        let json = create_sample_database_json(count);

        group.bench_with_input(
            BenchmarkId::new("parse_database_properties", name),
            &json,
            |b, json| {
                b.iter(|| {
                    if let Some(props) = json.get("properties") {
                        parse_database_properties(black_box(props))
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_id_parsing(c: &mut Criterion) {
    use notion2prompt::types::{BlockId, DatabaseId, NotionId};

    let mut group = c.benchmark_group("id_parsing");

    let id_formats = vec![
        ("12345678-1234-1234-1234-123456789abc", "uuid_with_dashes"),
        ("123456781234123412341234567890ab", "uuid_without_dashes"),
        (
            "https://www.notion.so/Test-Page-123456781234123412341234567890ab",
            "full_url",
        ),
    ];

    for (id, name) in &id_formats {
        group.bench_with_input(BenchmarkId::new("parse_notion_id", name), id, |b, id| {
            b.iter(|| NotionId::parse(black_box(id)));
        });

        group.bench_with_input(BenchmarkId::new("parse_block_id", name), id, |b, id| {
            b.iter(|| BlockId::parse(black_box(id)));
        });

        group.bench_with_input(BenchmarkId::new("parse_database_id", name), id, |b, id| {
            b.iter(|| DatabaseId::parse(black_box(id)));
        });
    }

    group.finish();
}

fn bench_zero_copy_operations(c: &mut Criterion) {
    use notion2prompt::performance::zero_copy::{zero_copy_parse, StringView};

    let mut group = c.benchmark_group("zero_copy");

    let json_sizes = vec![(100, "small"), (1000, "medium"), (10000, "large")];

    for (size, name) in json_sizes {
        let json = create_sample_block_json(size);
        let json_str = json.to_string();

        group.bench_with_input(
            BenchmarkId::new("traditional_parse", name),
            &json_str,
            |b, json_str| {
                b.iter(|| {
                    let value: serde_json::Value =
                        serde_json::from_str(black_box(json_str)).unwrap();
                    value
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("zero_copy_parse", name),
            &json_str,
            |b, json_str| {
                b.iter(|| zero_copy_parse(black_box(json_str)));
            },
        );
    }

    // Benchmark StringView operations
    let test_strings = vec![
        "short",
        "a medium length string that's not too long",
        &"a".repeat(1000),
    ];

    group.bench_function("string_view_creation", |b| {
        b.iter(|| {
            let mut views = Vec::new();
            for s in &test_strings {
                views.push(StringView::from(black_box(*s)));
            }
            views
        });
    });

    group.bench_function("string_view_to_owned", |b| {
        let views: Vec<StringView> = test_strings.iter().map(|s| StringView::from(*s)).collect();

        b.iter(|| {
            let mut owned = Vec::new();
            for view in &views {
                owned.push(black_box(view.to_owned()));
            }
            owned
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_parsing,
    bench_id_parsing,
    bench_zero_copy_operations
);
criterion_main!(benches);
