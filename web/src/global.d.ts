// Global type declarations for Chrome Extension Bridge
declare global {
  interface Window {
    chromeBridge?: {
      send: (command: string, params?: Record<string, unknown>) => Promise<{
        status: string
        result?: unknown
        error?: { code: string; message: string }
      }>
      isConnected: () => Promise<boolean>
      getVersion: () => Promise<string>
      onDisconnect: (() => void) | null
    }
  }
}

export {}
