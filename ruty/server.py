"""FastAPI backend server for Tauri frontend.

Provides:
- Chat endpoints (REST and WebSocket)
- Provider configuration management
- Session management
- Local context loading
"""
import os
import uuid
from datetime import datetime
from typing import Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn

from langchain_core.messages import HumanMessage, AIMessage
from .agent import create_agent, get_agent, reset_agent
from .memory import read_directory_context
from .config import api_key_context
from .providers import (
    get_config, update_config, list_providers, 
    PROVIDERS, RutyConfig
)


# Session storage
sessions: dict = {}


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifecycle - startup and shutdown."""
    print("🧠 Ruty AI backend starting...")
    yield
    print("👋 Ruty AI backend shutting down...")


app = FastAPI(
    title="Ruty AI Backend",
    description="Personal AI assistant with memory",
    version="0.3.0",
    lifespan=lifespan,
)

# CORS for Tauri frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ============== Request/Response Models ==============

class ChatRequest(BaseModel):
    """Request model for chat endpoint"""
    message: str
    session_id: str
    local_context: Optional[str] = None
    api_keys: Optional[dict] = None


class ChatResponse(BaseModel):
    """Response model for chat endpoint"""
    response: str
    tools_used: list[str] = []
    session_id: str


class ContextRequest(BaseModel):
    """Request model for context loading"""
    session_id: str
    path: str


class ProviderUpdateRequest(BaseModel):
    """Request to update provider configuration"""
    provider: Optional[str] = None
    model: Optional[str] = None
    api_key: Optional[str] = None  # Key for the current provider


# ============== Session Management ==============

def get_or_create_session(session_id: str) -> dict:
    """Get existing session or create new one."""
    if session_id not in sessions:
        sessions[session_id] = {
            "agent": create_agent(),
            "config": {"configurable": {"thread_id": session_id}},
            "local_context": "",
            "created_at": datetime.now().isoformat(),
        }
    return sessions[session_id]


# ============== Chat Endpoints ==============

@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """
    Process a chat message and return response.
    This is a synchronous endpoint that returns the full response.
    """
    # Set API key context for this request
    token = api_key_context.set(request.api_keys or {})
    
    try:
        session = get_or_create_session(request.session_id)
        agent = session["agent"]
        config = session["config"]
        
        # Update local context if provided
        if request.local_context:
            session["local_context"] = request.local_context
        
        # Build input state
        input_state = {"messages": [HumanMessage(content=request.message)]}
        if session["local_context"]:
            input_state["local_context"] = session["local_context"]
        
        # Process with agent
        tools_used = []
        final_response = ""
        
        try:
            for event in agent.stream(input_state, config=config, stream_mode="values"):
                if "messages" in event:
                    last_msg = event["messages"][-1]
                    
                    # Track tool calls
                    if hasattr(last_msg, "tool_calls") and last_msg.tool_calls:
                        for tc in last_msg.tool_calls:
                            tools_used.append(tc["name"])
                    
                    # Capture final response (AI message without tool calls)
                    if hasattr(last_msg, "content") and last_msg.content:
                        if not hasattr(last_msg, "tool_calls") or not last_msg.tool_calls:
                            final_response = last_msg.content
        except Exception as e:
            final_response = f"Error: {str(e)}"
        
        return ChatResponse(
            response=final_response,
            tools_used=list(set(tools_used)),  # Deduplicate
            session_id=request.session_id
        )
    finally:
        api_key_context.reset(token)


@app.websocket("/ws/{session_id}")
async def websocket_chat(websocket: WebSocket, session_id: str):
    """
    WebSocket endpoint for streaming chat responses.
    Enables real-time token streaming to the frontend.
    """
    await websocket.accept()
    session = get_or_create_session(session_id)
    
    try:
        while True:
            # Receive message from frontend
            data = await websocket.receive_json()
            message = data.get("message", "")
            local_context = data.get("local_context", "")
            api_keys = data.get("api_keys", {})
            
            # Set context for this iteration
            token = api_key_context.set(api_keys)
            
            try:
                if local_context:
                    session["local_context"] = local_context
                
                # Build input state
                input_state = {"messages": [HumanMessage(content=message)]}
                if session["local_context"]:
                    input_state["local_context"] = session["local_context"]
                
                agent = session["agent"]
                config = session["config"]
                
                # Stream response
                try:
                    for event in agent.stream(input_state, config=config, stream_mode="values"):
                        if "messages" in event:
                            last_msg = event["messages"][-1]
                            
                            # Send tool usage updates
                            if hasattr(last_msg, "tool_calls") and last_msg.tool_calls:
                                for tc in last_msg.tool_calls:
                                    await websocket.send_json({
                                        "type": "tool",
                                        "name": tc["name"]
                                    })
                            
                            # Send final response
                            if hasattr(last_msg, "content") and last_msg.content:
                                if not hasattr(last_msg, "tool_calls") or not last_msg.tool_calls:
                                    await websocket.send_json({
                                        "type": "response",
                                        "content": last_msg.content
                                    })
                    
                    # Signal completion
                    await websocket.send_json({"type": "done"})
                    
                except Exception as e:
                    await websocket.send_json({
                        "type": "error",
                        "message": str(e)
                    })
            finally:
                api_key_context.reset(token)
                
    except WebSocketDisconnect:
        print(f"Session {session_id} disconnected")


# ============== Provider Management ==============

@app.get("/providers")
async def get_providers():
    """List available LLM providers and their models."""
    return {
        "providers": list_providers(),
        "current": {
            "provider": get_config().provider,
            "model": get_config().current_model,
        }
    }


@app.post("/providers/update")
async def update_provider(request: ProviderUpdateRequest):
    """Update the current provider configuration."""
    config = get_config()
    
    updates = {}
    
    if request.provider and request.provider in PROVIDERS:
        updates["provider"] = request.provider
    
    if request.model:
        updates["model"] = request.model
    
    if request.api_key:
        # Store the API key for the target provider
        target_provider = request.provider or config.provider
        new_keys = dict(config.api_keys)
        new_keys[target_provider] = request.api_key
        updates["api_keys"] = new_keys
    
    if updates:
        new_config = update_config(**updates)
        # Reset agent to pick up new config
        reset_agent()
        # Reset all sessions to use new agent
        sessions.clear()
        
        return {
            "success": True,
            "provider": new_config.provider,
            "model": new_config.current_model,
        }
    
    return {"success": False, "error": "No valid updates provided"}


@app.get("/config")
async def get_current_config():
    """Get current configuration (without sensitive keys)."""
    config = get_config()
    return {
        "provider": config.provider,
        "model": config.current_model,
        "theme": config.theme,
        "hotkey": config.hotkey,
        "has_api_key": bool(config.current_api_key),
        "has_supermemory_key": bool(config.get_supermemory_key()),
    }


# ============== Context Management ==============

@app.post("/context/load")
async def load_context(request: ContextRequest):
    """Load local files as context for the session"""
    from pathlib import Path
    
    session = get_or_create_session(request.session_id)
    path = Path(request.path).expanduser().resolve()
    
    if not path.exists():
        return {"success": False, "error": f"Path not found: {path}"}
    
    try:
        if path.is_file():
            content = path.read_text(encoding="utf-8")
            session["local_context"] = f"### {path.name}\n```\n{content[:5000]}\n```"
            return {"success": True, "loaded": path.name, "type": "file"}
        else:
            content = read_directory_context(path)
            session["local_context"] = content
            return {"success": True, "loaded": path.name, "type": "directory"}
    except Exception as e:
        return {"success": False, "error": str(e)}


@app.post("/context/clear")
async def clear_context(session_id: str):
    """Clear local context for a session"""
    session = get_or_create_session(session_id)
    session["local_context"] = ""
    return {"success": True}


# ============== Session Management ==============

@app.get("/sessions")
async def list_sessions_endpoint():
    """List active sessions"""
    return {
        "sessions": [
            {"id": sid, "created_at": s["created_at"]}
            for sid, s in sessions.items()
        ]
    }


@app.delete("/sessions/{session_id}")
async def delete_session(session_id: str):
    """Delete a session"""
    if session_id in sessions:
        del sessions[session_id]
        return {"success": True}
    return {"success": False, "error": "Session not found"}


# ============== Health Check ==============

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    config = get_config()
    return {
        "status": "healthy",
        "provider": config.provider,
        "model": config.current_model,
        "sessions_active": len(sessions),
    }


# ============== Server Runner ==============

def run_server(host: str = "127.0.0.1", port: int = 3847):
    """Run the FastAPI server"""
    print(f"🧠 Ruty backend running at http://{host}:{port}")
    uvicorn.run(app, host=host, port=port, log_level="warning")


if __name__ == "__main__":
    run_server()
