"""CLI entry point for notion2prompt Python package.

Provides the same interface as the Rust binary, using the compiled
Rust backend for performance.
"""

from __future__ import annotations

import argparse
import asyncio
import sys


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="notion2prompt",
        description="Convert Notion pages and databases into structured prompts for AI models",
    )
    parser.add_argument(
        "notion_input",
        help='Notion page/database URL or ID (e.g., "https://www.notion.so/...")',
    )
    parser.add_argument(
        "-o",
        "--output-file",
        help="Output file for the final prompt",
    )
    parser.add_argument(
        "--template",
        default="claude-xml",
        help="Template name (default: claude-xml)",
    )
    parser.add_argument(
        "--depth",
        type=int,
        default=5,
        help="Maximum recursion depth (default: 5)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=1000,
        help="Maximum items to fetch (default: 1000)",
    )
    parser.add_argument(
        "-p",
        "--pipe",
        action="store_true",
        help="Output prompt to stdout for piping",
    )
    parser.add_argument(
        "--always-fetch-databases",
        action="store_true",
        help="Always fetch child databases regardless of depth",
    )
    parser.add_argument(
        "--include-properties",
        action="store_true",
        help="Include properties section in output",
    )
    parser.add_argument(
        "--instruction",
        help="Custom instruction text for the prompt",
    )
    parser.add_argument(
        "--no-cache",
        action="store_true",
        help="Disable response caching",
    )
    parser.add_argument(
        "--cache-ttl",
        type=int,
        default=300,
        help="Cache TTL in seconds (default: 300)",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        help="Number of concurrent API workers",
    )
    parser.add_argument(
        "--api-key",
        help="Notion API key (default: reads NOTION_API_KEY env var)",
    )
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    from notion2prompt._notion2prompt import fetch_and_render

    try:
        prompt = asyncio.run(
            fetch_and_render(
                notion_id=args.notion_input,
                api_key=args.api_key,
                depth=args.depth,
                limit=args.limit,
                template=args.template,
                always_fetch_databases=args.always_fetch_databases,
                include_properties=args.include_properties,
                instruction=args.instruction,
                no_cache=args.no_cache,
                cache_ttl=args.cache_ttl,
                concurrency=args.concurrency,
            )
        )
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.output_file:
        with open(args.output_file, "w") as f:
            f.write(prompt)
        print(f"Prompt saved to {args.output_file}", file=sys.stderr)
    elif args.pipe:
        print(prompt, end="")
    else:
        print(prompt, end="")


if __name__ == "__main__":
    main()
