import { useState, useEffect } from 'react'
import '../styles/DeviceList.css'

interface Device {
  id: string
  vendor_id: number
  product_id: number
  device_type: 'Hid' | 'Ccid'
  manufacturer?: string
  product_name?: string
  serial_number?: string
  path: string
}

interface DeviceListProps {
  onRefresh?: () => void
}

export default function DeviceList({ onRefresh }: DeviceListProps) {
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null)
  const [connecting, setConnecting] = useState<string | null>(null)

  const loadDevices = async () => {
    setLoading(true)
    setError(null)

    try {
      // Check if chromeBridge exists
      if (!window.chromeBridge) {
        throw new Error('Chrome extension not connected. Please install the extension.')
      }

      const response = await window.chromeBridge.send('listDevices')
      
      if (response.status === 'ok' && response.result) {
        setDevices(response.result.devices || [])
      } else {
        throw new Error(response.error?.message || 'Failed to list devices')
      }
    } catch (err) {
      console.error('Error loading devices:', err)
      setError(err instanceof Error ? err.message : 'Failed to load devices')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadDevices()
  }, [])

  const handleRefresh = () => {
    loadDevices()
    onRefresh?.()
  }

  const handleConnect = async (deviceId: string) => {
    try {
      setConnecting(deviceId)
      const response = await window.chromeBridge.send('openDevice', { deviceId })
      
      if (response.status === 'ok') {
        setConnectedDeviceId(deviceId)
        console.log('Device connected successfully:', deviceId)
      } else {
        throw new Error(response.error?.message || 'Failed to connect')
      }
    } catch (err) {
      console.error('Error connecting to device:', err)
      const message = err instanceof Error ? err.message : 'Connection failed'
      alert(`Failed to connect: ${message}`)
    } finally {
      setConnecting(null)
    }
  }

  const handleDisconnect = async (deviceId: string) => {
    try {
      const response = await window.chromeBridge.send('closeDevice', { deviceId })
      
      if (response.status === 'ok') {
        setConnectedDeviceId(null)
        console.log('Device disconnected successfully:', deviceId)
      } else {
        throw new Error(response.error?.message || 'Failed to disconnect')
      }
    } catch (err) {
      console.error('Error disconnecting device:', err)
      const message = err instanceof Error ? err.message : 'Disconnect failed'
      alert(`Failed to disconnect: ${message}`)
    }
  }

  const formatVendorId = (vid: number) => `0x${vid.toString(16).padStart(4, '0').toUpperCase()}`
  const formatProductId = (pid: number) => `0x${pid.toString(16).padStart(4, '0').toUpperCase()}`

  if (loading) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Devices</h2>
          <button onClick={handleRefresh} className="btn-secondary" disabled>
            <span className="icon">🔄</span>
            Refresh
          </button>
        </div>
        <div className="loading-state">
          <div className="spinner"></div>
          <p>Scanning for Feitian devices...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Devices</h2>
          <button onClick={handleRefresh} className="btn-secondary">
            <span className="icon">🔄</span>
            Refresh
          </button>
        </div>
        <div className="error-state">
          <span className="error-icon">⚠️</span>
          <h3>Error</h3>
          <p>{error}</p>
          <button onClick={handleRefresh} className="btn-primary">
            Try Again
          </button>
        </div>
      </div>
    )
  }

  if (devices.length === 0) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Devices</h2>
          <button onClick={handleRefresh} className="btn-secondary">
            <span className="icon">🔄</span>
            Refresh
          </button>
        </div>
        <div className="empty-state">
          <span className="empty-icon">🔌</span>
          <h3>No Feitian devices detected</h3>
          <p>Please plug in your Feitian security key and click Refresh.</p>
          <button onClick={handleRefresh} className="btn-primary">
            Refresh
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="device-list">
      <div className="device-list-header">
        <h2>Connected Devices</h2>
        <button onClick={handleRefresh} className="btn-secondary">
          <span className="icon">🔄</span>
          Refresh
        </button>
      </div>
      <div className="device-grid">
        {devices.map((device) => {
          const isConnected = connectedDeviceId === device.id
          const isConnecting = connecting === device.id
          const canConnect = !connectedDeviceId || isConnected
          
          return (
            <div key={device.id} className={`device-card ${isConnected ? 'connected' : ''}`}>
              <div className="device-card-header">
                <span className="device-icon">🔑</span>
                <span className={`device-type-badge ${device.device_type.toLowerCase()}`}>
                  {device.device_type.toUpperCase()}
                </span>
                {isConnected && (
                  <span className="connected-badge">Connected</span>
                )}
              </div>
              <div className="device-card-body">
                <h3 className="device-name">
                  {device.manufacturer && device.product_name
                    ? `${device.manufacturer} ${device.product_name}`
                    : device.product_name || device.manufacturer || 'Unknown Device'}
                </h3>
                <div className="device-details">
                  <div className="device-detail-row">
                    <span className="detail-label">Type:</span>
                    <span className="detail-value">{device.device_type}</span>
                  </div>
                  <div className="device-detail-row">
                    <span className="detail-label">VID:</span>
                    <span className="detail-value">{formatVendorId(device.vendor_id)}</span>
                    <span className="detail-label">PID:</span>
                    <span className="detail-value">{formatProductId(device.product_id)}</span>
                  </div>
                  {device.serial_number && (
                    <div className="device-detail-row">
                      <span className="detail-label">Serial:</span>
                      <span className="detail-value">{device.serial_number}</span>
                    </div>
                  )}
                </div>
              </div>
              <div className="device-card-footer">
                {isConnected ? (
                  <button 
                    className="btn-secondary btn-disconnect" 
                    onClick={() => handleDisconnect(device.id)}
                  >
                    Disconnect
                  </button>
                ) : (
                  <button 
                    className="btn-primary btn-connect" 
                    onClick={() => handleConnect(device.id)}
                    disabled={!canConnect || isConnecting}
                  >
                    {isConnecting ? 'Connecting...' : 'Connect'}
                  </button>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

// Extend Window interface for TypeScript
declare global {
  interface Window {
    chromeBridge: {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      send: (command: string, params?: Record<string, unknown>) => Promise<any>
      isConnected: () => Promise<boolean>
      getVersion: () => Promise<string>
    }
  }
}
