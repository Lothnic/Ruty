import os
import requests
from openai import OpenAI
import dotenv
import hashlib
import uuid
from datetime import datetime
from pathlib import Path

dotenv.load_dotenv()

GROQ_API_KEY = os.getenv("GROQ_API_KEY")
SUPERMEMORY_API_KEY = os.getenv("SUPERMEMORY_API_KEY")

DATA_FOLDER = Path(__file__).parent / "data"

TEXT_EXTENSIONS = {'.txt', '.md', '.py', '.json', '.csv', '.html', '.css', '.js'}
BINARY_EXTENSIONS = {'.pdf', '.png', '.jpg', '.jpeg', '.gif', '.webp'}
SUPPORTED_EXTENSIONS = TEXT_EXTENSIONS | BINARY_EXTENSIONS

# CONFIG
model_name = "moonshotai/kimi-k2-instruct-0905"

# Session management
current_session_id = None
conversation_history = []  # Accumulate messages during session

def get_session_id():
    """Get or create a session ID for this chat session"""
    global current_session_id
    if current_session_id is None:
        current_session_id = f"session_{datetime.now().strftime('%Y%m%d_%H%M%S')}_{uuid.uuid4().hex[:8]}"
    return current_session_id

def add_to_history(user_message: str, assistant_response: str):
    """Add an exchange to the conversation history (in-memory)"""
    conversation_history.append({
        "user": user_message,
        "assistant": assistant_response,
        "timestamp": datetime.now().isoformat()
    })

def save_conversation_history():
    """Save the entire conversation history to Supermemory (call on exit)"""
    global conversation_history
    if not conversation_history:
        return None
    
    session_id = get_session_id()
    
    # Build full conversation content
    lines = [f"[Conversation Session - {session_id}]\n"]
    for i, msg in enumerate(conversation_history, 1):
        lines.append(f"--- Exchange {i} ({msg['timestamp']}) ---")
        lines.append(f"User: {msg['user']}")
        lines.append(f"Assistant: {msg['assistant']}\n")
    
    content = "\n".join(lines)
    
    # Create a summary title from first message
    first_msg = conversation_history[0]['user'][:50]
    title = f"Chat: {first_msg}... ({len(conversation_history)} exchanges)"
    
    result = add_memory(content, custom_id=session_id, title=title)
    
    if result:
        print(f"✓ Saved {len(conversation_history)} exchanges to memory")
        conversation_history = []  # Clear after saving
    
    return result

def get_headers():
    """Get common headers for Supermemory API"""
    return {
        "Authorization": f"Bearer {SUPERMEMORY_API_KEY}",
        "Content-Type": "application/json"
    } 

# 2. List all memories from Supermemory
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
            print(f"Error listing memories: {response.status_code} - {response.text}")
            break
        
        data = response.json()
        memories = data.get("documents", [])
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
        print(f"Error listing documents: {response.status_code} - {response.text}")
        return None
    return response.json()

def delete_document(doc_id: str):
    """Delete a document from Supermemory"""
    response = requests.delete(
        f"https://api.supermemory.ai/v3/documents/{doc_id}",
        headers={"Authorization": f"Bearer {SUPERMEMORY_API_KEY}"}
    )
    if not response.ok:
        print(f"Error deleting document: {response.status_code} - {response.text}")
        return None
    return response.json()

# 3. Add memory to Supermemory with custom ID
def add_memory(content: str, custom_id: str = None, title: str = None):
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
        print(f"Error adding memory: {response.status_code} - {response.text}")
        return None
    return response.json()


# 4. Upload a file (PDF, images) to Supermemory
def upload_file(file_path: Path, custom_id: str = None):
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
        print(f"Error uploading file: {response.status_code} - {response.text}")
        return None
    return response.json()


