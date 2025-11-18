import { DeviceList } from '../components'
import { useState, useEffect } from 'react'
import '../styles/Dashboard.css'

export default function Dashboard() {
  const [extensionConnected, setExtensionConnected] = useState(false)
  const [nativeHostConnected, setNativeHostConnected] = useState(false)
  const [version, setVersion] = useState<string>('unknown')
  const [checking, setChecking] = useState(true)

  useEffect(() => {
    // Wait for chromeBridge to be available
    const waitForBridge = () => {
      if (window.chromeBridge) {
        checkConnections()
      } else {
        // Retry after a short delay
        setTimeout(waitForBridge, 100)
      }
    }
    
    waitForBridge()
    
    // Also listen for the chromeBridgeReady event
    const handleBridgeReady = () => {
      checkConnections()
    }
    
    window.addEventListener('chromeBridgeReady', handleBridgeReady)
    
    // Set up interval to check connections live every 3 seconds
    const interval = setInterval(checkConnections, 3000)
    
    return () => {
      window.removeEventListener('chromeBridgeReady', handleBridgeReady)
      clearInterval(interval)
    }
  }, [])

  const checkConnections = async () => {
    setChecking(true)
    
    // Check extension
    if (window.chromeBridge) {
      setExtensionConnected(true)

      // Check native host
      try {
        const connected = await window.chromeBridge.isConnected()
        setNativeHostConnected(connected)

        if (connected) {
          const ver = await window.chromeBridge.getVersion()
          setVersion(ver)
        }
      } catch (err) {
        console.error('Error checking native host connection:', err)
        setNativeHostConnected(false)
      }
    } else {
      setExtensionConnected(false)
      setNativeHostConnected(false)
    }
    
    setChecking(false)
  }

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
          <DeviceList onRefresh={checkConnections} />
        </div>
      </div>
    </div>
  )
}
