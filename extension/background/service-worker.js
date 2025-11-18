/**
 * Background Service Worker for Feitian SK Manager Extension
 * 
 * This service worker acts as a bridge between the web UI and the native host.
 * It handles:
 * - Native messaging connection to the Rust native host
 * - Request/response queue management with ID matching
 * - Message validation and error handling
 * - Reconnection logic on failure
 */

const NATIVE_HOST_NAME = 'com.feitian.sk_manager';

let nativePort = null;
let requestQueue = new Map(); // Map of request ID to callback
let requestIdCounter = 0;
let isConnected = false;

/**
 * Connect to the native messaging host
 */
function connectToNativeHost() {
  console.log('[Background] Attempting to connect to native host:', NATIVE_HOST_NAME);
  
  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST_NAME);
    
    nativePort.onMessage.addListener(handleNativeMessage);
    nativePort.onDisconnect.addListener(handleNativeDisconnect);
    
    isConnected = true;
    console.log('[Background] Connected to native host');
  } catch (error) {
    console.error('[Background] Failed to connect to native host:', error);
    isConnected = false;
  }
}

/**
 * Handle messages from the native host
 */
function handleNativeMessage(message) {
  console.log('[Background] Received from native host:', message);
  
  if (!message || typeof message.id === 'undefined') {
    console.error('[Background] Invalid message format from native host:', message);
    return;
  }
  
  const callback = requestQueue.get(message.id);
  if (callback) {
    callback(message);
    requestQueue.delete(message.id);
  } else {
    console.warn('[Background] No callback found for message ID:', message.id);
  }
}

/**
 * Handle native host disconnection
 */
function handleNativeDisconnect() {
  console.log('[Background] Native host disconnected');
  isConnected = false;
  
  if (chrome.runtime.lastError) {
    console.error('[Background] Disconnect error:', chrome.runtime.lastError.message);
  }
  
  // Clear pending requests with error
  requestQueue.forEach((callback) => {
    callback({
      status: 'error',
      error: {
        code: 'DISCONNECTED',
        message: 'Native host disconnected'
      }
    });
  });
  requestQueue.clear();
  
  // Attempt to reconnect after 5 seconds
  setTimeout(() => {
    console.log('[Background] Attempting to reconnect...');
    connectToNativeHost();
  }, 5000);
}

/**
 * Send a message to the native host
 */
function sendToNativeHost(command, params = {}) {
  return new Promise((resolve, reject) => {
    if (!isConnected || !nativePort) {
      reject({
        status: 'error',
        error: {
          code: 'NOT_CONNECTED',
          message: 'Not connected to native host'
        }
      });
      return;
    }
    
    const id = ++requestIdCounter;
    const message = { id, command, params };
    
    requestQueue.set(id, (response) => {
      if (response.status === 'error') {
        reject(response);
      } else {
        resolve(response);
      }
    });
    
    try {
      nativePort.postMessage(message);
      console.log('[Background] Sent to native host:', message);
    } catch (error) {
      requestQueue.delete(id);
      reject({
        status: 'error',
        error: {
          code: 'SEND_FAILED',
          message: error.message
        }
      });
    }
  });
}

/**
 * Handle messages from content scripts
 */
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log('[Background] Received from content script:', request);
  
  if (!request || !request.command) {
    sendResponse({
      status: 'error',
      error: {
        code: 'INVALID_REQUEST',
        message: 'Invalid request format'
      }
    });
    return true;
  }
  
  sendToNativeHost(request.command, request.params)
    .then(response => sendResponse(response))
    .catch(error => sendResponse(error));
  
  return true; // Indicate async response
});

/**
 * Initialize on install/startup
 */
chrome.runtime.onInstalled.addListener(() => {
  console.log('[Background] Extension installed/updated');
  connectToNativeHost();
});

chrome.runtime.onStartup.addListener(() => {
  console.log('[Background] Extension started');
  connectToNativeHost();
});

// Connect on load
connectToNativeHost();

console.log('[Background] Service worker initialized');
