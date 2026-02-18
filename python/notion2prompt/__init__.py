"""notion2prompt — Convert Notion pages and databases into structured prompts.

High-performance Notion content fetcher and prompt renderer, powered by Rust.

Usage:
    import asyncio
    import notion2prompt

    # One-shot: fetch and render in a single call
    prompt = asyncio.run(notion2prompt.fetch_and_render(
        "https://www.notion.so/your-page-id",
        api_key="secret_...",
    ))

    # Two-stage: fetch first, render later (possibly with different templates)
    content = asyncio.run(notion2prompt.fetch_content("your-page-id"))
    prompt = notion2prompt.render_content(content, template="claude-xml")
"""

from notion2prompt._notion2prompt import (
    PipelineConfig,
    NotionContent,
    fetch_and_render,
    fetch_content,
    render_content,
)

__all__ = [
    "PipelineConfig",
    "NotionContent",
    "fetch_and_render",
    "fetch_content",
    "render_content",
]
