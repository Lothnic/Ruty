"""Supermemory client wrapper for memory operations"""
import os
import requests
from pathlib import Path
from dotenv import load_dotenv

# Ensure .env is loaded before accessing environment variables
load_dotenv()

from .config import api_key_context

def get_supermemory_key():
    return api_key_context.get().get("supermemory") or os.getenv("SUPERMEMORY_API_KEY")

TEXT_EXTENSIONS = {'.txt', '.md', '.py', '.json', '.csv', '.html', '.css', '.js'}
BINARY_EXTENSIONS = {'.pdf', '.png', '.jpg', '.jpeg', '.gif', '.webp'}
SUPPORTED_EXTENSIONS = TEXT_EXTENSIONS | BINARY_EXTENSIONS


def get_headers():
    """Get common headers for Supermemory API"""
    return {
        "Authorization": f"Bearer {get_supermemory_key()}",
        "Content-Type": "application/json"
    }


def list_memories():
    """Get all existing memories from Supermemory"""
    all_memories = []
    page = 1
    
    while True:
        response = requests.post(
            "https://api.supermemory.ai/v3/documents/list",
            headers=get_headers(),
            json={"limit": 200, "page": page}
        )
        if not response.ok:
            break
        
        data = response.json()
        # API returns 'memories' not 'documents'
        memories = data.get("memories", [])
        if not memories:
            break
        
        all_memories.extend(memories)
        page += 1
    
    return all_memories


def list_docs():
    """List all documents from Supermemory"""
    response = requests.post(
        "https://api.supermemory.ai/v3/documents/list",
        headers=get_headers(),
        json={"limit": 200, "page": 1}
    )
    if not response.ok:
        return None
    return response.json()


def delete_document(doc_id: str):
    """Delete a document from Supermemory"""
    response = requests.delete(
        f"https://api.supermemory.ai/v3/documents/{doc_id}",
        headers={"Authorization": f"Bearer {SUPERMEMORY_API_KEY}"}
    )
    if not response.ok:
        return None
    return response.json()


def add_memory_to_supermemory(content: str, custom_id: str = None, title: str = None):
    """Add a memory to Supermemory"""
    payload = {"content": content}
    if custom_id:
        payload["customId"] = custom_id
    if title:
        payload["title"] = title
    
    response = requests.post(
        "https://api.supermemory.ai/v3/memories",
        headers=get_headers(),
        json=payload
    )
    if not response.ok:
        return None
    return response.json()


def upload_file_to_supermemory(file_path: Path, custom_id: str = None):
    """Upload a file directly to Supermemory"""
    with open(file_path, 'rb') as f:
        files = {'file': (file_path.name, f)}
        data = {}
        if custom_id:
            data['customId'] = custom_id
        
        response = requests.post(
            "https://api.supermemory.ai/v3/documents/file",
            headers={"Authorization": f"Bearer {SUPERMEMORY_API_KEY}"},
            files=files,
            data=data
        )
    if not response.ok:
        return None
    return response.json()


def search_supermemory(query: str, limit: int = 5):
    """Search for relevant context in Supermemory using v4 hybrid mode.
    
    Searches both memories and document chunks for best results.
    """
    response = requests.post(
        "https://api.supermemory.ai/v4/search",  # v4 endpoint
        headers=get_headers(),
        json={
            "q": query, 
            "limit": limit,
            "searchMode": "hybrid"  # Search both memories and chunks
        }
    )
    if not response.ok:
        return []
    data = response.json()
    return data.get("results", [])


def sync_directory_to_supermemory(directory: Path, recursive: bool = True):
    """Sync all files in a directory to Supermemory"""
    if not directory.exists() or not directory.is_dir():
        return {"error": f"Invalid directory: {directory}", "synced": 0, "skipped": 0}
    
    # Get existing memories
    existing_memories = list_memories()
    existing_ids = {m.get("customId") for m in existing_memories if m.get("customId")}
    
    files_synced = 0
    files_skipped = 0
    
    pattern = "**/*" if recursive else "*"
    
    for file_path in directory.glob(pattern):
        if not file_path.is_file():
            continue
        
        if file_path.suffix.lower() not in SUPPORTED_EXTENSIONS:
            continue
        
        rel_path = file_path.relative_to(directory)
        custom_id = f"dir:{directory.name}/{rel_path}"
        
        if custom_id in existing_ids:
            files_skipped += 1
            continue
        
        try:
            if file_path.suffix.lower() in BINARY_EXTENSIONS:
                result = upload_file_to_supermemory(file_path, custom_id=custom_id)
            else:
                content = file_path.read_text(encoding="utf-8")
                result = add_memory_to_supermemory(content, custom_id=custom_id, title=str(rel_path))
            
            if result:
                files_synced += 1
        except Exception:
            pass
    
    return {"synced": files_synced, "skipped": files_skipped}


def read_directory_context(directory: Path, max_files: int = 20) -> str:
    """Read text files from a directory into a string for context"""
    if not directory.exists():
        return f"Directory not found: {directory}"
    
    context_parts = []
    file_count = 0
    
    for file_path in directory.rglob("*"):
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in TEXT_EXTENSIONS:
            continue
        if file_count >= max_files:
            context_parts.append(f"\n... (truncated, {max_files} files limit)")
            break
        
        try:
            content = file_path.read_text(encoding="utf-8")
            rel_path = file_path.relative_to(directory)
            context_parts.append(f"### {rel_path}\n```\n{content[:2000]}\n```")
            file_count += 1
        except Exception:
            continue
    
    return "\n\n".join(context_parts) if context_parts else "No readable files found."
