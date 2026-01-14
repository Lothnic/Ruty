/**
 * Ruty Result List UI
 * Raycast-style keyboard-navigable result list
 */

import { TypeIcons } from './commands.js';

class ResultList {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.results = [];
        this.selectedIndex = 0;
        this.onSelect = null;
        this.visible = false;

        // Debounce timer for updates
        this.updateTimer = null;
    }

    /**
     * Show results in the list
     */
    show(results) {
        this.results = results || [];
        this.selectedIndex = 0;

        if (this.results.length === 0) {
            this.hide();
            return;
        }

        this.visible = true;
        this.render();
        this.container.classList.remove('hidden');
    }

    /**
     * Hide the result list
     */
    hide() {
        this.visible = false;
        this.results = [];
        this.container.classList.add('hidden');
        this.container.innerHTML = '';
    }

    /**
     * Render the result list
     */
    render() {
        // Group results by type
        const grouped = this.groupResults();

        let html = '';
        let globalIndex = 0;

        for (const [type, items] of Object.entries(grouped)) {
            // Category header
            html += `<div class="result-category">${this.getCategoryLabel(type)}</div>`;

            // Items
            for (const item of items) {
                const isSelected = globalIndex === this.selectedIndex;
                html += this.renderItem(item, globalIndex, isSelected);
                globalIndex++;
            }
        }

        this.container.innerHTML = html;

        // Add click handlers
        this.container.querySelectorAll('.result-item').forEach((el, idx) => {
            el.addEventListener('click', () => this.selectAndExecute(idx));
            el.addEventListener('mouseenter', () => this.setSelected(idx));
        });
    }

    /**
     * Render a single result item
     */
    renderItem(item, index, isSelected) {
        const selectedClass = isSelected ? 'selected' : '';
        const iconHtml = item.icon || TypeIcons[item.type] || TypeIcons.action;
        const shortcutHtml = item.shortcut
            ? `<kbd class="result-shortcut">${item.shortcut}</kbd>`
            : '';

        return `
            <div class="result-item ${selectedClass}" data-index="${index}">
                <div class="result-icon">${iconHtml}</div>
                <div class="result-content">
                    <div class="result-title">${this.escapeHtml(item.title)}</div>
                    ${item.subtitle ? `<div class="result-subtitle">${this.escapeHtml(item.subtitle)}</div>` : ''}
                </div>
                ${shortcutHtml}
            </div>
        `;
    }

    /**
     * Group results by type
     */
    groupResults() {
        const groups = {};
        for (const item of this.results) {
            const type = item.type || 'action';
            if (!groups[type]) groups[type] = [];
            groups[type].push(item);
        }
        return groups;
    }

    /**
     * Get human-readable category label
     */
    getCategoryLabel(type) {
        const labels = {
            ai: 'AI Assistant',
            app: 'Applications',
            file: 'Files',
            system: 'System',
            clipboard: 'Clipboard',
            action: 'Actions',
        };
        return labels[type] || type.charAt(0).toUpperCase() + type.slice(1);
    }

    /**
     * Handle keyboard navigation
     */
    handleKeyDown(e) {
        if (!this.visible || this.results.length === 0) return false;

        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                this.moveSelection(1);
                return true;

            case 'ArrowUp':
                e.preventDefault();
                this.moveSelection(-1);
                return true;

            case 'Tab':
                e.preventDefault();
                this.moveSelection(e.shiftKey ? -1 : 1);
                return true;

            case 'Enter':
                if (!e.shiftKey) {
                    e.preventDefault();
                    this.executeSelected();
                    return true;
                }
                break;
        }

        return false;
    }

    /**
     * Move selection up/down
     */
    moveSelection(delta) {
        const newIndex = this.selectedIndex + delta;
        if (newIndex >= 0 && newIndex < this.results.length) {
            this.setSelected(newIndex);
        }
    }

    /**
     * Set the selected item
     */
    setSelected(index) {
        this.selectedIndex = index;

        // Update visual selection
        this.container.querySelectorAll('.result-item').forEach((el, idx) => {
            el.classList.toggle('selected', idx === index);
        });

        // Scroll into view
        const selectedEl = this.container.querySelector('.result-item.selected');
        if (selectedEl) {
            selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
    }

    /**
     * Execute the selected item
     */
    async executeSelected() {
        if (this.results.length === 0) return null;

        const item = this.results[this.selectedIndex];
        if (item && item.action && this.onSelect) {
            const result = await item.action();
            this.onSelect(result, item);
            return result;
        }
        return null;
    }

    /**
     * Select by index and execute
     */
    async selectAndExecute(index) {
        this.setSelected(index);
        return this.executeSelected();
    }

    /**
     * Get selected item
     */
    getSelected() {
        return this.results[this.selectedIndex] || null;
    }

    /**
     * Check if list is visible
     */
    isVisible() {
        return this.visible && this.results.length > 0;
    }

    /**
     * Escape HTML to prevent XSS
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Export singleton
export const resultList = new ResultList('results');
