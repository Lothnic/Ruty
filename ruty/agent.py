"""LangGraph ReAct Agent for Ruty"""
import os
from dotenv import load_dotenv
from langgraph.graph import StateGraph, START, END
from langgraph.prebuilt import ToolNode, tools_condition
from langgraph.checkpoint.memory import MemorySaver
from langchain_openai import ChatOpenAI

from .state import AgentState
from .tools import ALL_TOOLS

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

Guidelines:
1. When the user asks a question, FIRST search your memory to find relevant context
2. Use add_memory ONLY if user explicitly asks you to remember something
3. Preferences and facts are auto-saved on exit - you don't need to save them manually
4. Be conversational and helpful - you're a personal assistant
5. If you don't find relevant information in memory, say so and offer to help anyway
6. Keep responses concise but informative
"""


def create_agent(model_name: str = None, checkpointer = None):
    """Build and return the LangGraph agent.
    
    Args:
        model_name: Override the default model
        checkpointer: Optional persistence layer (defaults to in-memory)
        
    Returns:
        Compiled LangGraph agent with checkpointing
    """
    # Default to the model from environment or fallback
    if model_name is None:
        model_name = os.getenv("RUTY_MODEL", "moonshotai/kimi-k2-instruct-0905")
    
    # Initialize LLM with tool binding
    # Using Groq via OpenAI-compatible API
    llm = ChatOpenAI(
        model=model_name,
        api_key=os.getenv("GROQ_API_KEY"),
        base_url="https://api.groq.com/openai/v1",
        temperature=0.7,
        max_tokens=2000,
    ).bind_tools(ALL_TOOLS)
    
    # Define the assistant node (reasoning)
    def assistant(state: AgentState):
        """The reasoning node - processes messages and decides actions"""
        messages = [{"role": "system", "content": SYSTEM_PROMPT}]
        
        # Add local context if available
        if state.get("local_context"):
            messages.append({
                "role": "system", 
                "content": f"[Local Context]\n{state['local_context']}"
            })
        
        # Add conversation history
        messages.extend(state["messages"])
        
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
        checkpointer = MemorySaver()
        
    return graph.compile(checkpointer=checkpointer)


# Singleton agent instance
_agent = None


def get_agent():
    """Get or create the singleton agent instance"""
    global _agent
    if _agent is None:
        _agent = create_agent()
    return _agent
