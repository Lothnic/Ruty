/**
 * Tauri v2 Store Plugin Wrapper
 * 
 * Uses the tauri-plugin-store v2 API correctly.
 * The store plugin works by:
 * 1. Loading a store by path (returns a resource ID)
 * 2. Using the rid for get/set operations
 * 3. Calling save to persist to disk
 */

// Helper to access invoke
const getInvoke = () => {
    if (window.__TAURI__?.core?.invoke) {
        return window.__TAURI__.core.invoke;
    }
    if (window.__TAURI__?.tauri?.invoke) {
        return window.__TAURI__.tauri.invoke;
    }
    throw new Error('Tauri API not found');
};

export class Store {
    constructor(path) {
        this.path = path;
        this.rid = null;
        this._loading = null;
    }

    async ensureLoaded() {
        // Prevent multiple simultaneous loads
        if (this._loading) {
            return this._loading;
        }

        if (this.rid !== null) {
            return Promise.resolve();
        }

        const invoke = getInvoke();

        console.log(`[Store] Loading store: ${this.path}`);

        this._loading = (async () => {
            try {
                // In tauri-plugin-store v2, 'load' returns the resource ID
                this.rid = await invoke('plugin:store|load', { path: this.path });
                console.log(`[Store] Loaded with RID: ${this.rid}`);
            } catch (e) {
                console.error(`[Store] Load failed:`, e);
                // Try to create a new store if load fails
                try {
                    // Some versions use 'create' for new stores
                    this.rid = await invoke('plugin:store|load', {
                        path: this.path,
                        autoSave: true
                    });
                    console.log(`[Store] Created new store with RID: ${this.rid}`);
                } catch (e2) {
                    console.error(`[Store] Create also failed:`, e2);
                    throw e;
                }
            } finally {
                this._loading = null;
            }
        })();

        return this._loading;
    }

    async set(key, value) {
        await this.ensureLoaded();
        const invoke = getInvoke();

        console.log(`[Store] Setting key: ${key}`, value);

        try {
            await invoke('plugin:store|set', {
                rid: this.rid,
                key: key,
                value: value
            });
            console.log(`[Store] Set successful`);
        } catch (e) {
            console.error(`[Store] Set failed:`, e);
            throw e;
        }
    }

    async get(key) {
        await this.ensureLoaded();
        const invoke = getInvoke();

        try {
            const value = await invoke('plugin:store|get', {
                rid: this.rid,
                key: key
            });
            console.log(`[Store] Get ${key}:`, value);
            return value;
        } catch (e) {
            console.error(`[Store] Get failed:`, e);
            return null;
        }
    }

    async save() {
        await this.ensureLoaded();
        const invoke = getInvoke();

        console.log(`[Store] Saving...`);

        try {
            await invoke('plugin:store|save', {
                rid: this.rid
            });
            console.log(`[Store] Save successful`);
        } catch (e) {
            console.error(`[Store] Save failed:`, e);
            throw e;
        }
    }

    async delete(key) {
        await this.ensureLoaded();
        const invoke = getInvoke();

        try {
            await invoke('plugin:store|delete', {
                rid: this.rid,
                key: key
            });
        } catch (e) {
            console.error(`[Store] Delete failed:`, e);
        }
    }

    async keys() {
        await this.ensureLoaded();
        const invoke = getInvoke();

        try {
            return await invoke('plugin:store|keys', {
                rid: this.rid
            });
        } catch (e) {
            console.error(`[Store] Keys failed:`, e);
            return [];
        }
    }

    async clear() {
        await this.ensureLoaded();
        const invoke = getInvoke();

        try {
            await invoke('plugin:store|clear', {
                rid: this.rid
            });
            await this.save();
        } catch (e) {
            console.error(`[Store] Clear failed:`, e);
        }
    }
}
