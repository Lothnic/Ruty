"""Interactive CLI for Ruty"""
import os
import uuid
from pathlib import Path
from datetime import datetime
from langchain_core.messages import HumanMessage

from .agent import create_agent
from .memory import read_directory_context, add_memory_to_supermemory
from .extraction import extract_semantic_memories


def save_session_history(messages, session_id):
    """Save extracted insights to Supermemory (if not already saved by agent)"""
    # Check if agent already saved memories during the session
    from langchain_core.messages import AIMessage, ToolMessage
    
    # Look for add_memory tool calls
    already_saved = any(
        hasattr(msg, 'tool_calls') and msg.tool_calls and 
        any(tc.get('name') == 'add_memory' for tc in msg.tool_calls)
        for msg in messages if isinstance(msg, AIMessage)
    )
    
    if already_saved:
        print(f"\n✓ Memories already saved during session (skipping extraction)")
        return
    
    print(f"\n🧠 Analyzing session for long-term memories...")
    
    # Extract semantic memories
    extracted_content = extract_semantic_memories(messages)
    
    if not extracted_content:
        print("✓ No significant memories to save.")
        return
        
    print(f"📝 Extracted insights:\n{extracted_content}\n")
    
    # Generate title
    timestamp = datetime.now().strftime('%Y-%m-%d %H:%M')
    title = f"Memory Update: {timestamp}"
    
    # Save the *extracted* content, not the raw log
    final_content = f"Session ID: {session_id}\nDate: {timestamp}\n\n{extracted_content}"
    
    result = add_memory_to_supermemory(final_content, custom_id=f"mem_{session_id}", title=title)
    
    if result:
        print(f"✓ Saved insights to Supermemory")
    else:
        print(f"✗ Failed to save insights")


def interactive_chat():
    """Run interactive chat with the LangGraph agent"""
    
    # Use in-memory storage (no persistence)
    agent = create_agent()  # Uses default MemorySaver
    
    # Create session ID for this session
    session_id = f"session_{datetime.now().strftime('%Y%m%d_%H%M%S')}_{uuid.uuid4().hex[:8]}"
    config = {"configurable": {"thread_id": session_id}}
    
    # Local context state
    local_context = ""
    context_path = None
    
    print("\n🧠 Ruty - LangGraph Agent")
    print("=" * 40)
    print("Commands:")
    print("  /context <path>  - Load files temporarily")
    print("  /context clear   - Clear local context")
    print("  /clear           - Clear screen")
    print("  /quit            - Exit")
    print("=" * 40)
    print()
    print("💡 Try: 'Search my memory for...' or 'Upload ~/notes.txt'")
    print()
    
    try:
        while True:
            try:
                # Show context indicator in prompt
                if context_path:
                    prompt = f"You [{context_path.name}]: "
                else:
                    prompt = "You: "
                user_input = input(prompt).strip()
            except (KeyboardInterrupt, EOFError):
                print("\nGoodbye!")
                break
            
            if not user_input:
                continue
            
            # Handle commands
            if user_input.startswith("/"):
                parts = user_input.split(maxsplit=1)
                cmd = parts[0].lower()
                arg = parts[1] if len(parts) > 1 else ""
                
                if cmd in ["/quit", "/exit", "/q"]:
                    print("Goodbye!")
                    break
                elif cmd == "/clear":
                    os.system('clear' if os.name != 'nt' else 'cls')
                    continue
                elif cmd == "/context":
                    if not arg or arg.lower() == "clear":
                        local_context = ""
                        context_path = None
                        print("✓ Local context cleared")
                    else:
                        path = Path(arg).expanduser().resolve()
                        if path.exists():
                            try:
                                if path.is_file():
                                    content = path.read_text(encoding="utf-8")
                                    local_context = f"### {path.name}\n```\n{content[:5000]}\n```"
                                    context_path = path
                                    print(f"✓ Loaded: {path.name}")
                                else:
                                    local_context = read_directory_context(path)
                                    context_path = path
                                    print(f"✓ Loaded files from: {path.name}")
                            except Exception as e:
                                print(f"Error reading: {e}")
                        else:
                            print(f"Path not found: {path}")
                    continue
                else:
                    print(f"Unknown command: {cmd}")
                    continue
            
            # Process with agent
            print()
            try:
                # Stream events to show tool usage
                final_response = None
                
                # Build input state with local context
                input_state = {"messages": [HumanMessage(content=user_input)]}
                if local_context:
                    input_state["local_context"] = local_context
                
                for event in agent.stream(
                    input_state,
                    config=config,
                    stream_mode="values"
                ):
                    if "messages" in event:
                        last_msg = event["messages"][-1]
                        
                        # Show tool calls
                        if hasattr(last_msg, "tool_calls") and last_msg.tool_calls:
                            for tc in last_msg.tool_calls:
                                print(f"  🔧 Using: {tc['name']}")
                        
                        # Capture final response
                        if hasattr(last_msg, "content") and last_msg.content:
                            if not hasattr(last_msg, "tool_calls") or not last_msg.tool_calls:
                                final_response = last_msg.content
                
                if final_response:
                    print(f"Ruty: {final_response}")
                
            except Exception as e:
                print(f"Error: {e}")
            
            print()
            
    except Exception as e:
        print(f"Error: {e}")
    finally:
        # On exit (Ctrl+C or /quit), try to save important insights
        try:
            state = agent.get_state(config)
            if state and state.values and "messages" in state.values:
                messages = state.values["messages"]
                save_session_history(messages, session_id)
        except Exception as e:
            print(f"Error saving history: {e}")


def main():
    """Entry point"""
    interactive_chat()


if __name__ == "__main__":
    main()

