/**
 * Ruty Settings Page
 * Handles API key configuration using Tauri Store plugin
 */

import { Store } from './tauri-store.js';

document.addEventListener('DOMContentLoaded', async () => {
    // If not in Tauri, show warning
    if (!window.__TAURI__) {
        console.warn('[Settings] Tauri API not found');
        document.body.innerHTML = '<p style="color:white;padding:20px;">Please run this app through Tauri.</p>';
        return;
    }

    const store = new Store('settings.json');

    const groqInput = document.getElementById('groq-key');
    const smInput = document.getElementById('supermemory-key');
    const saveBtn = document.getElementById('save-btn');
    const cancelBtn = document.getElementById('cancel-btn');

    // Load existing keys
    try {
        console.log('[Settings] Loading existing keys...');
        const keys = await store.get('api_keys');
        console.log('[Settings] Loaded keys:', keys ? 'found' : 'none');

        if (keys) {
            groqInput.value = keys.groq || '';
            smInput.value = keys.supermemory || '';
        }
    } catch (e) {
        console.error('[Settings] Failed to load settings:', e);
    }

    // Cancel handler - always go back to main page
    cancelBtn.addEventListener('click', () => {
        window.location.href = 'index.html';
    });

    // Save handler
    saveBtn.addEventListener('click', async () => {
        const groqKey = groqInput.value.trim();
        const smKey = smInput.value.trim();

        // Validate - at least one key should be provided
        if (!groqKey && !smKey) {
            alert('Please enter at least one API key.');
            return;
        }

        const originalText = saveBtn.textContent;
        saveBtn.textContent = 'Saving...';
        saveBtn.disabled = true;

        try {
            console.log('[Settings] Saving keys...');

            // Save the keys
            await store.set('api_keys', {
                groq: groqKey,
                supermemory: smKey
            });

            // Persist to disk
            await store.save();

            console.log('[Settings] Keys saved successfully!');

            // Show success briefly
            saveBtn.textContent = '✓ Saved!';
            saveBtn.style.background = '#22c55e';

            // Redirect back to chat after a brief delay
            setTimeout(() => {
                window.location.href = 'index.html';
            }, 500);

        } catch (e) {
            console.error('[Settings] Save failed:', e);

            // Show error to user
            saveBtn.textContent = '✗ Error';
            saveBtn.style.background = '#ef4444';

            // Show detailed error
            const errorMsg = e.message || String(e);
            alert(`Failed to save settings:\n${errorMsg}\n\nCheck the browser console for details.`);

            setTimeout(() => {
                saveBtn.textContent = originalText;
                saveBtn.style.background = '';
                saveBtn.disabled = false;
            }, 2000);
        }
    });

    // Resize window for settings view
    try {
        if (window.__TAURI__?.window) {
            const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;
            const win = getCurrentWindow();
            await win.setSize(new LogicalSize(900, 400));
            await win.center();
            await win.show();
        }
    } catch (e) {
        console.error('[Settings] Window resize error:', e);
    }

    // Focus first empty input
    if (!groqInput.value) {
        groqInput.focus();
    } else if (!smInput.value) {
        smInput.focus();
    }
});
