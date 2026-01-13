document.addEventListener('DOMContentLoaded', async () => {
    // If not in Tauri, show warning
    if (!window.__TAURI__) {
        console.warn('Tauri API not found');
        return;
    }

    const { Store } = window.__TAURI__.store;
    // Use the same store filename as app.js
    const store = new Store('.settings.dat');

    const groqInput = document.getElementById('groq-key');
    const smInput = document.getElementById('supermemory-key');
    const saveBtn = document.getElementById('save-btn');

    try {
        // Load existing keys
        const keys = await store.get('api_keys');
        if (keys) {
            groqInput.value = keys.groq || '';
            smInput.value = keys.supermemory || '';
        }
    } catch (e) {
        console.error('Failed to load settings:', e);
    }

    // Save handler
    saveBtn.addEventListener('click', async () => {
        const originalText = saveBtn.textContent;
        saveBtn.textContent = 'Saving...';
        saveBtn.disabled = true;

        try {
            await store.set('api_keys', {
                groq: groqInput.value.trim(),
                supermemory: smInput.value.trim()
            });
            await store.save();

            // Redirect back to chat
            window.location.href = 'index.html';
        } catch (e) {
            console.error('Failed to save:', e);
            saveBtn.textContent = 'Error!';
            setTimeout(() => {
                saveBtn.textContent = originalText;
                saveBtn.disabled = false;
            }, 2000);
        }
    });

    // Resize window for settings view
    if (window.__TAURI__.window) {
        const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;
        const win = getCurrentWindow();
        await win.setSize(new LogicalSize(900, 600));
        await win.center();
    }
});
