/**
 * Ruty Command Registry
 * Manages built-in and plugin commands for the launcher
 */

// Command types for icons and styling
export const CommandType = {
    AI: 'ai',
    APP: 'app',
    FILE: 'file',
    SYSTEM: 'system',
    CLIPBOARD: 'clipboard',
    ACTION: 'action',
};

// Icons by command type (using SVG paths)
export const TypeIcons = {
    ai: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 3a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5zm5 10.5a7 7 0 0 1-10 0 1.5 1.5 0 0 1 0-2 7 7 0 0 1 10 0 1.5 1.5 0 0 1 0 2z"/></svg>`,
    app: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/></svg>`,
    file: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`,
    system: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/></svg>`,
    clipboard: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>`,
    folder: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>`,
    action: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`,
};

/**
 * Base command result structure
 */
export class CommandResult {
    constructor({
        id,
        title,
        subtitle = '',
        type = CommandType.ACTION,
        icon = null,
        action = null,
        shortcut = null,
        data = null,
    }) {
        this.id = id;
        this.title = title;
        this.subtitle = subtitle;
        this.type = type;
        this.icon = icon || TypeIcons[type];
        this.action = action;
        this.shortcut = shortcut;
        this.data = data;
    }
}

/**
 * Command registry - central store for all commands
 */
class CommandRegistry {
    constructor() {
        this.commands = new Map();
        this.registerBuiltInCommands();
    }

    /**
     * Register a command
     */
    register(name, handler) {
        this.commands.set(name.toLowerCase(), handler);
    }

    /**
     * Get all registered command names
     */
    getCommandNames() {
        return Array.from(this.commands.keys());
    }

    /**
     * Get results for a query
     */
    async getResults(query) {
        // Parse the query
        const trimmed = query.trim();

        // Empty query - show quick actions
        if (!trimmed) {
            return this.getQuickActions();
        }

        // Command mode (/ prefix)
        if (trimmed.startsWith('/')) {
            return this.handleCommandQuery(trimmed);
        }

        // AI explicit mode (> prefix)
        if (trimmed.startsWith('>')) {
            return [{
                id: 'ai_query',
                title: `Ask AI: ${trimmed.slice(1).trim()}`,
                subtitle: 'Send to Ruty AI assistant',
                type: CommandType.AI,
                icon: TypeIcons.ai,
                action: async () => ({ type: 'ai', query: trimmed.slice(1).trim() }),
            }];
        }

        // Universal search - combine all sources
        return this.universalSearch(trimmed);
    }

    /**
     * Handle /command queries
     */
    async handleCommandQuery(query) {
        const parts = query.slice(1).split(' ');
        const cmdName = parts[0].toLowerCase();
        const args = parts.slice(1).join(' ');

        // If no command name yet, show available commands
        if (!cmdName) {
            return this.getAvailableCommands();
        }

        // Find matching commands
        const matches = this.getCommandNames().filter(name =>
            name.startsWith(cmdName)
        );

        // If exact match, execute
        if (this.commands.has(cmdName)) {
            const handler = this.commands.get(cmdName);
            return handler.getResults ? await handler.getResults(args) : [];
        }

        // Show matching commands
        return matches.map(name => {
            const handler = this.commands.get(name);
            return new CommandResult({
                id: `cmd_${name}`,
                title: `/${name}`,
                subtitle: handler.description || '',
                type: handler.type || CommandType.ACTION,
                action: () => ({ type: 'insert', value: `/${name} ` }),
            });
        });
    }

