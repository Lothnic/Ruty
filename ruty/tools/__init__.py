# Ruty Tools
from .memory import search_memory, add_memory
from .filesystem import sync_folder, upload_file, load_local_context
from .system import list_documents, delete_document, open_url, run_shell, get_system_info

ALL_TOOLS = [
    # Memory tools
    search_memory,
    add_memory,
    # File tools
    sync_folder,
    upload_file,
    load_local_context,
    # System tools
    list_documents,
    delete_document,
    open_url,
    run_shell,
    get_system_info,
]

__all__ = ["ALL_TOOLS"] + [t.name for t in ALL_TOOLS]
