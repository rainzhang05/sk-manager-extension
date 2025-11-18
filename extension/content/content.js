/**
 * Content Script for Feitian SK Manager Extension
 * 
 * This script is injected into web pages and provides a bridge API
 * for the web UI to communicate with the extension background worker.
 * Uses script injection to bypass Manifest V3 isolated worlds.
 */

(function() {
  'use strict';
  
  console.log('[Content] Feitian SK Manager content script loaded');
  
  // Inject a script into the page context to create window.chromeBridge
  const script = document.createElement('script');
  script.textContent = `
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
  `;
  
  (document.head || document.documentElement).appendChild(script);
  script.remove();
  
  console.log('[Content] Script injected into page context');
  
  /**
   * Chrome Bridge API (for content script context)
   * Provides methods for the web page to communicate with the extension
   */
  const chromeBridge = {
    /**
     * Send a command to the native host
     * @param {string} command - The command name
     * @param {object} params - Command parameters
     * @returns {Promise} Promise that resolves with the response
     */
    send: function(command, params = {}) {
      return new Promise((resolve, reject) => {
        chrome.runtime.sendMessage(
          { command, params },
          (response) => {
            if (chrome.runtime.lastError) {
              reject({
                status: 'error',
                error: {
                  code: 'RUNTIME_ERROR',
                  message: chrome.runtime.lastError.message
                }
              });
              return;
            }
            
            if (response.status === 'error') {
              reject(response);
            } else {
              resolve(response);
            }
          }
        );
      });
    }
  };
  
  /**
   * Listen for messages from the page (injected script)
   * Forward them to the background service worker
   */
  window.addEventListener('message', (event) => {
    // Only accept messages from the same origin
    if (event.source !== window) {
      return;
    }
    
    if (event.data.type === 'FEITIAN_SK_MANAGER_REQUEST') {
      const { id, command, params } = event.data;
      
      chromeBridge.send(command, params)
        .then(response => {
          window.postMessage({
            type: 'FEITIAN_SK_MANAGER_RESPONSE',
            id,
            response
          }, '*');
        })
        .catch(error => {
          window.postMessage({
            type: 'FEITIAN_SK_MANAGER_RESPONSE',
            id,
            response: error
          }, '*');
        });
    }
  });
  
  console.log('[Content] Message listener initialized');
})();
