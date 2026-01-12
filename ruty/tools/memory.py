"""Memory tools for searching and adding to Supermemory"""
from langchain_core.tools import tool
from ..memory import search_supermemory, add_memory_to_supermemory, list_memories


@tool
def search_memory(query: str) -> str:
    """Search your personal knowledge base for relevant information.
    
    Use this tool to find information from your saved documents, notes, 
    and previous conversations. Always search before answering questions
    that might relate to stored knowledge.
    
    Args:
        query: What to search for in your memories (be specific)
    
    Returns:
        Relevant content from your knowledge base, or a message if nothing found
    """
    context_parts = []
    query_lower = query.lower()
    
    # 1. Search documents (PDFs) via search API
    results = search_supermemory(query, limit=5)
    for result in results:
        chunks = result.get("chunks", [])
        for chunk in chunks:
            content = chunk.get("content", "")
            if content:
                title = result.get("title", "")
                context_parts.append(f"**{title}**:\n{content}" if title else content)
    
    # 2. Search text memories by listing and matching (since search API doesn't index them)
    memories = list_memories()
    for mem in memories:
        if mem.get("type") == "text":
            title = mem.get("title", "")
            summary = mem.get("summary", "")
            # Simple keyword matching
            if any(word in (title + " " + summary).lower() for word in query_lower.split()):
                if summary:
                    context_parts.append(f"**{title}**:\n{summary}" if title else summary)
    
    if not context_parts:
        return "No relevant memories found for this query."
    
    # Deduplicate and limit
    seen = set()
    unique_parts = []
    for part in context_parts:
        if part not in seen:
            seen.add(part)
            unique_parts.append(part)
    
    return "\n\n---\n\n".join(unique_parts[:5])


@tool
def add_memory(content: str, title: str = "") -> str:
    """Save new information to your knowledge base.
    
    Use this tool to remember important information, notes, or insights
    that you want to recall later.
    
    Args:
        content: The content to remember (be detailed)
        title: Optional short title for the memory
    
    Returns:
        Confirmation message
    """
    result = add_memory_to_supermemory(
        content=content,
        title=title if title else None
    )
    
    if result:
        return f"✓ Memory saved: {title if title else content[:50]}..."
    else:
        return "✗ Failed to save memory"
