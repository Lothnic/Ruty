//! Slash command parsing and handling
//!
//! Parses commands like /context, /clear, /providers from user input.

/// Parsed command from user input
#[derive(Debug, Clone)]
pub enum Command {
    /// Search and launch apps: /app <query>
    App { query: String },
    /// Load local files as context: /context <path>
    Context { path: String },
    /// Clear conversation: /clear
    Clear,
    /// Show/switch providers: /providers [provider] [model]
    Providers { 
        provider: Option<String>, 
        model: Option<String> 
    },
    /// Open settings: /settings
    Settings,
    /// Show help: /help
    Help,
    /// Not a command, regular chat message (default - AI)
    Chat { message: String },
}

impl Command {
    /// Parse user input into a command
    pub fn parse(input: &str) -> Self {
        let input = input.trim();
        
        if !input.starts_with('/') {
            return Command::Chat { message: input.to_string() };
        }
        
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let args = parts.get(1).map(|s| s.trim()).unwrap_or("");
        
        match cmd.as_str() {
            "/app" | "/a" => {
                if args.is_empty() {
                    Command::Chat { 
                        message: "Usage: /app <query>".to_string() 
                    }
                } else {
                    Command::App { query: args.to_string() }
                }
            }
            "/context" | "/ctx" | "/c" => {
                if args.is_empty() {
                    Command::Chat { 
                        message: "Usage: /context <path>".to_string() 
                    }
                } else {
                    Command::Context { path: args.to_string() }
                }
            }
            "/clear" | "/cl" => Command::Clear,
            "/providers" | "/provider" | "/p" => {
                let provider_parts: Vec<&str> = args.split_whitespace().collect();
                Command::Providers {
                    provider: provider_parts.first().map(|s| s.to_string()),
                    model: provider_parts.get(1).map(|s| s.to_string()),
                }
            }
            "/settings" | "/s" => Command::Settings,
            "/help" | "/h" | "/?" => Command::Help,
            _ => Command::Chat { 
                message: format!("Unknown command: {}. Type /help for available commands.", cmd) 
            },
        }
    }
    
    /// Get help text for all commands
    pub fn help_text() -> &'static str {
        r#"Available Commands:
/app <query>     - Search and launch applications (default: AI)
/context <path>  - Load local files as context
/clear           - Clear conversation history
/providers       - Show available providers
/settings        - Open settings
/help            - Show this help

Tip: Just type your question to chat with AI!"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_context() {
        match Command::parse("/context ./src") {
            Command::Context { path } => assert_eq!(path, "./src"),
            _ => panic!("Expected Context command"),
        }
    }
    
    #[test]
    fn test_parse_clear() {
        match Command::parse("/clear") {
            Command::Clear => {}
            _ => panic!("Expected Clear command"),
        }
    }
    
    #[test]
    fn test_parse_chat() {
        match Command::parse("Hello world") {
            Command::Chat { message } => assert_eq!(message, "Hello world"),
            _ => panic!("Expected Chat"),
        }
    }
}
