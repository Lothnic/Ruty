"""Agent state schema for LangGraph"""
from typing import TypedDict, Annotated
from langgraph.graph.message import add_messages


class AgentState(TypedDict):
    """State passed between nodes in the agent graph.
    
    Attributes:
        messages: Conversation history (automatically accumulated)
        local_context: Optional local file context for current session
        session_id: Unique identifier for the conversation session
    """
    messages: Annotated[list, add_messages]
    local_context: str
    session_id: str
