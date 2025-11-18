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
  
  // Inject the bridge script into the page context
  const script = document.createElement('script');
  script.src = chrome.runtime.getURL('content/injected-bridge.js');
  script.onload = function() {
    this.remove();
  };
  (document.head || document.documentElement).appendChild(script);
  
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
