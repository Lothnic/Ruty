"""Module for extracting semantic memories from conversation history"""
from langchain_core.messages import HumanMessage, AIMessage, SystemMessage
from langchain_openai import ChatOpenAI
import os

# Prompt for extracting memories
EXTRACTION_PROMPT = """You are an expert memory assistant. Your goal is to extract valuable information from a conversation log.
Analyze the following conversation and extract:
1. User preferences (what they like/dislike, their tech stack, goals)
2. Facts established (e.g., project names, decisions made)
3. Action items or future intentions
4. Context that would be useful for a future session

Ignore pleasantries, system errors, or transient info.
Format the output as a bulleted list of standalone facts that can be stored in a knowledge base.
If nothing is worth remembering, return "NO_MEMORY".

Conversation:
{conversation}
"""

def extract_semantic_memories(messages, model_name=None):
    """
    Extract meaningful insights from a conversation history suitable for long-term memory.
    
    Args:
        messages: List of LangChain message objects
        model_name: Optional model override
        
    Returns:
        str: Extracted memories as text, or None if nothing to save
    """
    # Filter for meaningful content
    exchanges = [m for m in messages if isinstance(m, (HumanMessage, AIMessage))]
    if len(exchanges) < 2:
        return None
        
    # Format conversation for LLM
    conversation_text = ""
    for msg in exchanges:
        role = "User" if isinstance(msg, HumanMessage) else "Assistant"
        content = msg.content
        if not content: continue
        conversation_text += f"{role}: {content}\n"
        
    if not conversation_text:
        return None

    # Default to the environment model
    if model_name is None:
        model_name = os.getenv("RUTY_MODEL", "moonshotai/kimi-k2-instruct-0905")

    # Use a separate LLM instance for extraction (can use a cheaper/faster model if desired)
    llm = ChatOpenAI(
        model=model_name,
        api_key=os.getenv("GROQ_API_KEY"),
        base_url="https://api.groq.com/openai/v1",
        temperature=0.3, # Lower temp for factual extraction
    )
    
    try:
        # Run extraction
        prompt = EXTRACTION_PROMPT.format(conversation=conversation_text)
        response = llm.invoke([HumanMessage(content=prompt)])
        content = response.content.strip()
        
        if content == "NO_MEMORY":
            return None
            
        return content
    except Exception as e:
        print(f"Error extracting memories: {e}")
        return None
