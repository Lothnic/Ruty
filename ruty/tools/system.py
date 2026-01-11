"""System tools for managing documents"""
from langchain_core.tools import tool
from ..memory import list_docs, delete_document as delete_doc_api


@tool
def list_documents() -> str:
    """List all documents in your knowledge base.
    
    Returns a list of all saved documents and memories.
    
    Returns:
        List of document titles/IDs
    """
    docs = list_docs()
    
    if not docs:
        return "No documents found in your knowledge base."
    
    memories = docs.get("memories", [])
    
    if not memories:
        return "No documents found in your knowledge base."
    
    lines = [f"Found {len(memories)} documents:"]
    for doc in memories[:30]:  # Limit to 30 for readability
        title = doc.get("title") or doc.get("customId") or doc.get("id", "Unknown")
        lines.append(f"  • {title}")
    
    if len(memories) > 30:
        lines.append(f"  ... and {len(memories) - 30} more")
    
    return "\n".join(lines)


@tool
def delete_document(doc_id: str) -> str:
    """Delete a document from your knowledge base.
    
    Use this to remove a specific document by its ID.
    
    Args:
        doc_id: ID of the document to delete
    
    Returns:
        Confirmation message
    """
    result = delete_doc_api(doc_id)
    
    if result:
        return f"✓ Deleted document: {doc_id}"
    else:
        return f"✗ Failed to delete document: {doc_id}"
