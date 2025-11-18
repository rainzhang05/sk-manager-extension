/**
 * Content Script for Feitian SK Manager Extension
 * 
 * This script is injected into web pages and provides a bridge API
 * for the web UI to communicate with the extension background worker.
 * It exposes the window.chromeBridge object.
 */

(function() {
  'use strict';
  
  console.log('[Content] Feitian SK Manager content script loaded');
  
  /**
   * Chrome Bridge API
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
    },
    
    /**
     * Check if the extension is connected to the native host
     * @returns {Promise<boolean>} True if connected
     */
    isConnected: async function() {
      try {
        const response = await this.send('ping');
        return response.status === 'ok';
      } catch (error) {
        return false;
      }
    },
    
    /**
     * Event handler for disconnection (can be overridden)
     */
    onDisconnect: null,
    
    /**
     * Get the version of the native host
     * @returns {Promise<string>} Version string
     */
    getVersion: async function() {
      const response = await this.send('getVersion');
      return response.result?.version || 'unknown';
    }
  };
  
  // Expose the bridge to the window object
  if (typeof window !== 'undefined') {
    window.chromeBridge = chromeBridge;
    console.log('[Content] chromeBridge API exposed to window');
    
    // Dispatch a custom event to notify the page that the bridge is ready
    const event = new CustomEvent('chromeBridgeReady', { detail: chromeBridge });
    window.dispatchEvent(event);
  }
  
  /**
   * Listen for messages from the page
   * This allows the page to communicate even if the bridge hasn't loaded yet
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
