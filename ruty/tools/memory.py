"""Memory tools for searching and adding to Supermemory"""
from langchain_core.tools import tool
from ..memory import search_supermemory, add_memory_to_supermemory


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
    results = search_supermemory(query, limit=5)
    
    if not results:
        return "No relevant memories found for this query."
    
    # Extract content from results
    context_parts = []
    for doc in results:
        chunks = doc.get("chunks", [])
        for chunk in chunks:
            content = chunk.get("content", "")
            if content:
                context_parts.append(content)
    
    if not context_parts:
        return "No relevant memories found for this query."
    
    return "\n\n---\n\n".join(context_parts)


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
