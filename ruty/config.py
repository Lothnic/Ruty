from contextvars import ContextVar

# Context variable to store API keys for the current request
# Keys: "groq", "supermemory"
api_key_context: ContextVar[dict] = ContextVar("api_keys", default={})