# 5. Sync any directory to Supermemory
def sync_directory(directory: Path, recursive: bool = True):
    """Sync all files in a directory to Supermemory"""
    if not directory.exists():
        print(f"Directory not found: {directory}")
        return
    
    if not directory.is_dir():
        print(f"Not a directory: {directory}")
        return
    
    # Get existing memories
    print("Fetching existing memories...")
    existing_memories = list_memories()
    existing_ids = {m.get("customId") for m in existing_memories if m.get("customId")}
    print(f"Found {len(existing_memories)} existing memories")
    
    # Scan files
    files_synced = 0
    files_skipped = 0
    
    # Use rglob for recursive, glob for non-recursive
    pattern = "**/*" if recursive else "*"
    
    for file_path in directory.glob(pattern):
        if not file_path.is_file():
            continue
        
        # Check if file type is supported
        if file_path.suffix.lower() not in SUPPORTED_EXTENSIONS:
            continue
        
        # Use relative path as custom ID for deduplication
        rel_path = file_path.relative_to(directory)
        custom_id = f"dir:{directory.name}/{rel_path}"
        
        if custom_id in existing_ids:
            print(f"Already synced: {rel_path}")
            files_skipped += 1
            continue
        
        # Upload file based on type
        try:
            print(f"Uploading: {rel_path}...")
            
            if file_path.suffix.lower() in BINARY_EXTENSIONS:
                result = upload_file(file_path, custom_id=custom_id)
            else:
                content = file_path.read_text(encoding="utf-8")
                result = add_memory(content, custom_id=custom_id, title=str(rel_path))
            
            if result:
                print(f"  ✓ Uploaded successfully")
                files_synced += 1
            else:
                print(f"  ✗ Upload failed")
        except Exception as e:
            print(f"  ✗ Error: {e}")
    
    print(f"\nSync complete: {files_synced} uploaded, {files_skipped} already present")


# Keep old sync_folder for backward compatibility
def sync_folder():
    """Sync the default data folder"""
    sync_directory(DATA_FOLDER)


# 6. Search for context before each query
def get_context(query: str):
    """Retrieve relevant context"""
    response = requests.post(
        "https://api.supermemory.ai/v3/search",
        headers=get_headers(),
        json={"q": query, "limit": 5}
    )
    if not response.ok:
        print(f"Error searching: {response.status_code} - {response.text}")
        return []
    data = response.json()
    return data.get("results", [])


# 7. Chat with LLM + context
def chat(query: str, extra_context: str = ""):
    # Get relevant context from Supermemory
    context_docs = get_context(query)
    if context_docs:
        # Extract content from chunks
        context_parts = []
        for doc in context_docs:
            chunks = doc.get("chunks", [])
            for chunk in chunks:
                content = chunk.get("content", "")
                if content:
                    context_parts.append(content)
        supermem_context = "\n\n".join(context_parts) if context_parts else ""
    else:
        supermem_context = ""
    
    # Combine with extra local context
    all_context = []
    if extra_context:
        all_context.append(f"[Local Files]\n{extra_context}")
    if supermem_context:
        all_context.append(f"[Memory]\n{supermem_context}")
    
    context = "\n\n---\n\n".join(all_context) if all_context else "No context available."
    
    # Call LLM
    client = OpenAI(
        api_key=GROQ_API_KEY,
        base_url="https://api.groq.com/openai/v1"
    )
    
    response = client.chat.completions.create(
        model=model_name,
        messages=[
            {"role": "system", "content": "You are a helpful assistant. Use the provided context to answer questions. If the context doesn't contain relevant information, say so."},
            {"role": "user", "content": f"Context:\n{context}\n\nQuestion: {query}"}
        ],
        max_tokens=1000
    )
    return response.choices[0].message.content


# 8. Read files from a directory into context
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
        except Exception as e:
            continue
    
    return "\n\n".join(context_parts) if context_parts else "No readable files found."


