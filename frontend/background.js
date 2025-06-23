// Splice Alt - Background Service Worker
// Uses webRequest API to capture Splice API responses directly

console.log('Splice Alt extension loaded');

// Store for captured metadata
const capturedMetadata = new Map();
const processedSamples = new Set();

// Configuration - will be loaded from storage
let config = {
    enabled: true,
    autoCapture: true
};

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

// Set up webRequest listener to capture API responses
browserAPI.webRequest.onBeforeRequest.addListener(
    function(details) {
        if (details.method === 'GET' && details.url.includes('api.splice.com')) {
            console.log('Splice Alt: API request detected:', details.url);
        }
    },
    {
        urls: ["*://api.splice.com/*", "*://*.api.splice.com/*"]
    },
    ["requestBody"]
);

// Use response filtering to capture actual response data (Firefox feature)
if (browserAPI.webRequest.filterResponseData) {
    browserAPI.webRequest.onBeforeRequest.addListener(
        function(details) {
            if (!config.enabled || !config.autoCapture) return;
            
            if (details.url.includes('api.splice.com') && (
                details.url.includes('/v2/premium/samples/') || 
                details.url.includes('/www/me/premium')
            )) {
                console.log('Splice Alt: Filtering response for:', details.url);
                
                const filter = browserAPI.webRequest.filterResponseData(details.requestId);
                const decoder = new TextDecoder("utf-8");
                let responseData = '';
                
                filter.ondata = event => {
                    const str = decoder.decode(event.data, {stream: true});
                    responseData += str;
                    filter.write(event.data);
                };
                
                filter.onstop = event => {
                    try {
                        if (responseData.trim() === '') {
                            console.log('Splice Alt: Empty response data, skipping');
                            filter.disconnect();
                            return;
                        }
                        
                        const jsonData = JSON.parse(responseData);
                        console.log('Splice Alt: Captured API response:', details.url);
                        console.log('Splice Alt: Response structure:', Object.keys(jsonData));
                        processApiResponse(jsonData, details.url);
                    } catch (error) {
                        console.error('Splice Alt: Failed to parse API response:', error);
                        console.error('Splice Alt: Raw response data length:', responseData.length);
                        console.error('Splice Alt: Response preview:', responseData.substring(0, 200));
                    }
                    filter.disconnect();
                };
            }
        },
        {
            urls: ["*://api.splice.com/*", "*://*.api.splice.com/*"]
        },
        ["blocking"]
    );
} else {
    console.log('Splice Alt: Response filtering not available, falling back to request detection only');
    
    // Fallback: just detect requests and try to make our own
    browserAPI.webRequest.onCompleted.addListener(
        async function(details) {
            if (!config.enabled || !config.autoCapture) return;
            
            if (details.statusCode === 200 && details.url.includes('api.splice.com')) {
                console.log('Splice Alt: API response completed:', details.url);
                
                // Try to make our own request to get the data
                if (details.url.includes('/v2/premium/samples/')) {
                    console.log('Splice Alt: Attempting to fetch sample data:', details.url);
                    await fetchSampleData(details.url);
                }
            }
        },
        {
            urls: ["*://api.splice.com/*", "*://*.api.splice.com/*"]
        }
    );
}

function processApiResponse(data, url) {
    console.log('Splice Alt: Processing API response from:', url);
    console.log('Splice Alt: Response keys:', Object.keys(data));
    
    // Handle individual sample responses matching your example structure
    if (url.includes('/v2/premium/samples/')) {
        console.log('Splice Alt: Individual sample API response detected');
        
        // Check for the exact structure from your example
        if (data.sample_meta_data && data.sample) {
            const sampleIdMatch = url.match(/\/v2\/premium\/samples\/([^/?]+)/);
            if (sampleIdMatch) {
                const sampleId = sampleIdMatch[1];
                console.log('Splice Alt: Processing sample with structure:', {
                    filename: data.sample_meta_data.filename,
                    file_hash: data.sample_meta_data.file_hash,
                    pack_name: data.sample_meta_data.pack?.name,
                    bpm: data.sample_meta_data.bpm,
                    key: data.sample_meta_data.audio_key,
                    tags: data.sample_meta_data.tags
                });
                
                processCapturedMetadata(data, sampleId);
                
                // Notify content script about successful capture
                notifyContentScript('METADATA_CAPTURED', {
                    filename: data.sample_meta_data.filename,
                    fileHash: data.sample_meta_data.file_hash
                });
            }
        } else {
            console.log('Splice Alt: Sample response missing expected structure');
            console.log('Splice Alt: Has sample_meta_data:', !!data.sample_meta_data);
            console.log('Splice Alt: Has sample:', !!data.sample);
        }
    }
    // Handle bulk premium responses
    else if (url.includes('/www/me/premium') && data.samples) {
        console.log('Splice Alt: Processing bulk response with', data.samples.length, 'samples');
        data.samples.forEach((sample, index) => {
            if (sample.sample_meta_data) {
                const sampleId = sample.sample_meta_data.file_hash || `bulk_${index}`;
                console.log('Splice Alt: Processing bulk sample:', sample.sample_meta_data.filename);
                processCapturedMetadata(sample, sampleId);
            }
        });
    } else {
        console.log('Splice Alt: Unrecognized API response structure for:', url);
    }
}

