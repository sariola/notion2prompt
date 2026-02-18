"""Type stubs for notion2prompt."""

from typing import Optional

class PipelineConfig:
    """Configuration for the notion2prompt pipeline."""

    def __init__(
        self,
        notion_id: str,
        api_key: str,
        depth: int = 5,
        limit: int = 1000,
        template: str = "claude-xml",
        always_fetch_databases: bool = False,
        include_properties: bool = False,
        instruction: Optional[str] = None,
        no_cache: bool = False,
        cache_ttl: int = 300,
        concurrency: Optional[int] = None,
    ) -> None: ...

    @property
    def depth(self) -> int: ...
    @property
    def limit(self) -> int: ...
    @property
    def template(self) -> str: ...
    @property
    def always_fetch_databases(self) -> bool: ...
    @property
    def include_properties(self) -> bool: ...

class NotionContent:
    """Fetched Notion content — a page, database, or block."""

    @property
    def content_type(self) -> str:
        """The type of the content: "page", "database", or "block"."""
        ...
    @property
    def title(self) -> str:
        """The display title of the content."""
        ...
    def to_json(self) -> str:
        """Serialize the content to a JSON string."""
        ...

async def fetch_and_render(
    notion_id: str,
    api_key: Optional[str] = None,
    depth: int = 5,
    limit: int = 1000,
    template: str = "claude-xml",
    always_fetch_databases: bool = False,
    include_properties: bool = False,
    instruction: Optional[str] = None,
    no_cache: bool = False,
    cache_ttl: int = 300,
    concurrency: Optional[int] = None,
) -> str:
    """Fetch a Notion page/database and render it to a prompt string.

    Args:
        notion_id: Notion page/database URL or 32-char hex ID
        api_key: Notion API key (reads NOTION_API_KEY env var if None)
        depth: Maximum recursion depth (default 5)
        limit: Maximum items to fetch (default 1000)
        template: Template name (default "claude-xml")
        always_fetch_databases: Always fetch child databases regardless of depth
        include_properties: Include properties section in output
        instruction: Custom instruction text for the prompt
        no_cache: Disable response caching
        cache_ttl: Cache TTL in seconds (default 300)
        concurrency: Number of concurrent API workers

    Returns:
        The rendered prompt string.
    """
    ...

async def fetch_content(
    notion_id: str,
    api_key: Optional[str] = None,
    depth: int = 5,
    limit: int = 1000,
    always_fetch_databases: bool = False,
    no_cache: bool = False,
    cache_ttl: int = 300,
    concurrency: Optional[int] = None,
) -> NotionContent:
    """Fetch Notion content without rendering.

    Returns a NotionContent handle that can be passed to render_content().
    """
    ...

def render_content(
    content: NotionContent,
    template: str = "claude-xml",
    include_properties: bool = False,
    instruction: Optional[str] = None,
) -> str:
    """Render previously fetched content to a prompt string."""
    ...
