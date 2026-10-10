"""Parser tests for the Python CLI. These do not call the Notion API."""

from notion2prompt.cli import build_parser


def test_parser_defaults():
    args = build_parser().parse_args(["0123456789abcdef0123456789abcdef"])
    assert args.notion_input == "0123456789abcdef0123456789abcdef"
    assert args.template == "claude-xml"
    assert args.depth == 5
    assert args.limit == 1000
    assert args.pipe is False
    assert args.always_fetch_databases is False
    assert args.include_properties is False
    assert args.no_cache is False
    assert args.cache_ttl == 300
    assert args.output_file is None
    assert args.api_key is None


def test_parser_flags():
    args = build_parser().parse_args(
        [
            "https://www.notion.so/example",
            "--template",
            "default",
            "--depth",
            "2",
            "--limit",
            "10",
            "--pipe",
            "--always-fetch-databases",
            "--include-properties",
            "--no-cache",
            "--cache-ttl",
            "15",
            "--concurrency",
            "4",
            "--api-key",
            "secret_test",
            "-o",
            "out.md",
            "--instruction",
            "be brief",
        ]
    )
    assert args.template == "default"
    assert args.depth == 2
    assert args.limit == 10
    assert args.pipe is True
    assert args.always_fetch_databases is True
    assert args.include_properties is True
    assert args.no_cache is True
    assert args.cache_ttl == 15
    assert args.concurrency == 4
    assert args.api_key == "secret_test"
    assert args.output_file == "out.md"
    assert args.instruction == "be brief"
