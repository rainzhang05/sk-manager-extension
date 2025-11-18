import { DeviceList } from '../components'
import { useState, useEffect } from 'react'
import '../styles/Dashboard.css'

export default function Dashboard() {
  const [extensionConnected, setExtensionConnected] = useState(false)
  const [nativeHostConnected, setNativeHostConnected] = useState(false)
  const [version, setVersion] = useState<string>('unknown')

  useEffect(() => {
    checkConnections()
  }, [])

  const checkConnections = async () => {
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
        setNativeHostConnected(false)
      }
    } else {
      setExtensionConnected(false)
      setNativeHostConnected(false)
    }
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
                <span className="status-icon">{extensionConnected ? '✅' : '❌'}</span>
                <span className="status-label">Chrome Extension</span>
              </div>
              <div className="status-value">
                {extensionConnected ? 'Connected' : 'Not Connected'}
              </div>
              {!extensionConnected && (
                <p className="status-help">
                  Please install the Feitian SK Manager extension from the Chrome Web Store.
                </p>
              )}
            </div>

            <div className={`status-card ${nativeHostConnected ? 'connected' : 'disconnected'}`}>
              <div className="status-header">
                <span className="status-icon">{nativeHostConnected ? '✅' : '❌'}</span>
                <span className="status-label">Native Host</span>
              </div>
              <div className="status-value">
                {nativeHostConnected ? `Connected (v${version})` : 'Not Connected'}
              </div>
              {!nativeHostConnected && extensionConnected && (
                <p className="status-help">
                  Please install the native messaging host application.
                </p>
              )}
            </div>
          </div>
        </div>

        <div className="devices-section">
          <DeviceList onRefresh={checkConnections} />
        </div>

        <div className="quick-actions-section">
          <h2>Quick Actions</h2>
          <div className="quick-actions-grid">
            <a href="/fido2" className="quick-action-card">
              <span className="action-icon">🛡️</span>
              <h3>FIDO2</h3>
              <p>Manage authentication credentials</p>
            </a>
            <a href="/piv" className="quick-action-card">
              <span className="action-icon">🎫</span>
              <h3>PIV</h3>
              <p>Manage smart card certificates</p>
            </a>
            <a href="/otp" className="quick-action-card">
              <span className="action-icon">🔢</span>
              <h3>OTP</h3>
              <p>Configure one-time passwords</p>
            </a>
            <a href="/protocols" className="quick-action-card">
              <span className="action-icon">⚙️</span>
              <h3>Protocols</h3>
              <p>View supported protocols</p>
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}