def interactive_chat():
    """Run interactive chat in terminal"""
    # Hold local context for temporary file context
    local_context = ""
    context_path = None
    
    print("\n🧠 SuperMemory Chat")
    print("=" * 40)
    print("Commands:")
    print("  /context <path> - Load files temporarily")
    print("  /context clear  - Clear local context")
    print("  /sync <path>    - Upload to Supermemory")
    print("  /upload <file>  - Upload a file")
    print("  /list           - List documents")
    print("  /clear          - Clear screen")
    print("  /quit           - Exit")
    print("=" * 40)
    print()
    
    while True:
        try:
            prompt = f"You [{context_path.name if context_path else 'no context'}]: " if context_path else "You: "
            user_input = input(prompt).strip()
        except (KeyboardInterrupt, EOFError):
            save_conversation_history()  # Save on Ctrl+C
            print("\nGoodbye!")
            break
        
        if not user_input:
            continue
        
        if user_input.startswith("/"):
            parts = user_input.split(maxsplit=1)
            cmd = parts[0].lower()
            arg = parts[1] if len(parts) > 1 else ""
            
            if cmd == "/quit" or cmd == "/exit":
                save_conversation_history()  # Save on exit
                print("Goodbye!")
                break
            elif cmd == "/clear":
                os.system('clear' if os.name != 'nt' else 'cls')
            elif cmd == "/context":
                if not arg or arg.lower() == "clear":
                    local_context = ""
                    context_path = None
                    print("✓ Local context cleared")
                else:
                    path = Path(arg).expanduser().resolve()
                    if path.exists():
                        if path.is_file():
                            try:
                                local_context = f"### {path.name}\n```\n{path.read_text()[:5000]}\n```"
                                context_path = path
                                print(f"✓ Loaded: {path.name}")
                            except Exception as e:
                                print(f"Error reading file: {e}")
                        else:
                            local_context = read_directory_context(path)
                            context_path = path
                            print(f"✓ Loaded files from: {path.name}")
                    else:
                        print(f"Path not found: {path}")
            elif cmd == "/sync":
                if not arg:
                    print("Usage: /sync <directory_path>")
                else:
                    path = Path(arg).expanduser().resolve()
                    sync_directory(path)
            elif cmd == "/upload":
                if not arg:
                    print("Usage: /upload <file_path>")
                else:
                    path = Path(arg).expanduser().resolve()
                    if path.exists():
                        custom_id = f"file:{path.name}"
                        result = upload_file(path, custom_id=custom_id) if path.suffix.lower() in BINARY_EXTENSIONS else add_memory(path.read_text(), custom_id=custom_id, title=path.name)
                        print("✓ Uploaded!" if result else "✗ Failed")
                    else:
                        print(f"File not found: {path}")
            elif cmd == "/list":
                docs = list_docs()
                if docs:
                    memories = docs.get("memories", [])
                    print(f"\nFound {len(memories)} documents:")
                    for doc in memories[:20]:
                        title = doc.get("title") or doc.get("customId") or doc.get("id", "Unknown")
                        print(f"  • {title}")
                    if len(memories) > 20:
                        print(f"  ... and {len(memories) - 20} more")
                else:
                    print("No documents found")
            elif cmd == "/help":
                print("Commands: /context, /sync, /upload, /list, /clear, /quit")
            else:
                print(f"Unknown command: {cmd}")
            continue
        
        print("\nThinking...")
        try:
            response = chat(user_input, extra_context=local_context)
            print(f"\nAssistant: {response}\n")
            
            # Add to history (will be saved on exit)
            add_to_history(user_input, response)
        except Exception as e:
            print(f"\nError: {e}\n")


# Usage
if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        cmd = sys.argv[1]
        
        if cmd == "sync":
            if len(sys.argv) > 2:
                sync_directory(Path(sys.argv[2]).expanduser().resolve())
            else:
                sync_folder()
        
        elif cmd == "upload":
            if len(sys.argv) < 3:
                print("Usage: python main.py upload <file_path>")
                sys.exit(1)
            file_path = Path(sys.argv[2]).expanduser().resolve()
            if not file_path.exists():
                print(f"File not found: {file_path}")
                sys.exit(1)
            custom_id = f"file:{file_path.name}"
            if file_path.suffix.lower() in BINARY_EXTENSIONS:
                result = upload_file(file_path, custom_id=custom_id)
            else:
                result = add_memory(file_path.read_text(), custom_id=custom_id, title=file_path.name)
            print("✓ Uploaded!" if result else "✗ Failed")
        
        elif cmd == "delete":
            if len(sys.argv) < 3:
                print("Usage: python main.py delete <doc_id>")
                sys.exit(1)
            result = delete_document(sys.argv[2])
            print("✓ Deleted!" if result else "✗ Failed")
        
        elif cmd == "list":
            docs = list_docs()
            if docs:
                memories = docs.get("memories", [])
                print(f"Found {len(memories)} documents:")
                for doc in memories:
                    title = doc.get("title") or doc.get("customId") or doc.get("id", "Unknown")
                    print(f"  • {title}")
            else:
                print("No documents found")
        
        elif cmd == "chat":
            if len(sys.argv) > 2:
                query = " ".join(sys.argv[2:])
                print(chat(query))
            else:
                interactive_chat()
        
        else:
            print(f"Unknown command: {cmd}")
            print("Usage: python main.py [sync|upload|delete|list|chat]")
    
    else:
        interactive_chat()