// benches/formatting_bench.rs
//! Benchmarks for formatting performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use notion2prompt::formatting::{
    core::format_block_recursive,
    parallel_rayon::ParallelFormatter,
    pure_visitor::{PureBlockVisitor, PureMarkdownFormatter},
    state::FormatContext,
    streaming::{StreamingFormatter, StreamingVisitor},
};
use notion2prompt::model::{Block, BlockCommon, DividerBlock, HeadingBlock, ParagraphBlock};
use notion2prompt::types::{BlockId, RichTextItem};
use std::io::Cursor;

/// Create a sample block tree for benchmarking
fn create_sample_blocks(depth: usize, breadth: usize) -> Vec<Box<Block>> {
    fn create_block_tree(level: usize, max_depth: usize, breadth: usize) -> Box<Block> {
        let id = format!("block-{}-{}", level, rand::random::<u32>());
        let children = if level < max_depth {
            (0..breadth)
                .map(|_| create_block_tree(level + 1, max_depth, breadth))
                .collect()
        } else {
            vec![]
        };

        if level % 3 == 0 {
            Box::new(Block::Heading1(HeadingBlock {
                common: BlockCommon {
                    id: BlockId::parse(&id).unwrap(),
                    children,
                    has_children: !children.is_empty(),
                    archived: false,
                },
                content: notion2prompt::model::HeadingContent {
                    rich_text: vec![RichTextItem {
                        plain_text: format!("Heading at level {}", level),
                        ..Default::default()
                    }],
                },
                color: notion2prompt::types::Color::Default,
                is_toggleable: false,
            }))
        } else if level % 3 == 1 {
            Box::new(Block::Paragraph(ParagraphBlock {
                common: BlockCommon {
                    id: BlockId::parse(&id).unwrap(),
                    children,
                    has_children: !children.is_empty(),
                    archived: false,
                },
                content: notion2prompt::model::ParagraphContent {
                    rich_text: vec![RichTextItem {
                        plain_text: format!("This is a paragraph at level {} with some content to make it more realistic", level),
                        ..Default::default()
                    }],
                },
                color: notion2prompt::types::Color::Default,
            }))
        } else {
            Box::new(Block::Divider(DividerBlock {
                common: BlockCommon {
                    id: BlockId::parse(&id).unwrap(),
                    children,
                    has_children: !children.is_empty(),
                    archived: false,
                },
            }))
        }
    }

    (0..breadth)
        .map(|_| create_block_tree(0, depth, breadth))
        .collect()
}

fn bench_formatting_approaches(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting");

    // Test different tree sizes
    let tree_configs = vec![
        (3, 3, "small"),  // 3 levels, 3 children per node
        (4, 4, "medium"), // 4 levels, 4 children per node
        (5, 5, "large"),  // 5 levels, 5 children per node
    ];

    for (depth, breadth, name) in tree_configs {
        let blocks = create_sample_blocks(depth, breadth);
        let block_count = count_blocks(&blocks);

        // Benchmark traditional formatting
        group.bench_with_input(
            BenchmarkId::new("traditional", name),
            &blocks,
            |b, blocks| {
                b.iter(|| {
                    let mut output = String::new();
                    let context = FormatContext::new();
                    for block in blocks {
                        let _ = format_block_recursive(
                            black_box(block.as_ref()),
                            black_box(&mut output),
                            black_box(&context),
                            black_box(&Default::default()),
                        );
                    }
                    output
                });
            },
        );

        // Benchmark pure visitor formatting
        group.bench_with_input(
            BenchmarkId::new("pure_visitor", name),
            &blocks,
            |b, blocks| {
                b.iter(|| {
                    let formatter = PureMarkdownFormatter::new(&Default::default());
                    let mut context = FormatContext::new();
                    let mut output = String::new();
                    for block in blocks {
                        if let Ok(result) =
                            formatter.visit_block(black_box(block.as_ref()), black_box(context))
                        {
                            output.push_str(&result.content);
                            context = result.context;
                        }
                    }
                    output
                });
            },
        );

        // Benchmark streaming formatting
        group.bench_with_input(BenchmarkId::new("streaming", name), &blocks, |b, blocks| {
            b.iter(|| {
                let formatter = PureMarkdownFormatter::new(&Default::default());
                let streaming = StreamingVisitor::new(formatter);
                let mut output = Vec::new();
                let mut cursor = Cursor::new(&mut output);
                let context = FormatContext::new();
                let _ = streaming.format_blocks_to_writer(
                    black_box(blocks),
                    black_box(&mut cursor),
                    black_box(context),
                );
                output
            });
        });

        // Benchmark parallel formatting (only for larger trees)
        if block_count > 100 {
            group.bench_with_input(BenchmarkId::new("parallel", name), &blocks, |b, blocks| {
                b.iter(|| {
                    let formatter = PureMarkdownFormatter::new(&Default::default());
                    let parallel = ParallelFormatter::new(formatter, None);
                    parallel.format_blocks_parallel(black_box(blocks))
                });
            });
        }
    }

    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    use notion2prompt::performance::interning::{InternedString, StringInterner};
    use notion2prompt::performance::string_builder::StringBuilder;

    let mut group = c.benchmark_group("string_operations");

    // Benchmark string building
    let parts = vec![
        "Hello",
        " ",
        "World",
        "!",
        " This is a longer string to test performance.",
    ];

    group.bench_function("std_string_concat", |b| {
        b.iter(|| {
            let mut s = String::new();
            for part in &parts {
                s.push_str(black_box(part));
            }
            s
        });
    });

    group.bench_function("string_builder", |b| {
        b.iter(|| {
            let mut builder = StringBuilder::new();
            for part in &parts {
                builder.push(black_box(part));
            }
            builder.build()
        });
    });

    // Benchmark string interning
    let strings = (0..1000)
        .map(|i| format!("string_{}", i % 100)) // 100 unique strings, repeated 10 times
        .collect::<Vec<_>>();

    group.bench_function("without_interning", |b| {
        b.iter(|| {
            let mut collection = Vec::new();
            for s in &strings {
                collection.push(black_box(s.clone()));
            }
            collection
        });
    });

    group.bench_function("with_interning", |b| {
        let interner = StringInterner::new();
        b.iter(|| {
            let mut collection = Vec::new();
            for s in &strings {
                collection.push(black_box(interner.intern(s)));
            }
            collection
        });
    });

    group.finish();
}

