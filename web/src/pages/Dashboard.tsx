import { DeviceList } from '../components'
import { useState, useEffect } from 'react'
import { connectionManager, ConnectionState } from '../services/ConnectionManager'
import '../styles/Dashboard.css'

export default function Dashboard() {
  const [connectionState, setConnectionState] = useState<ConnectionState>(
    connectionManager.getState()
  )

  useEffect(() => {
    console.log('[Dashboard] Mounting and subscribing to connection manager')

    // Subscribe to connection state updates
    const unsubscribe = connectionManager.subscribe((state) => {
      console.log('[Dashboard] Received state update:', state)
      setConnectionState(state)
    })

    // Request an immediate refresh
    connectionManager.refreshConnections()

    // Cleanup: unsubscribe when component unmounts
    return () => {
      console.log('[Dashboard] Unmounting but connection manager continues running')
      unsubscribe()
    }
  }, [])

  const {
    extensionConnected,
    nativeHostConnected,
    version,
    checking,
  } = connectionState

  return (
    <div className="page">
      <div className="page-header">
        <h1>Dashboard</h1>
        <p className="page-description">
          Overview of your Feitian security keys and connection status
        </p>
      </div>

      <div className="dashboard-grid">
        <div className="status-section">
          <h2>Connection Status</h2>
          <div className="status-cards">
            <div className={`status-card ${extensionConnected ? 'connected' : 'disconnected'}`}>
              <div className="status-header">
                <span className="status-label">Chrome Extension</span>
              </div>
              <div className="status-value">
                {checking && !extensionConnected ? 'Checking...' : extensionConnected ? 'Connected' : 'Not Connected'}
              </div>
              {!extensionConnected && !checking && (
                <p className="status-help">
                  Please install and enable the Feitian SK Manager extension. Make sure you've loaded the unpacked extension from chrome://extensions/
                </p>
              )}
            </div>

            <div className={`status-card ${nativeHostConnected ? 'connected' : 'disconnected'}`}>
              <div className="status-header">
                <span className="status-label">Native Host</span>
              </div>
              <div className="status-value">
                {checking && !nativeHostConnected ? 'Checking...' : nativeHostConnected ? `Connected (v${version})` : 'Not Connected'}
              </div>
              {!nativeHostConnected && extensionConnected && !checking && (
                <p className="status-help">
                  Native host not responding. Please ensure: 1) Native host is built (cargo build --release), 2) Manifest is installed (run setup-native-host.sh), 3) Chrome is fully restarted.
                </p>
              )}
            </div>
          </div>
        </div>

        <div className="devices-section">
          <DeviceList onRefresh={() => connectionManager.refreshConnections()} />
        </div>
      </div>
    </div>
  )
}
