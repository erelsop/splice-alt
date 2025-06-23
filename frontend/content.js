// Splice Alt - Content Script
// Provides visual feedback and integration with Splice website
// Network monitoring is now handled by webRequest API in background script

console.log('Splice Alt content script loaded - using webRequest API for network monitoring');

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

// Track processed samples to avoid duplicate notifications
const processedSamples = new Set();

// Listen for messages from background script
browserAPI.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.type) {
        case 'METADATA_CAPTURED':
            handleMetadataCaptured(message);
            break;
        case 'JSON_CREATED':
            handleJsonCreated(message);
            break;
        case 'TOGGLE_DEBUG_PANEL':
            toggleStatusPanel();
            break;
    }
});

function handleMetadataCaptured(message) {
    if (processedSamples.has(message.fileHash)) {
        return;
    }
    
    processedSamples.add(message.fileHash);
    console.log('Metadata captured for:', message.filename);
    
    // Show visual feedback
    showNotification(`🎵 Splice Alt: Metadata captured for ${message.filename}`, 'info');
}

function handleJsonCreated(message) {
    console.log('JSON file created:', message.jsonFilename);
    
    // Show visual feedback
    showNotification(`✅ Splice Alt: JSON metadata created for ${message.filename}`, 'success');
}

function showNotification(message, type = 'info') {
    // Remove any existing notifications first
    const existing = document.querySelectorAll('.splice-alt-notification');
    existing.forEach(el => el.remove());
    
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `splice-alt-notification splice-alt-${type}`;
    notification.textContent = message;
    
    // Add styles
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${type === 'success' ? '#4CAF50' : type === 'error' ? '#f44336' : '#2196F3'};
        color: white;
        padding: 12px 20px;
        border-radius: 8px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        font-size: 14px;
        font-weight: 500;
        box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        z-index: 10000;
        max-width: 350px;
        word-wrap: break-word;
        animation: slideIn 0.3s ease-out;
        border: none;
        backdrop-filter: blur(10px);
    `;
    
    // Add animation styles if not already present
    if (!document.getElementById('splice-alt-styles')) {
        const style = document.createElement('style');
        style.id = 'splice-alt-styles';
        style.textContent = `
            @keyframes slideIn {
                from {
                    transform: translateX(100%);
                    opacity: 0;
                }
                to {
                    transform: translateX(0);
                    opacity: 1;
                }
            }
            
            @keyframes slideOut {
                from {
                    transform: translateX(0);
                    opacity: 1;
                }
                to {
                    transform: translateX(100%);
                    opacity: 0;
                }
            }
            
            .splice-alt-notification:hover {
                opacity: 0.8;
                cursor: pointer;
            }
        `;
        document.head.appendChild(style);
    }
    
    document.body.appendChild(notification);
    
    // Make notification clickable to dismiss
    notification.addEventListener('click', () => {
        notification.style.animation = 'slideOut 0.3s ease-in';
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 300);
    });
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
        if (notification.parentNode) {
            notification.style.animation = 'slideOut 0.3s ease-in';
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
            }, 300);
        }
    }, 5000);
}

// Add visual indicators to sample cards
function addSampleIndicators() {
    // Look for sample elements (Splice uses various selectors)
    const selectors = [
        '[data-testid*="sample"]',
        '.sample-card',
        '.track-card',
        '[data-cy*="sample"]',
        '.SampleCard',
        '.TrackCard'
    ];
    
    const sampleElements = document.querySelectorAll(selectors.join(', '));
    
    sampleElements.forEach(element => {
        if (element.querySelector('.splice-alt-indicator')) {
            return; // Already processed
        }
        
        // Add indicator
        const indicator = document.createElement('div');
        indicator.className = 'splice-alt-indicator';
        indicator.innerHTML = '🎵';
        indicator.title = 'Splice Alt: Ready to organize samples';
        indicator.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            background: rgba(76, 175, 80, 0.9);
            color: white;
            border-radius: 50%;
            width: 28px;
            height: 28px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 14px;
            z-index: 1000;
            opacity: 0.9;
            box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            transition: all 0.2s ease;
        `;
        
        // Add hover effect
        indicator.addEventListener('mouseenter', () => {
            indicator.style.transform = 'scale(1.1)';
            indicator.style.opacity = '1';
        });
        
        indicator.addEventListener('mouseleave', () => {
            indicator.style.transform = 'scale(1)';
            indicator.style.opacity = '0.9';
        });
        
        // Make parent relative if needed
        const computedStyle = getComputedStyle(element);
        if (computedStyle.position === 'static') {
            element.style.position = 'relative';
        }
        
        element.appendChild(indicator);
    });
}

