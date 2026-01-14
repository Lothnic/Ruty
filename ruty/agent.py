"""LangGraph ReAct Agent for Ruty.

A personal AI assistant with memory, supporting multiple LLM providers
and persistent conversation history.
"""
import os
import sqlite3
from pathlib import Path
from dotenv import load_dotenv
from langgraph.graph import StateGraph, START, END
from langgraph.prebuilt import ToolNode, tools_condition
from langgraph.checkpoint.memory import MemorySaver
from langgraph.checkpoint.sqlite import SqliteSaver
from langchain_openai import ChatOpenAI

from .state import AgentState
from .tools import ALL_TOOLS
from .providers import get_config, PROVIDERS

load_dotenv()

# System prompt for the agent
SYSTEM_PROMPT = """You are Ruty, a personal AI assistant with access to a knowledge base.

You have the following capabilities through your tools:
- **search_memory**: Search your personal knowledge base for relevant information
- **add_memory**: Save ONLY when user EXPLICITLY asks (e.g. "remember this", "save to memory"). DO NOT auto-save preferences.
- **sync_folder**: Upload all files from a folder to your knowledge base
- **upload_file**: Upload a single file to your knowledge base
- **load_local_context**: Temporarily load local files for the current conversation
- **list_documents**: List all documents in your knowledge base
- **delete_document**: Delete a document from your knowledge base
- **open_url**: Open a URL in the user's default browser
- **run_shell**: Execute a shell command (be careful!)

Guidelines:
1. When the user asks a question, FIRST search your memory to find relevant context
2. Use add_memory ONLY if user explicitly asks you to remember something
3. Preferences and facts are auto-saved on exit - you don't need to save them manually
4. Be conversational and helpful - you're a personal assistant
5. If you don't find relevant information in memory, say so and offer to help anyway
6. Keep responses concise but informative
7. For URLs or web content, use open_url to help the user
8. For system tasks, use run_shell cautiously and explain what you're doing
"""

# SQLite database for conversation persistence
DB_PATH = Path.home() / ".config" / "ruty" / "conversations.db"


def get_checkpointer(persistent: bool = True):
    """Get a checkpointer for conversation persistence.
    
    Args:
        persistent: If True, use SQLite. If False, use in-memory.
    
    Returns:
        A LangGraph checkpointer instance.
    """
    if persistent:
        DB_PATH.parent.mkdir(parents=True, exist_ok=True)
        # Use sqlite3.connect with check_same_thread=False for thread safety
        conn = sqlite3.connect(str(DB_PATH), check_same_thread=False)
        return SqliteSaver(conn)
    return MemorySaver()


def create_llm(config=None, api_key_override: str = None):
    """Create an LLM instance based on configuration.
    
    Args:
        config: RutyConfig instance (uses global if None)
        api_key_override: Override API key for this request
    
    Returns:
        A ChatOpenAI instance configured for the selected provider.
    """
    if config is None:
        config = get_config()
    
    provider = config.current_provider
    model = config.current_model
    api_key = api_key_override or config.current_api_key
    
    # For providers that don't require keys (Ollama), use dummy
    if not provider.requires_key:
        api_key = "not-needed"
    
    # Preventing crash if key is missing (allow initialization, fail on call)
    if not api_key:
        api_key = "missing-key-placeholder"
    
    return ChatOpenAI(
        model=model,
        api_key=api_key,
        base_url=provider.base_url,
        temperature=0.7,
        max_tokens=2000,
    )


def create_agent(checkpointer=None, config=None):
    """Build and return the LangGraph agent.
    
    Args:
        checkpointer: Optional persistence layer (defaults to SQLite)
        config: Optional RutyConfig override
        
    Returns:
        Compiled LangGraph agent with checkpointing
    """
    if config is None:
        config = get_config()
    
    # Define the assistant node (reasoning)
    def assistant(state: AgentState):
        """The reasoning node - processes messages and decides actions"""
        from langchain_core.messages import HumanMessage, AIMessage, ToolMessage
        from .config import api_key_context
        
        # Get API key from context or config
        ctx_keys = api_key_context.get()
        provider_id = config.provider
        api_key = ctx_keys.get(provider_id) or config.current_api_key
        
        # Create LLM with current config
        llm = create_llm(config, api_key_override=api_key).bind_tools(ALL_TOOLS)
        
        messages = [{"role": "system", "content": SYSTEM_PROMPT}]
        
        # Add local context if available
        if state.get("local_context"):
            messages.append({
                "role": "system", 
                "content": f"[Local Context]\n{state['local_context']}"
            })
        
        # Aggressive trimming: keep only last 6 messages to stay under context limits
        MAX_MESSAGES = 6
        conversation = state["messages"]
        if len(conversation) > MAX_MESSAGES:
            conversation = conversation[-MAX_MESSAGES:]
        
        messages.extend(conversation)
        
        # Get LLM response
        response = llm.invoke(messages)
        
        return {"messages": [response]}
    
    # Build the graph
    graph = StateGraph(AgentState)
    
    # Add nodes
    graph.add_node("assistant", assistant)
    graph.add_node("tools", ToolNode(ALL_TOOLS))
    
    # Define edges (ReAct loop)
    graph.add_edge(START, "assistant")
    graph.add_conditional_edges(
        "assistant",
        tools_condition,  # Routes to "tools" if tool call, else END
    )
    graph.add_edge("tools", "assistant")  # Loop back after tool execution
    
    # Compile with checkpointer
    if checkpointer is None:
        checkpointer = get_checkpointer(persistent=True)
        
    return graph.compile(checkpointer=checkpointer)


# Singleton agent instance
_agent = None


def get_agent(force_new: bool = False):
    """Get or create the singleton agent instance.
    
    Args:
        force_new: Force creation of a new agent (e.g., after config change)
    
    Returns:
        Compiled LangGraph agent
    """
    global _agent
    if _agent is None or force_new:
        _agent = create_agent()
    return _agent


def reset_agent():
    """Reset the agent (useful after config changes)."""
    global _agent
    _agent = None
