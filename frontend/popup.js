// Splice Alt - Popup Script
// Settings interface for the browser extension

// Browser compatibility layer
const browserAPI = (function() {
    if (typeof browser !== 'undefined') {
        return browser;
    } else if (typeof chrome !== 'undefined') {
        return chrome;
    } else {
        throw new Error('No browser API available');
    }
})();

document.addEventListener('DOMContentLoaded', async () => {
    console.log('Splice Alt popup loaded');
    
    // Load current configuration
    await loadConfiguration();
    
    // Load statistics
    await loadStatistics();
    
    // Set up event listeners
    setupEventListeners();
    
    // Auto-refresh stats every 3 seconds
    setInterval(loadStatistics, 3000);
});

async function loadConfiguration() {
    try {
        const response = await browserAPI.runtime.sendMessage({ type: 'GET_CONFIG' });
        
        // Update UI elements
        document.getElementById('enabled').checked = response.enabled;
        document.getElementById('autoCapture').checked = response.autoCapture;
        
        // Update status indicator
        updateStatusIndicator(response.enabled);
        
    } catch (error) {
        console.error('Failed to load configuration:', error);
        showError('Failed to load settings');
    }
}

async function loadStatistics() {
    try {
        const response = await browserAPI.runtime.sendMessage({ type: 'GET_STATS' });
        
        // Update stats display
        document.getElementById('capturedCount').textContent = response.capturedMetadata || 0;
        document.getElementById('processedCount').textContent = response.processedSamples || 0;
        document.getElementById('lastUpdate').textContent = new Date().toLocaleTimeString();
        
    } catch (error) {
        console.error('Failed to load statistics:', error);
        document.getElementById('capturedCount').textContent = 'Error';
        document.getElementById('processedCount').textContent = 'Error';
    }
}

function setupEventListeners() {
    // Configuration toggles
    document.getElementById('enabled').addEventListener('change', async (e) => {
        await updateConfiguration({ enabled: e.target.checked });
        updateStatusIndicator(e.target.checked);
    });
    
    document.getElementById('autoCapture').addEventListener('change', async (e) => {
        await updateConfiguration({ autoCapture: e.target.checked });
    });
    
    // Action buttons
    document.getElementById('clearMetadata').addEventListener('click', clearMetadata);
    document.getElementById('testConnection').addEventListener('click', testConnection);
    document.getElementById('openDebugPanel').addEventListener('click', openDebugPanel);
}

async function updateConfiguration(config) {
    try {
        await browserAPI.runtime.sendMessage({ 
            type: 'UPDATE_CONFIG', 
            config: config 
        });
        
        showSuccess('Settings updated');
        
    } catch (error) {
        console.error('Failed to update configuration:', error);
        showError('Failed to update settings');
    }
}

async function clearMetadata() {
    try {
        await browserAPI.runtime.sendMessage({ type: 'CLEAR_METADATA' });
        showSuccess('Metadata cache cleared');
        await loadStatistics();
        
    } catch (error) {
        console.error('Failed to clear metadata:', error);
        showError('Failed to clear metadata');
    }
}

async function testConnection() {
    const button = document.getElementById('testConnection');
    const originalText = button.textContent;
    
    button.textContent = 'Testing...';
    button.disabled = true;
    
    try {
        // Test communication with background script
        const response = await browserAPI.runtime.sendMessage({ type: 'GET_STATS' });
        
        if (response && typeof response.capturedMetadata !== 'undefined') {
            showSuccess('✅ Extension is working correctly');
        } else {
            showError('❌ Extension communication failed');
        }
        
    } catch (error) {
        console.error('Connection test failed:', error);
        showError('❌ Extension communication failed');
    } finally {
        button.textContent = originalText;
        button.disabled = false;
    }
}

async function openDebugPanel() {
    try {
        // Get the active tab
        const tabs = await browserAPI.tabs.query({ active: true, currentWindow: true });
        
        if (tabs[0] && tabs[0].url.includes('splice.com')) {
            // Send message to content script to open debug panel
            await browserAPI.tabs.sendMessage(tabs[0].id, { type: 'TOGGLE_DEBUG_PANEL' });
            showSuccess('Debug panel toggled (or press Ctrl+Shift+S on Splice)');
        } else {
            showError('Please navigate to splice.com first');
        }
        
    } catch (error) {
        console.error('Failed to open debug panel:', error);
        showError('Debug panel requires Splice.com to be open');
    }
}

function updateStatusIndicator(enabled) {
    const indicator = document.getElementById('statusIndicator');
    const status = document.getElementById('statusText');
    
    if (enabled) {
        indicator.className = 'status-indicator active';
        status.textContent = 'Active';
    } else {
        indicator.className = 'status-indicator inactive';
        status.textContent = 'Disabled';
    }
}

function showSuccess(message) {
    showMessage(message, 'success');
}

function showError(message) {
    showMessage(message, 'error');
}

function showMessage(message, type) {
    // Remove any existing messages
    const existing = document.querySelectorAll('.message');
    existing.forEach(el => el.remove());
    
    // Create message element
    const messageEl = document.createElement('div');
    messageEl.className = `message ${type}`;
    messageEl.textContent = message;
    
    // Insert at the top of the popup
    const container = document.querySelector('.popup-container');
    container.insertBefore(messageEl, container.firstChild);
    
    // Auto-remove after 3 seconds
    setTimeout(() => {
        if (messageEl.parentNode) {
            messageEl.remove();
        }
    }, 3000);
} 