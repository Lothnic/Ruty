/**
 * Ruty Frontend Application
 * Handles Tauri IPC communication, command system, and UI interactions
 */

import { Store } from './tauri-store.js';
import { commandRegistry } from './commands.js';
import { resultList } from './results.js';

// API Configuration
const API_BASE = 'http://127.0.0.1:3847';
const WS_BASE = 'ws://127.0.0.1:3847';

// State
let sessionId = `session_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
let ws = null;
let isProcessing = false;
let lastFocusTime = 0;
let apiKeys = {}; // Store API keys loaded from store
let inputDebounceTimer = null;

// DOM Elements
const input = document.getElementById('input');
const response = document.getElementById('response');
const responseContent = document.getElementById('response-content');
const tools = document.getElementById('tools');
const toolsText = document.getElementById('tools-text');
const contextBadge = document.getElementById('context-badge');
const contextName = document.getElementById('context-name');
const contextClear = document.getElementById('context-clear');
const container = document.getElementById('container');

/**
 * Initialize WebSocket connection for streaming
 */
function initWebSocket() {
    ws = new WebSocket(`${WS_BASE}/ws/${sessionId}`);

    ws.onopen = () => {
        console.log('🔌 WebSocket connected');
    };

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleWSMessage(data);
    };

    ws.onclose = () => {
        console.log('🔌 WebSocket disconnected, reconnecting...');
        setTimeout(initWebSocket, 2000);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
}

/**
 * Handle incoming WebSocket messages
 */
function handleWSMessage(data) {
    switch (data.type) {
        case 'tool':
            showToolUsage(data.name);
            break;
        case 'response':
            document.querySelector('.input-icon')?.classList.remove('rotating');
            showResponse(data.content);
            isProcessing = false;
            break;
        case 'done':
            hideTools();
            isProcessing = false;
            break;
        case 'error':
            showResponse(`Error: ${data.message}`);
            hideTools();
            isProcessing = false;
            break;
    }
}

/**
 * Send message via HTTP (fallback if WebSocket not available)
 */
async function sendMessageHTTP(message) {
    try {
        const res = await fetch(`${API_BASE}/chat`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ message, session_id: sessionId, api_keys: apiKeys })
        });

        const data = await res.json();

        if (data.tools_used.length > 0) {
            data.tools_used.forEach(tool => showToolUsage(tool));
        }

        showResponse(data.response);
        hideTools();

    } catch (error) {
        showResponse(`Connection error: ${error.message}`);
        hideTools();
    }
}

/**
 * Send message via WebSocket for streaming
 */
function sendMessageWS(message) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ message, api_keys: apiKeys }));
    } else {
        // Fallback to HTTP
        sendMessageHTTP(message);
    }
}

/**
 * Show tool usage indicator (with friendly names for new tools)
 */
function showToolUsage(toolName) {
    const friendlyNames = {
        'search_memory': 'Searching memory...',
        'add_memory': 'Saving to memory...',
        'sync_folder': 'Syncing folder...',
        'upload_file': 'Uploading file...',
        'load_local_context': 'Loading context...',
        'list_documents': 'Listing documents...',
        'delete_document': 'Deleting document...',
        'open_url': 'Opening URL...',
        'run_shell': 'Running command...',
        'get_system_info': 'Getting system info...',
    };

    toolsText.textContent = friendlyNames[toolName] || `Using ${toolName}...`;
    tools.classList.remove('hidden');
    resizeWindow();
}

/**
 * Hide tools indicator
 */
function hideTools() {
    tools.classList.add('hidden');
}

/**
 * Show response in the UI
 */
function showResponse(text) {
    resultList.hide(); // Hide result list when showing response
    responseContent.textContent = text;
    response.classList.remove('hidden');
    resizeWindow();
}

/**
 * Clear response
 */
function clearResponse() {
    response.classList.add('hidden');
    responseContent.textContent = '';
    hideTools();
    resizeWindow();
}

/**
 * Resize Tauri window to fit content
 * Note: We don't re-center to avoid jarring jumps while typing
 * Layout: Input (136px) -> Results (above) -> Response (below)
 */
async function resizeWindow() {
    if (!window.__TAURI__ || !window.__TAURI__.window) return;

    const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;
    const win = getCurrentWindow();

    try {
        const responseEl = document.getElementById('response');
        const toolsEl = document.getElementById('tools');
        const resultsEl = document.getElementById('results');

        const hasResponse = responseEl && !responseEl.classList.contains('hidden');
        const hasTools = toolsEl && !toolsEl.classList.contains('hidden');
        const hasResults = resultsEl && !resultsEl.classList.contains('hidden');

        // Base height (input only)
        let height = 136;

        // Add results height if visible
        if (hasResults) {
            const resultsHeight = Math.min(resultsEl.scrollHeight, 300);
            height += resultsHeight + 12;
        }

        // Add response/tools height if visible
        if (hasResponse || hasTools) {
            // Cap response display at reasonable height
            height += 300;
        }

        // Cap total height
        height = Math.min(height, 700);

        await win.setSize(new LogicalSize(900, height));
        // Don't re-center - causes jarring jumps while typing
    } catch (e) {
        console.log('Resize error:', e);
    }
}

/**
 * Handle input changes - update command results
 */
async function handleInputChange() {
    const query = input.value;

    // Clear debounce timer
    if (inputDebounceTimer) {
        clearTimeout(inputDebounceTimer);
    }

    // Debounce to avoid too many updates
    inputDebounceTimer = setTimeout(async () => {
        // DON'T hide previous response while typing - keep it visible
        // Response will only be replaced when new query is sent

        // Get results from command registry
        const results = await commandRegistry.getResults(query);

        if (results.length > 0 && query.length > 0) {
            resultList.show(results);
            resizeWindow();
        } else if (query.length === 0) {
            resultList.hide();
            resizeWindow();
        }
    }, 100);
}

/**
 * Handle result selection
 */
async function handleResultAction(result, item) {
    if (!result) return;

    switch (result.type) {
        case 'ai':
            // Send to AI
            resultList.hide();
            isProcessing = true;
            const icon = document.querySelector('.input-icon');
            if (icon) icon.classList.add('rotating');
            sendMessageWS(result.query);
            input.value = '';
            break;

        case 'insert':
            // Insert text into input
            input.value = result.value;
            input.focus();
            handleInputChange();
            break;

        case 'navigate':
            // Navigate to URL
            window.location.href = result.url;
            break;

        case 'url':
            // Open URL (tell AI to open it)
            resultList.hide();
            sendMessageWS(`Please open this URL: ${result.url}`);
            input.value = '';
            break;

        case 'copy':
            // Copy to clipboard
            await navigator.clipboard.writeText(result.value);
            showResponse(`✓ Copied: ${result.value}`);
            resultList.hide();
            input.value = '';
            break;

        case 'context':
            // Load context
            resultList.hide();
            await loadContext(result.path);
            input.value = '';
            break;

        case 'clear':
            resultList.hide();
            clearResponse();
            input.value = '';
            break;

        case 'clearContext':
            resultList.hide();
            await clearContext();
            input.value = '';
            break;

        case 'provider':
            // Switch provider
            try {
                await fetch(`${API_BASE}/providers/update`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ provider: result.provider })
                });
                showResponse(`✓ Switched to ${result.provider}`);
            } catch (e) {
                showResponse(`✗ Failed to switch provider: ${e.message}`);
            }
            resultList.hide();
            input.value = '';
            break;

        case 'launchApp':
            // Launch application via Tauri
            resultList.hide();
            input.value = '';
            try {
                if (window.__TAURI__?.core?.invoke) {
                    const msg = await window.__TAURI__.core.invoke('launch_app', { appId: result.appId });
                    showResponse(`✓ ${msg}`);
                    // Optionally hide window after launch
                    if (window.__TAURI__?.window) {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        setTimeout(() => getCurrentWindow().hide(), 500);
                    }
                } else {
                    showResponse(`✗ Tauri not available`);
                }
            } catch (e) {
                showResponse(`✗ Failed to launch: ${e}`);
            }
            break;

        case 'quit':
            // Quit the application
            if (window.__TAURI__?.core?.invoke) {
                // Use Tauri's exit API
                const { exit } = await import('@tauri-apps/plugin-process') || {};
                if (exit) {
                    await exit(0);
                } else {
                    // Fallback: close window
                    if (window.__TAURI__?.window) {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        await getCurrentWindow().close();
                    }
                }
            }
            break;

        case 'openFile':
            // Open file via Tauri
            resultList.hide();
            input.value = '';
            try {
                if (window.__TAURI__?.core?.invoke) {
                    const msg = await window.__TAURI__.core.invoke('open_file', { path: result.path });
                    showResponse(`✓ ${msg}`);
                    // Optionally hide window after open
                    if (window.__TAURI__?.window) {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        setTimeout(() => getCurrentWindow().hide(), 500);
                    }
                } else {
                    showResponse(`✗ Tauri not available`);
                }
            } catch (e) {
                showResponse(`✗ Failed to open: ${e}`);
            }
            break;

        case 'copyToClipboard':
            // Copy via Rust backend (moves to top of history/pastes to clipboard)
            resultList.hide();
            input.value = '';
            try {
                if (window.__TAURI__?.core?.invoke) {
                    await window.__TAURI__.core.invoke('copy_to_clipboard', { content: result.content });
                    showResponse('✓ Copied to clipboard');
                    // Hide window
                    if (window.__TAURI__?.window) {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        setTimeout(() => getCurrentWindow().hide(), 500);
                    }
                } else {
                    // Fallback to browser API if Tauri not available (but browser API is limited)
                    try {
                        await navigator.clipboard.writeText(result.content);
                        showResponse('✓ Copied (local)');
                    } catch (e) {
                        showResponse(`✗ Failed to copy: ${e}`);
                    }
                }
            } catch (e) {
                showResponse(`✗ Failed to copy: ${e}`);
            }
            break;

        case 'openUrl':
            // Open URL via open_file (xdg-open handles URLs)
            resultList.hide();
            input.value = '';
            try {
                if (window.__TAURI__?.core?.invoke) {
                    await window.__TAURI__.core.invoke('open_file', { path: result.url });
                    showResponse(`✓ Opened ${result.url}`);
                    // Hide window
                    if (window.__TAURI__?.window) {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        setTimeout(() => getCurrentWindow().hide(), 500);
                    }
                } else {
                    window.open(result.url, '_blank');
                    showResponse('✓ Opened in browser');
                }
            } catch (e) {
                showResponse(`✗ Failed to open URL: ${e}`);
            }
            break;
    }
}

// Initialize clipboard monitor
if (window.__TAURI__?.core?.invoke) {
    window.__TAURI__.core.invoke('init_clipboard')
        .catch(e => console.error('Failed to init clipboard:', e));
}

/**
 * Handle input submission (Enter key)
 */
async function handleSubmit() {
    const message = input.value.trim();
    if (!message || isProcessing) return;

    // Capture selected result before hiding (if any)
    const selectedResult = resultList.isVisible() ? resultList.getSelected() : null;

    // Hide result list immediately to prevent flash
    resultList.hide();
    resizeWindow();

    // If there was a selected result with an action, execute it
    if (selectedResult && selectedResult.action) {
        const result = await selectedResult.action();
        await handleResultAction(result, selectedResult);
        return;
    }

    // Otherwise send to AI directly
    isProcessing = true;
    input.value = '';

    const icon = document.querySelector('.input-icon');
    if (icon) icon.classList.add('rotating');

    sendMessageWS(message);
}

/**
 * Load local context
 */
async function loadContext(path) {
    try {
        const res = await fetch(`${API_BASE}/context/load`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path, session_id: sessionId })
        });

        const data = await res.json();

        if (data.success) {
            contextName.textContent = data.loaded;
            contextBadge.classList.remove('hidden');
            showResponse(`✓ Loaded: ${data.loaded}`);
        } else {
            showResponse(`Error: ${data.error}`);
        }
    } catch (error) {
        showResponse(`Error loading context: ${error.message}`);
    }
}

/**
 * Clear local context
 */
async function clearContext() {
    try {
        await fetch(`${API_BASE}/context/clear?session_id=${sessionId}`, {
            method: 'POST'
        });
        contextBadge.classList.add('hidden');
        showResponse('✓ Context cleared');
    } catch (error) {
        console.error('Error clearing context:', error);
    }
}

/**
 * Toggle window visibility (Tauri command)
 */
async function toggleWindow() {
    if (window.__TAURI__ && window.__TAURI__.tauri) {
        const { invoke } = window.__TAURI__.tauri;
        await invoke('toggle_window_cmd');
    }
}

// ============== Event Listeners ==============

// Input changes
input.addEventListener('input', handleInputChange);

// Keyboard navigation
input.addEventListener('keydown', (e) => {
    // Prevent space input if window just focused (avoids Super+Space artifact)
    if (e.key === ' ' && Date.now() - lastFocusTime < 200) {
        e.preventDefault();
        return;
    }

    // Let result list handle navigation keys
    if (resultList.handleKeyDown(e)) {
        return;
    }

    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
    }
});

// Escape to hide window or clear
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        if (resultList.isVisible()) {
            resultList.hide();
            resizeWindow();
        } else if (window.__TAURI__) {
            toggleWindow();
        }
    }
});

// Clear context button
contextClear.addEventListener('click', clearContext);

// Window focus tracking
window.addEventListener('focus', () => {
    lastFocusTime = Date.now();
    input.focus();
});

// Set up result list callback
resultList.onSelect = handleResultAction;

// ============== Initialize ==============
document.addEventListener('DOMContentLoaded', () => {
    // Retry connecting to API with backoff
    const tryConnect = (attempts = 0) => {
        if (attempts >= 10) {
            console.log('Max retries reached, will use HTTP fallback on demand');
            return;
        }

        fetch(`${API_BASE}/health`)
            .then(res => res.json())
            .then((data) => {
                console.log(`✅ Connected to Python backend (${data.provider}/${data.model})`);
                initWebSocket();
            })
            .catch(() => {
                const delay = Math.min(500 * Math.pow(1.5, attempts), 5000);
                console.log(`Backend not ready, retrying in ${delay}ms...`);
                setTimeout(() => tryConnect(attempts + 1), delay);
            });
    };

    // Wait a bit before first attempt to let backend start
    setTimeout(() => tryConnect(), 1000);

    input.focus();

    // Force initial resize
    setTimeout(resizeWindow, 100);

    // Initialize Store and load keys
    if (window.__TAURI__) {
        const store = new Store('settings.json');

        store.get('api_keys').then(keys => {
            if (keys && (keys.groq || keys.supermemory)) {
                console.log('✅ API keys loaded');
                apiKeys = keys;
            } else {
                console.log('⚠️ No API keys found - some features may not work');
                // Don't auto-redirect, let user choose to configure
            }
        }).catch(err => {
            console.error('Failed to load settings:', err);
            // Don't redirect on error - just continue without keys
        });
    }
});