async function notifyContentScript(type, data) {
    try {
        const tabs = await browserAPI.tabs.query({ url: "*://splice.com/*" });
        for (const tab of tabs) {
            browserAPI.tabs.sendMessage(tab.id, {
                type: type,
                ...data
            }).catch(() => {}); // Ignore errors if content script not ready
        }
    } catch (error) {
        console.log('Could not notify content script:', error);
    }
}

async function fetchSampleData(url) {
    try {
        console.log('Splice Alt: Making fetch request to:', url);
        const response = await fetch(url);
        if (response.ok) {
            const data = await response.json();
            processApiResponse(data, url);
        } else {
            console.log('Splice Alt: Fetch failed with status:', response.status);
        }
    } catch (error) {
        console.error('Splice Alt: Fetch error:', error);
    }
}

// Load configuration on startup
browserAPI.runtime.onStartup.addListener(loadConfiguration);
browserAPI.runtime.onInstalled.addListener(loadConfiguration);

async function loadConfiguration() {
    try {
        const result = await browserAPI.storage.sync.get({
            enabled: true,
            autoCapture: true
        });
        config = result;
        console.log('Splice Alt configuration loaded:', config);
    } catch (error) {
        console.error('Failed to load configuration:', error);
    }
}

function processCapturedMetadata(sampleData, sampleId) {
    if (!sampleData.sample_meta_data && !sampleData.sample) {
        console.error('Invalid sample data structure - missing sample_meta_data and sample');
        console.error('Available keys:', Object.keys(sampleData));
        return;
    }
    
    // Extract filename from different possible locations
    let filename = null;
    let fileHash = null;
    
    if (sampleData.sample_meta_data) {
        filename = sampleData.sample_meta_data.filename;
        fileHash = sampleData.sample_meta_data.file_hash;
    } else if (sampleData.sample) {
        filename = sampleData.sample.filename || sampleData.sample.path?.split('/').pop();
        fileHash = sampleData.sample.file_hash;
    }
    
    if (!filename) {
        console.error('Could not extract filename from sample data');
        console.error('Sample data structure:', sampleData);
        return;
    }
    
    console.log('Processing captured metadata for:', filename);
    
    // Store metadata using multiple keys for better matching
    capturedMetadata.set(filename, sampleData);
    if (fileHash) {
        capturedMetadata.set(fileHash, sampleData);
    }
    capturedMetadata.set(sampleId, sampleData);
    
    // Also store with base filename (without extension)
    const baseFilename = filename.replace(/\.wav$/, '');
    capturedMetadata.set(baseFilename, sampleData);
    
    console.log('Stored metadata for keys:', [filename, fileHash, sampleId, baseFilename].filter(Boolean));
}

