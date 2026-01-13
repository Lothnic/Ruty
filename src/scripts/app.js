/**
 * Ruty Frontend Application
 * Handles Tauri IPC communication and UI interactions
 */

// API Configuration
const API_BASE = 'http://127.0.0.1:3847';
const WS_BASE = 'ws://127.0.0.1:3847';

// State
let sessionId = `session_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
let ws = null;
let isProcessing = false;
let lastFocusTime = 0;

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
            body: JSON.stringify({ message, session_id: sessionId })
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
        ws.send(JSON.stringify({ message }));
    } else {
        // Fallback to HTTP
        sendMessageHTTP(message);
    }
}

/**
 * Show tool usage indicator
 */
function showToolUsage(toolName) {
    const friendlyNames = {
        'search_memory': 'Searching memory...',
        'add_memory': 'Saving to memory...',
        'sync_folder': 'Syncing folder...',
        'upload_file': 'Uploading file...',
        'load_local_context': 'Loading context...',
        'list_documents': 'Listing documents...',
        'delete_document': 'Deleting document...'
    };

    toolsText.textContent = friendlyNames[toolName] || `Using ${toolName}...`;
    tools.classList.remove('hidden');
    resizeWindow(); // Also resize when tools appear
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
    responseContent.textContent = text;
    response.classList.remove('hidden');

    // Resize window to fit content (Tauri-specific)
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
 */
async function resizeWindow() {
    console.log('resizeWindow called');
    console.log('__TAURI__ exists:', !!window.__TAURI__);
    if (window.__TAURI__) {
        console.log('__TAURI__ keys:', Object.keys(window.__TAURI__));
    }

    if (!window.__TAURI__ || !window.__TAURI__.window) {
        console.log('No Tauri window API available');
        return;
    }

    const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;
    const win = getCurrentWindow();

    try {
        const responseEl = document.getElementById('response');
        const toolsEl = document.getElementById('tools');
        const hasResponse = responseEl && !responseEl.classList.contains('hidden');
        const hasTools = toolsEl && !toolsEl.classList.contains('hidden');

        console.log('hasResponse:', hasResponse, 'hasTools:', hasTools);

        // Small when empty, large when any content exists
        const height = (hasResponse || hasTools) ? 600 : 136;

        console.log('Setting height to:', height);

        await win.setSize(new LogicalSize(900, height));
        await win.center();

        console.log('Resize complete');
    } catch (e) {
        console.log('Resize error:', e);
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

/**
 * Handle input submission
 */
async function handleSubmit() {
    const message = input.value.trim();
    if (!message || isProcessing) return;

    isProcessing = true;
    input.value = '';

    // Show loading state on input icon instead of response area
    const icon = document.querySelector('.input-icon');
    if (icon) icon.classList.add('rotating');

    // Send message
    sendMessageWS(message);
}

/**
 * Handle slash commands
 */
async function handleCommand(cmd) {
    const [command, ...args] = cmd.slice(1).split(' ');
    const arg = args.join(' ');

    switch (command.toLowerCase()) {
        case 'context':
            if (!arg || arg === 'clear') {
                clearContext();
            } else {
                await loadContext(arg);
            }
            break;
        case 'clear':
            clearResponse();
            break;
        default:
            showResponse(`Unknown command: /${command}`);
    }
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

// Event Listeners
input.addEventListener('keydown', (e) => {
    // Prevent space input if window just focused (avoids Super+Space artifact)
    if (e.key === ' ' && Date.now() - lastFocusTime < 200) {
        e.preventDefault();
        return;
    }

    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        const value = input.value.trim();

        if (value.startsWith('/')) {
            handleCommand(value);
            input.value = '';
        } else {
            handleSubmit();
        }
    }
});

// Escape to hide window
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        if (window.__TAURI__) {
            toggleWindow();
        }
    }
});

// Clear context button
contextClear.addEventListener('click', clearContext);

// Window focus tracking
window.addEventListener('focus', () => {
    lastFocusTime = Date.now();
});

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    // Retry connecting to API with backoff
    const tryConnect = (attempts = 0) => {
        if (attempts >= 10) {
            console.log('Max retries reached, will use HTTP fallback on demand');
            return;
        }

        fetch(`${API_BASE}/health`)
            .then(res => res.json())
            .then(() => {
                console.log('✅ Connected to Python backend');
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

    // Force initial resize to ensure correct padding/shadows are visible
    setTimeout(resizeWindow, 100);
});
