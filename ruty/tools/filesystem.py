"""Filesystem tools for syncing and uploading files"""
from pathlib import Path
from langchain_core.tools import tool
from ..memory import (
    sync_directory_to_supermemory,
    upload_file_to_supermemory,
    add_memory_to_supermemory,
    read_directory_context,
    BINARY_EXTENSIONS,
) 

@tool
def sync_folder(path: str) -> str:
    """Upload all files from a folder to your knowledge base.
    
    This syncs an entire directory, uploading text files and documents
    to your memory. Already-synced files are skipped.
    
    Args:
        path: Path to the folder to sync (use absolute path or ~ for home)
    
    Returns:
        Summary of files synced and skipped
    """
    directory = Path(path).expanduser().resolve()
    
    if not directory.exists():
        return f"✗ Path not found: {directory}"
    
    if not directory.is_dir():
        return f"✗ Not a directory: {directory}"
    
    result = sync_directory_to_supermemory(directory)
    
    if "error" in result:
        return f"✗ {result['error']}"
    
    return f"✓ Synced {result['synced']} files ({result['skipped']} already present)"


@tool
def upload_file(path: str) -> str:
    """Upload a single file to your knowledge base.
    
    Use this to add a specific document or file to your memory.
    
    Args:
        path: Path to the file to upload
    
    Returns:
        Confirmation message
    """
    file_path = Path(path).expanduser().resolve()
    
    if not file_path.exists():
        return f"✗ File not found: {file_path}"
    
    if not file_path.is_file():
        return f"✗ Not a file: {file_path}"
    
    custom_id = f"file:{file_path.name}"
    
    try:
        if file_path.suffix.lower() in BINARY_EXTENSIONS:
            result = upload_file_to_supermemory(file_path, custom_id=custom_id)
        else:
            content = file_path.read_text(encoding="utf-8")
            result = add_memory_to_supermemory(content, custom_id=custom_id, title=file_path.name)
        
        if result:
            return f"✓ Uploaded: {file_path.name}"
        else:
            return f"✗ Failed to upload: {file_path.name}"
    except Exception as e:
        return f"✗ Error: {e}"


@tool
def load_local_context(path: str) -> str:
    """Load local files into context for the current conversation only.
    
    This reads files from a directory or a single file and makes them
    available as context. Files are NOT uploaded to your knowledge base.
    Use this for temporary reference.
    
    Args:
        path: Path to file or directory to load
    
    Returns:
        The content of the files for context
    """
    target = Path(path).expanduser().resolve()
    
    if not target.exists():
        return f"✗ Path not found: {target}"
    
    if target.is_file():
        try:
            content = target.read_text(encoding="utf-8")
            return f"### {target.name}\n```\n{content[:5000]}\n```"
        except Exception as e:
            return f"✗ Error reading file: {e}"
    else:
        return read_directory_context(target)
