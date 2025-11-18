/**
 * Injected Bridge Script
 * 
 * This script runs in the page context (not isolated like content scripts)
 * and creates window.chromeBridge that the web app can access.
 * It communicates with the content script via window.postMessage.
 */

(function() {
  'use strict';
  console.log('[Injected] Creating chromeBridge in page context');
  
  let requestIdCounter = 0;
  const pendingRequests = new Map();
  
  // Listen for responses from content script
  window.addEventListener('message', (event) => {
    if (event.source !== window) return;
    
    if (event.data.type === 'FEITIAN_SK_MANAGER_RESPONSE') {
      const { id, response } = event.data;
      const resolve = pendingRequests.get(id);
      if (resolve) {
        resolve(response);
        pendingRequests.delete(id);
      }
    }
  });
  
  window.chromeBridge = {
    send: function(command, params = {}) {
      return new Promise((resolve) => {
        const id = ++requestIdCounter;
        pendingRequests.set(id, resolve);
        
        window.postMessage({
          type: 'FEITIAN_SK_MANAGER_REQUEST',
          id,
          command,
          params
        }, '*');
      });
    },
    
    isConnected: async function() {
      try {
        const response = await this.send('ping');
        return response.status === 'ok';
      } catch (error) {
        return false;
      }
    },
    
    getVersion: async function() {
      const response = await this.send('getVersion');
      return response.result?.version || 'unknown';
    },
    
    onDisconnect: null
  };
  
  console.log('[Injected] chromeBridge created and exposed to window');
  window.dispatchEvent(new CustomEvent('chromeBridgeReady', { detail: window.chromeBridge }));
})();
