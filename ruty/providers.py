"""LLM Provider configuration for Ruty.

Supports multiple providers with BYOK (Bring Your Own Key) capability.
Currently supports: Groq, OpenAI, Ollama (local)
"""
import os
from dataclasses import dataclass, field
from typing import Optional
from pathlib import Path
import json


@dataclass
class ProviderConfig:
    """Configuration for an LLM provider."""
    name: str
    base_url: str
    default_model: str
    api_key_env: str  # Environment variable name for API key
    models: list[str] = field(default_factory=list)
    requires_key: bool = True


# Supported providers
PROVIDERS = {
    "groq": ProviderConfig(
        name="Groq",
        base_url="https://api.groq.com/openai/v1",
        default_model="moonshotai/kimi-k2-instruct",
        api_key_env="GROQ_API_KEY",
        models=[
            "moonshotai/kimi-k2-instruct",
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "gemma2-9b-it",
            "mixtral-8x7b-32768",
        ],
    ),
    "openai": ProviderConfig(
        name="OpenAI",
        base_url="https://api.openai.com/v1",
        default_model="gpt-4o-mini",
        api_key_env="OPENAI_API_KEY",
        models=[
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
        ],
    ),
    "ollama": ProviderConfig(
        name="Ollama (Local)",
        base_url="http://localhost:11434/v1",
        default_model="llama3.2",
        api_key_env="",
        requires_key=False,
        models=[
            "llama3.2",
            "llama3.1",
            "mistral",
            "codellama",
            "qwen2.5",
        ],
    ),
    "openrouter": ProviderConfig(
        name="OpenRouter",
        base_url="https://openrouter.ai/api/v1",
        default_model="anthropic/claude-3.5-sonnet",
        api_key_env="OPENROUTER_API_KEY",
        models=[
            "anthropic/claude-3.5-sonnet",
            "anthropic/claude-3-haiku",
            "google/gemini-2.0-flash-exp:free",
            "meta-llama/llama-3.3-70b-instruct",
        ],
    ),
}


@dataclass
class RutyConfig:
    """Main Ruty configuration."""
    provider: str = "groq"
    model: Optional[str] = None  # None = use provider default
    api_keys: dict = field(default_factory=dict)  # Provider -> API key
    supermemory_key: Optional[str] = None
    
    # UI preferences
    theme: str = "dark"
    hotkey: str = "Super+Space"
    
    @property
    def current_provider(self) -> ProviderConfig:
        """Get the current provider config."""
        return PROVIDERS.get(self.provider, PROVIDERS["groq"])
    
    @property
    def current_model(self) -> str:
        """Get the current model name."""
        return self.model or self.current_provider.default_model
    
    @property
    def current_api_key(self) -> Optional[str]:
        """Get API key for current provider."""
        # First check stored keys, then environment
        if self.provider in self.api_keys:
            key = self.api_keys[self.provider]
            if key and str(key).strip():
                return str(key).strip()
        
        env_key = os.getenv(self.current_provider.api_key_env)
        if env_key and env_key.strip():
            return env_key.strip()
        return None
    
    def get_supermemory_key(self) -> Optional[str]:
        """Get Supermemory API key."""
        key = self.supermemory_key or os.getenv("SUPERMEMORY_API_KEY")
        return key.strip() if key else None


# Config file path
CONFIG_DIR = Path.home() / ".config" / "ruty"
CONFIG_FILE = CONFIG_DIR / "config.json"


def load_config() -> RutyConfig:
    """Load configuration from file or return defaults."""
    if CONFIG_FILE.exists():
        try:
            with open(CONFIG_FILE, "r") as f:
                data = json.load(f)
            return RutyConfig(**data)
        except Exception as e:
            print(f"⚠️ Failed to load config: {e}")
    return RutyConfig()


def save_config(config: RutyConfig):
    """Save configuration to file."""
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    with open(CONFIG_FILE, "w") as f:
        json.dump({
            "provider": config.provider,
            "model": config.model,
            "api_keys": config.api_keys,
            "supermemory_key": config.supermemory_key,
            "theme": config.theme,
            "hotkey": config.hotkey,
        }, f, indent=2)


# Singleton config instance
_config: Optional[RutyConfig] = None


def get_config() -> RutyConfig:
    """Get or load the configuration."""
    global _config
    if _config is None:
        _config = load_config()
    return _config


def update_config(**kwargs) -> RutyConfig:
    """Update configuration and save."""
    global _config
    config = get_config()
    for key, value in kwargs.items():
        if hasattr(config, key):
            setattr(config, key, value)
    save_config(config)
    _config = config
    return config


def list_providers() -> dict:
    """List available providers for UI."""
    return {
        pid: {
            "name": p.name,
            "models": p.models,
            "default_model": p.default_model,
            "requires_key": p.requires_key,
        }
        for pid, p in PROVIDERS.items()
    }