// Monitor for new sample cards being added to the page
const observer = new MutationObserver((mutations) => {
    let shouldUpdate = false;
    
    mutations.forEach((mutation) => {
        if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
            // Check if any added nodes contain sample elements
            mutation.addedNodes.forEach(node => {
                if (node.nodeType === Node.ELEMENT_NODE) {
                    const hassamples = node.querySelector && (
                        node.querySelector('[data-testid*="sample"]') ||
                        node.querySelector('.sample-card') ||
                        node.querySelector('.track-card')
                    );
                    if (hassamples || (node.className && typeof node.className === 'string' && (node.className.includes('sample') || node.className.includes('track')))) {
                        shouldUpdate = true;
                    }
                }
            });
        }
    });
    
    if (shouldUpdate) {
        // Add a small delay to ensure elements are fully rendered
        setTimeout(addSampleIndicators, 200);
    }
});

// Start observing when DOM is ready
function initializeContentScript() {
    addSampleIndicators();
    
    // Start observing for changes
    observer.observe(document.body, {
        childList: true,
        subtree: true
    });
    
    console.log('Splice Alt content script initialized');
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeContentScript);
} else {
    initializeContentScript();
}

// Clean up on page unload
window.addEventListener('beforeunload', () => {
    observer.disconnect();
});

// Add debug status panel (toggle with Ctrl+Shift+S)
let statusPanel = null;

function toggleStatusPanel() {
    if (statusPanel) {
        statusPanel.remove();
        statusPanel = null;
        return;
    }
    
    createStatusPanel();
}

function createStatusPanel() {
    statusPanel = document.createElement('div');
    statusPanel.id = 'splice-alt-status';
    statusPanel.innerHTML = `
        <div style="
            position: fixed;
            bottom: 20px;
            left: 20px;
            background: rgba(51, 51, 51, 0.95);
            color: white;
            padding: 15px;
            border-radius: 8px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 12px;
            z-index: 10001;
            min-width: 250px;
            max-width: 400px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.4);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.1);
        ">
            <div style="font-weight: bold; margin-bottom: 8px; color: #4CAF50;">
                🎵 Splice Alt Debug Panel
            </div>
            <div id="splice-alt-stats">Loading...</div>
            <button id="splice-alt-close" style="
                position: absolute;
                top: 8px;
                right: 8px;
                background: none;
                border: none;
                color: #ccc;
                cursor: pointer;
                font-size: 18px;
                width: 24px;
                height: 24px;
                display: flex;
                align-items: center;
                justify-content: center;
                border-radius: 4px;
                transition: all 0.2s ease;
            " onmouseover="this.style.background='rgba(255,255,255,0.1)'" 
               onmouseout="this.style.background='none'">×</button>
        </div>
    `;
    
    document.body.appendChild(statusPanel);
    
    // Close button functionality
    statusPanel.querySelector('#splice-alt-close').addEventListener('click', () => {
        toggleStatusPanel();
    });
    
    // Update stats
    updateStatusPanel();
    
    // Update stats every 2 seconds
    const updateInterval = setInterval(() => {
        if (statusPanel) {
            updateStatusPanel();
        } else {
            clearInterval(updateInterval);
        }
    }, 2000);
}

async function updateStatusPanel() {
    if (!statusPanel) return;
    
    try {
        const response = await browserAPI.runtime.sendMessage({ type: 'GET_STATS' });
        const statsDiv = statusPanel.querySelector('#splice-alt-stats');
        
        if (statsDiv) {
            statsDiv.innerHTML = `
                <div style="margin-bottom: 4px;">📊 Captured Metadata: ${response.capturedMetadata}</div>
                <div style="margin-bottom: 4px;">✅ Processed Samples: ${response.processedSamples}</div>
                <div style="margin-bottom: 4px;">🕐 Last Update: ${new Date().toLocaleTimeString()}</div>
                <div style="margin-bottom: 4px;">🔧 Network Monitoring: webRequest API</div>
                <div style="margin-top: 8px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.1); font-size: 10px; color: #aaa;">
                    Press Ctrl+Shift+S to toggle this panel
                </div>
            `;
        }
    } catch (error) {
        console.error('Failed to get stats:', error);
    }
}

// Keyboard shortcut to toggle debug panel
document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.shiftKey && event.key === 'S') {
        event.preventDefault();
        toggleStatusPanel();
    }
});

console.log('Splice Alt content script ready - Press Ctrl+Shift+S for debug panel'); 