    /**
     * Universal search across all sources
     */
    async universalSearch(query) {
        const results = [];

        // Always include AI as fallback
        results.push(new CommandResult({
            id: 'ai_fallback',
            title: query,
            subtitle: 'Ask Ruty AI',
            type: CommandType.AI,
            action: async () => ({ type: 'ai', query }),
        }));

        // Check for URL-like patterns
        if (query.match(/^(https?:\/\/|www\.)/) || query.match(/\.(com|org|net|io|dev|app)/)) {
            results.unshift(new CommandResult({
                id: 'open_url',
                title: `Open ${query}`,
                subtitle: 'Open in browser',
                type: CommandType.ACTION,
                icon: TypeIcons.action,
                action: async () => ({ type: 'url', url: query }),
            }));
        }

        // Check for calculator patterns
        if (query.match(/^[\d\s\+\-\*\/\(\)\.%]+$/)) {
            try {
                // Safe eval for math expressions
                const sanitized = query.replace(/[^-()\d/*+.%]/g, '');
                const result = new Function('return ' + sanitized)();
                if (typeof result === 'number' && isFinite(result)) {
                    results.unshift(new CommandResult({
                        id: 'calc_result',
                        title: `= ${result}`,
                        subtitle: `Copy result`,
                        type: CommandType.ACTION,
                        action: async () => ({ type: 'copy', value: String(result) }),
                    }));
                }
            } catch (e) {
                // Ignore calc errors
            }
        }

        return results;
    }

    /**
     * Get quick actions for empty query
     */
    getQuickActions() {
        return [
            new CommandResult({
                id: 'qa_ai',
                title: 'Ask AI',
                subtitle: 'Chat with Ruty assistant',
                type: CommandType.AI,
                shortcut: '↵',
            }),
            new CommandResult({
                id: 'qa_apps',
                title: 'Search Apps',
                subtitle: 'Launch applications',
                type: CommandType.APP,
                action: () => ({ type: 'insert', value: '/app ' }),
            }),
            new CommandResult({
                id: 'qa_files',
                title: 'Search Files',
                subtitle: 'Find files on your system',
                type: CommandType.FILE,
                action: () => ({ type: 'insert', value: '/file ' }),
            }),
            new CommandResult({
                id: 'qa_clipboard',
                title: 'Clipboard History',
                subtitle: 'View clipboard history',
                type: CommandType.CLIPBOARD,
                action: () => ({ type: 'insert', value: '/clip' }),
            }),
            new CommandResult({
                id: 'qa_settings',
                title: 'Settings',
                subtitle: 'Configure Ruty',
                type: CommandType.SYSTEM,
                action: () => ({ type: 'navigate', url: 'settings.html' }),
            }),
        ];
    }

    /**
     * Get available commands as results
     */
    getAvailableCommands() {
        return this.getCommandNames().map(name => {
            const handler = this.commands.get(name);
            return new CommandResult({
                id: `cmd_${name}`,
                title: `/${name}`,
                subtitle: handler.description || '',
                type: handler.type || CommandType.ACTION,
                action: () => ({ type: 'insert', value: `/${name} ` }),
            });
        });
    }

    /**
     * Register built-in commands
     */
    registerBuiltInCommands() {
        // Context command
        this.register('context', {
            description: 'Load local files as context',
            type: CommandType.FILE,
            getResults: async (args) => {
                if (!args) {
                    return [new CommandResult({
                        id: 'context_hint',
                        title: '/context <path>',
                        subtitle: 'Enter a file or folder path to load as context',
                        type: CommandType.FILE,
                    })];
                }
                return [new CommandResult({
                    id: 'context_load',
                    title: `Load: ${args}`,
                    subtitle: 'Load as context for AI',
                    type: CommandType.FILE,
                    action: async () => ({ type: 'context', path: args }),
                })];
            }
        });

        // Clear command
        this.register('clear', {
            description: 'Clear response and context',
            type: CommandType.ACTION,
            getResults: async () => [
                new CommandResult({
                    id: 'clear_response',
                    title: 'Clear Response',
                    subtitle: 'Clear the current response',
                    type: CommandType.ACTION,
                    action: async () => ({ type: 'clear' }),
                }),
                new CommandResult({
                    id: 'clear_context',
                    title: 'Clear Context',
                    subtitle: 'Remove loaded file context',
                    type: CommandType.ACTION,
                    action: async () => ({ type: 'clearContext' }),
                }),
            ]
        });

        // App command - search and launch applications via Rust backend
        this.register('app', {
            description: 'Search and launch applications',
            type: CommandType.APP,
            getResults: async (args) => {
                // Check if Tauri is available
                if (!window.__TAURI__?.core?.invoke) {
                    return [new CommandResult({
                        id: 'app_no_tauri',
                        title: 'App search unavailable',
                        subtitle: 'Tauri API not found',
                        type: CommandType.APP,
                    })];
                }

                try {
                    const invoke = window.__TAURI__.core.invoke;
                    const apps = await invoke('search_apps', { query: args || '' });

                    if (apps.length === 0) {
                        return [new CommandResult({
                            id: 'app_none',
                            title: args ? `No apps matching "${args}"` : 'No applications found',
                            subtitle: 'Try a different search term',
                            type: CommandType.APP,
                        })];
                    }

                    return apps.map(app => new CommandResult({
                        id: `app_${app.id}`,
                        title: app.name,
                        subtitle: app.subtitle,
                        type: CommandType.APP,
                        data: { appId: app.id },
                        action: async () => ({ type: 'launchApp', appId: app.id, appName: app.name }),
                    }));
                } catch (e) {
                    console.error('App search error:', e);
                    return [new CommandResult({
                        id: 'app_error',
                        title: 'Error searching apps',
                        subtitle: String(e),
                        type: CommandType.APP,
                    })];
                }
            }
        });

        // File command - search files via Rust backend
        this.register('file', {
            description: 'Search files',
            type: CommandType.FILE,
            getResults: async (args) => {
                if (!args || args.length < 2) {
                    return [new CommandResult({
                        id: 'file_hint',
                        title: '/file <search term>',
                        subtitle: 'Enter at least 2 characters to search',
                        type: CommandType.FILE,
                    })];
                }

                // Check if Tauri is available
                if (!window.__TAURI__?.core?.invoke) {
                    return [new CommandResult({
                        id: 'file_no_tauri',
                        title: 'File search unavailable',
                        subtitle: 'Tauri API not found',
                        type: CommandType.FILE,
                    })];
                }

                try {
                    const invoke = window.__TAURI__.core.invoke;
                    const files = await invoke('search_files', { query: args, maxResults: 15 });

                    if (files.length === 0) {
                        return [new CommandResult({
                            id: 'file_none',
                            title: `No files matching "${args}"`,
                            subtitle: 'Try a different search term',
                            type: CommandType.FILE,
                        })];
                    }

                    return files.map(file => new CommandResult({
                        id: `file_${file.path}`,
                        title: file.name,
                        subtitle: file.path,
                        type: CommandType.FILE,
                        icon: file.is_dir ? TypeIcons.file : TypeIcons.file,
                        data: { path: file.path, isDir: file.is_dir },
                        action: async () => ({ type: 'openFile', path: file.path, name: file.name }),
                    }));
                } catch (e) {
                    console.error('File search error:', e);
                    return [new CommandResult({
                        id: 'file_error',
                        title: 'Error searching files',
                        subtitle: String(e),
                        type: CommandType.FILE,
                    })];
                }
            }
        });

        // Folder command
        this.register('folder', {
            description: 'Search folders',
            type: CommandType.FILE,
            getResults: async (args) => {
                if (!args || args.length < 2) {
                    return [new CommandResult({
                        id: 'folder_hint',
                        title: '/folder <search term>',
                        subtitle: 'Enter at least 2 characters to search folders',
                        type: CommandType.FILE,
                    })];
                }

                if (!window.__TAURI__?.core?.invoke) {
                    return [new CommandResult({
                        id: 'folder_no_tauri',
                        title: 'Folder search unavailable',
                        subtitle: 'Tauri API not found',
                        type: CommandType.FILE,
                    })];
                }

                try {
                    const invoke = window.__TAURI__.core.invoke;
                    // Pass foldersOnly: true to filtering
                    const files = await invoke('search_files', { query: args, maxResults: 15, foldersOnly: true });

                    if (files.length === 0) {
                        return [new CommandResult({
                            id: 'folder_none',
                            title: `No folders matching "${args}"`,
                            subtitle: 'Try a different search term',
                            type: CommandType.FILE,
                        })];
                    }

                    return files.map(file => new CommandResult({
                        id: `folder_${file.path}`,
                        title: file.name,
                        subtitle: file.path,
                        type: CommandType.FILE,
                        icon: TypeIcons.file, // Could add a distinct folder icon if available
                        data: { path: file.path, isDir: true },
                        action: async () => ({ type: 'openFile', path: file.path, name: file.name }),
                    }));
                } catch (e) {
                    console.error('Folder search error:', e);
                    return [new CommandResult({
                        id: 'folder_error',
                        title: 'Error searching folders',
                        subtitle: String(e),
                        type: CommandType.FILE,
                    })];
                }
            }
        });

        // Clipboard command
        this.register('clip', {
            description: 'Clipboard history',
            type: CommandType.CLIPBOARD,
            getResults: async (args) => {
                if (!window.__TAURI__?.core?.invoke) {
                    return [new CommandResult({
                        id: 'clip_no_tauri',
                        title: 'Clipboard history unavailable',
                        subtitle: 'Tauri API not found',
                        type: CommandType.CLIPBOARD,
                    })];
                }

                try {
                    const invoke = window.__TAURI__.core.invoke;
                    const history = await invoke('get_clipboard_history');

                    if (history.length === 0) {
                        return [new CommandResult({
                            id: 'clip_empty',
                            title: 'Clipboard history empty',
                            subtitle: 'Copy some text to see it here',
                            type: CommandType.CLIPBOARD,
                        })];
                    }

                    // Filter by args if provided
                    let filtered = history;
                    if (args) {
                        const lowerArgs = args.toLowerCase();
                        filtered = history.filter(item => item.content.toLowerCase().includes(lowerArgs));
                    }

                    if (filtered.length === 0) {
                        return [new CommandResult({
                            id: 'clip_none',
                            title: `No items matching "${args}"`,
                            subtitle: 'Try a different search term',
                            type: CommandType.CLIPBOARD,
                        })];
                    }

                    return filtered.map(item => {
                        // Truncate for title
                        const title = item.content.length > 50
                            ? item.content.slice(0, 50).replace(/\n/g, ' ') + '...'
                            : item.content.replace(/\n/g, ' ');

                        return new CommandResult({
                            id: `clip_${item.timestamp}`,
                            title: title,
                            subtitle: 'Press Enter to copy',
                            type: CommandType.CLIPBOARD,
                            data: { content: item.content },
                            action: async () => ({ type: 'copyToClipboard', content: item.content }),
                        });
                    });
                } catch (e) {
                    console.error('Clipboard error:', e);
                    return [new CommandResult({
                        id: 'clip_error',
                        title: 'Error loading history',
                        subtitle: String(e),
                        type: CommandType.CLIPBOARD,
                    })];
                }
            }
        });

        // Provider command
        this.register('provider', {
            description: 'Change AI provider',
            type: CommandType.SYSTEM,
            getResults: async () => {
                try {
                    const res = await fetch('http://127.0.0.1:3847/providers');
                    const data = await res.json();
                    return Object.entries(data.providers).map(([id, p]) =>
                        new CommandResult({
                            id: `provider_${id}`,
                            title: p.name,
                            subtitle: id === data.current.provider ? '✓ Current' : `Switch to ${p.name}`,
                            type: CommandType.SYSTEM,
                            action: async () => ({ type: 'provider', provider: id }),
                        })
                    );
                } catch {
                    return [new CommandResult({
                        id: 'provider_error',
                        title: 'Error loading providers',
                        subtitle: 'Backend not available',
                        type: CommandType.SYSTEM,
                    })];
                }
            }
        });

        // Quit command
        this.register('quit', {
            description: 'Quit Ruty',
            type: CommandType.SYSTEM,
            getResults: async () => [
                new CommandResult({
                    id: 'quit_app',
                    title: 'Quit Ruty',
                    subtitle: 'Exit the application',
                    type: CommandType.SYSTEM,
                    action: async () => ({ type: 'quit' }),
                })
            ]
        });
    }
}

// Singleton instance
export const commandRegistry = new CommandRegistry();