fn bench_memory_efficient_processing(c: &mut Criterion) {
    use notion2prompt::performance::efficient_blocks::{
        BlockArena, ChunkedBlockProcessor, CompactBlock, CompactContent,
    };
    use notion2prompt::performance::lazy::{LazyMap, LazyValue};
    use std::borrow::Cow;

    let mut group = c.benchmark_group("memory_efficient");

    let blocks = create_sample_blocks(4, 4);

    // Benchmark chunked processing
    group.bench_with_input(
        BenchmarkId::new("chunked_processing", "medium"),
        &blocks,
        |b, blocks| {
            b.iter(|| {
                let processor = ChunkedBlockProcessor::new(50, 1024 * 1024); // 50 blocks, 1MB
                let _ = processor.process_blocks(black_box(blocks), |chunk| {
                    // Simulate processing
                    Ok(chunk.iter().map(|b| format!("{:?}", b)).collect::<Vec<_>>())
                });
            });
        },
    );

    // Benchmark block arena
    group.bench_function("block_arena", |b| {
        b.iter(|| {
            let mut arena = BlockArena::with_capacity(1000);
            for i in 0..1000 {
                let block = CompactBlock {
                    id: Cow::Borrowed("test"),
                    block_type: InternedString::new("paragraph"),
                    content: CompactContent::Text(Cow::Borrowed("Sample text")),
                    child_indices: vec![],
                };
                arena.add_block(black_box(block));
            }
            arena
        });
    });

    // Benchmark lazy evaluation
    group.bench_function("lazy_value", |b| {
        b.iter(|| {
            let lazy = LazyValue::new(|| {
                // Simulate expensive computation
                (0..1000).map(|i| i * i).sum::<i32>()
            });

            // Access the value multiple times
            for _ in 0..10 {
                let _ = black_box(*lazy);
            }
        });
    });

    group.bench_function("lazy_map", |b| {
        let map = LazyMap::new(|k: &i32| {
            // Simulate expensive computation
            k * k + k * 2 + 1
        });

        b.iter(|| {
            let mut sum = 0;
            for i in 0..100 {
                sum += black_box(map.get(&i));
            }
            sum
        });
    });

    group.finish();
}

fn count_blocks(blocks: &[Box<Block>]) -> usize {
    let mut count = blocks.len();
    for block in blocks {
        count += count_blocks(block.children());
    }
    count
}

criterion_group!(
    benches,
    bench_formatting_approaches,
    bench_string_operations,
    bench_memory_efficient_processing
);
criterion_main!(benches);