// Listen for downloads and create JSON files
browserAPI.downloads.onCreated.addListener(async (downloadItem) => {
    if (!config.enabled || !config.autoCapture) return;
    
    // Check if this is a WAV file from Splice
    if (!downloadItem.filename.endsWith('.wav') || 
        !downloadItem.url.includes('splice')) {
        return;
    }
    
    console.log('Splice WAV download detected:', downloadItem.filename);
    
    // Extract just the filename without path
    const filename = downloadItem.filename.split('/').pop();
    const baseFilename = filename.replace(/\.wav$/, '');
    
    console.log('Looking for metadata with keys:', [filename, baseFilename]);
    
    // Try to find matching metadata
    let metadata = capturedMetadata.get(filename) || 
                   capturedMetadata.get(baseFilename);
    
    // If not found, try to match by partial filename
    if (!metadata) {
        for (let [key, data] of capturedMetadata.entries()) {
            if (typeof key === 'string' && (
                key.includes(baseFilename) || 
                baseFilename.includes(key) ||
                (data.sample_meta_data && data.sample_meta_data.filename === filename)
            )) {
                metadata = data;
                console.log('Found metadata by partial match:', key);
                break;
            }
        }
    }
    
    if (!metadata) {
        console.log('No metadata found for:', filename);
        console.log('Available metadata keys:', Array.from(capturedMetadata.keys()));
        
        // Wait a bit and try again in case metadata arrives after download starts
        setTimeout(async () => {
            let delayedMetadata = capturedMetadata.get(filename) || 
                                  capturedMetadata.get(baseFilename);
            
            if (delayedMetadata) {
                console.log('Found delayed metadata for:', filename);
                await createJsonFile(downloadItem, delayedMetadata);
            } else {
                console.log('Still no metadata found after delay for:', filename);
            }
        }, 2000);
        
        return;
    }
    
    console.log('Creating JSON metadata file for:', filename);
    await createJsonFile(downloadItem, metadata);
});

async function createJsonFile(downloadItem, metadata) {
    try {
        // Create the JSON content
        const jsonContent = JSON.stringify(metadata, null, 2);
        
        // Create the JSON filename - extract just the filename, not the full path
        const wavFilename = downloadItem.filename.split('/').pop();
        const jsonFilename = wavFilename.replace(/\.wav$/, '.json');
        
        // Create a blob with the JSON content
        const blob = new Blob([jsonContent], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        // Download the JSON file to the same directory as the WAV
        const downloadOptions = {
            url: url,
            filename: jsonFilename,
            saveAs: false,
            conflictAction: 'overwrite'
        };
        
        console.log('Downloading JSON file:', downloadOptions);
        
        const downloadId = await browserAPI.downloads.download(downloadOptions);
        console.log('JSON metadata file created with ID:', downloadId);
        
        // Clean up the blob URL
        setTimeout(() => {
            URL.revokeObjectURL(url);
        }, 5000);
        
        // Notify content script
        try {
            const tabs = await browserAPI.tabs.query({ url: "*://splice.com/*" });
            for (const tab of tabs) {
                browserAPI.tabs.sendMessage(tab.id, {
                    type: 'JSON_CREATED',
                    filename: downloadItem.filename,
                    jsonFilename: jsonFilename
                }).catch(() => {}); // Ignore errors if content script not ready
            }
        } catch (error) {
            console.log('Could not notify content script:', error);
        }
        
    } catch (error) {
        console.error('Failed to create JSON file:', error);
    }
}

// Handle messages from popup and content scripts
browserAPI.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.type) {
        case 'GET_CONFIG':
            sendResponse(config);
            break;
            
        case 'UPDATE_CONFIG':
            updateConfiguration(message.config);
            sendResponse({ success: true });
            break;
            
        case 'GET_STATS':
            sendResponse({
                capturedMetadata: capturedMetadata.size,
                processedSamples: processedSamples.size
            });
            break;
            
        case 'CLEAR_METADATA':
            capturedMetadata.clear();
            processedSamples.clear();
            sendResponse({ success: true });
            break;
            
        case 'METADATA_INTERCEPTED':
            // Legacy handler for content script interception (now using webRequest API)
            console.log('Received legacy metadata from content script:', message.url);
            sendResponse({ success: true });
            break;
            
        default:
            sendResponse({ error: 'Unknown message type' });
    }
    
    // Return true to indicate we'll send a response asynchronously
    return true;
});

async function updateConfiguration(newConfig) {
    config = { ...config, ...newConfig };
    await browserAPI.storage.sync.set(config);
    console.log('Configuration updated:', config);
}

// Clean up old metadata periodically (keep last 50 samples)
setInterval(() => {
    if (capturedMetadata.size > 50) {
        // Convert to array and keep most recent entries
        const entries = Array.from(capturedMetadata.entries());
        capturedMetadata.clear();
        
        // Keep the last 30 entries
        entries.slice(-30).forEach(([key, value]) => {
            capturedMetadata.set(key, value);
        });
        
        console.log('Cleaned up old metadata, kept', capturedMetadata.size, 'entries');
    }
}, 60000); // Check every minute

console.log('Splice Alt background script initialized'); 