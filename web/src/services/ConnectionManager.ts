/**
 * Global connection manager that maintains state across page navigations
 * This acts as a singleton service to persist connection status
 */

class ConnectionManager {
  private static instance: ConnectionManager
  private pollingInterval: number | null = null
  private connectionCheckInterval: number | null = null
  private callbacks: Set<(state: ConnectionState) => void> = new Set()

  private state: ConnectionState = {
    extensionConnected: false,
    nativeHostConnected: false,
    version: 'unknown',
    checking: true,
    deviceId: null,
    deviceConnected: false,
  }

  private constructor() {
    // Private constructor for singleton
    this.initializeFromStorage()
    this.startMonitoring()
  }

  public static getInstance(): ConnectionManager {
    if (!ConnectionManager.instance) {
      ConnectionManager.instance = new ConnectionManager()
    }
    return ConnectionManager.instance
  }

  private initializeFromStorage() {
    // Load persisted state from sessionStorage
    const savedDeviceId = sessionStorage.getItem('connectedDeviceId')
    const savedDeviceConnected = sessionStorage.getItem('deviceConnected') === 'true'

    if (savedDeviceId) {
      this.state.deviceId = savedDeviceId
      this.state.deviceConnected = savedDeviceConnected
    }
  }

  private startMonitoring() {
    // Check connections every 5 seconds
    this.connectionCheckInterval = window.setInterval(() => {
      this.checkConnections()
    }, 5000)

    // Initial check
    this.checkConnections()

    // Listen for device events
    window.addEventListener('device-connected', this.handleDeviceConnected.bind(this))
    window.addEventListener('device-disconnected', this.handleDeviceDisconnected.bind(this))
  }

  private async checkConnections() {
    if (!window.chromeBridge) {
      this.updateState({
        extensionConnected: false,
        nativeHostConnected: false,
        checking: false,
      })
      return
    }

    try {
      this.updateState({ checking: true, extensionConnected: true })

      const connected = await window.chromeBridge.isConnected()
      this.updateState({ nativeHostConnected: connected })

      if (connected) {
        const version = await window.chromeBridge.getVersion()
        this.updateState({ version })
      }

      this.updateState({ checking: false })
    } catch (err) {
      console.error('[ConnectionManager] Error checking connections:', err)
      this.updateState({
        nativeHostConnected: false,
        checking: false,
      })
    }
  }

  private handleDeviceConnected(event: Event) {
    const customEvent = event as CustomEvent
    const deviceId = customEvent.detail.deviceId

    this.updateState({
      deviceId,
      deviceConnected: true,
    })

    // Persist to sessionStorage
    sessionStorage.setItem('connectedDeviceId', deviceId)
    sessionStorage.setItem('deviceConnected', 'true')
  }

  private handleDeviceDisconnected() {
    this.updateState({
      deviceId: null,
      deviceConnected: false,
    })

    // Clear from sessionStorage
    sessionStorage.removeItem('connectedDeviceId')
    sessionStorage.removeItem('deviceConnected')
  }

  private updateState(newState: Partial<ConnectionState>) {
    this.state = { ...this.state, ...newState }
    this.notifySubscribers()
  }

  private notifySubscribers() {
    this.callbacks.forEach(callback => {
      try {
        callback(this.state)
      } catch (err) {
        console.error('[ConnectionManager] Error in subscriber callback:', err)
      }
    })
  }

  public subscribe(callback: (state: ConnectionState) => void): () => void {
    this.callbacks.add(callback)

    // Immediately call with current state
    callback(this.state)

    // Return unsubscribe function
    return () => {
      this.callbacks.delete(callback)
    }
  }

  public getState(): ConnectionState {
    return { ...this.state }
  }

  public async refreshConnections() {
    await this.checkConnections()
  }

  public destroy() {
    if (this.connectionCheckInterval) {
      clearInterval(this.connectionCheckInterval)
      this.connectionCheckInterval = null
    }

    if (this.pollingInterval) {
      clearInterval(this.pollingInterval)
      this.pollingInterval = null
    }

    window.removeEventListener('device-connected', this.handleDeviceConnected.bind(this))
    window.removeEventListener('device-disconnected', this.handleDeviceDisconnected.bind(this))
  }
}

export interface ConnectionState {
  extensionConnected: boolean
  nativeHostConnected: boolean
  version: string
  checking: boolean
  deviceId: string | null
  deviceConnected: boolean
}

// Export singleton instance
export const connectionManager = ConnectionManager.getInstance()
