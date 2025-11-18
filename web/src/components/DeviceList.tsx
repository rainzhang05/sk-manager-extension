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
  const [device, setDevice] = useState<Device | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadDevices = async () => {
    setLoading(true)
    setError(null)

    try {
      console.log('[DeviceList] Starting loadDevices, checking for chromeBridge...')
      console.log('[DeviceList] window.chromeBridge exists?', !!window.chromeBridge)
      
      // Wait for chromeBridge to be available (with timeout)
      let retries = 0
      const maxRetries = 50 // 5 seconds max wait
      while (!window.chromeBridge && retries < maxRetries) {
        await new Promise(resolve => setTimeout(resolve, 100))
        retries++
      }

      console.log('[DeviceList] After waiting, chromeBridge exists?', !!window.chromeBridge)

      // Check if chromeBridge exists
      if (!window.chromeBridge) {
        console.log('[DeviceList] chromeBridge still not available after waiting')
        throw new Error('Chrome extension not connected. Please install the extension.')
      }

      console.log('[DeviceList] Calling listDevices...')
      const response = await window.chromeBridge!.send('listDevices')
      console.log('[DeviceList] Response:', response)
      
      if (response.status === 'ok' && response.result) {
        const result = response.result as { devices?: Device[] }
        const devices = result.devices || []
        // Only show the first device
        setDevice(devices.length > 0 ? devices[0] : null)
      } else {
        throw new Error(response.error?.message || 'Failed to list devices')
      }
    } catch (err) {
      console.error('Error loading devices:', err)
      const message = err instanceof Error ? err.message : 'Failed to load devices'
      setError(message)
      setDevice(null)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadDevices()
    
    // Poll for devices every 2 seconds
    const interval = setInterval(loadDevices, 2000)
    
    return () => clearInterval(interval)
  }, [])

  const handleRefresh = () => {
    loadDevices()
    if (onRefresh) {
      onRefresh()
    }
  }

  const formatVendorId = (vid: number) => `0x${vid.toString(16).padStart(4, '0').toUpperCase()}`
  const formatProductId = (pid: number) => `0x${pid.toString(16).padStart(4, '0').toUpperCase()}`

  if (loading && !device) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Device</h2>
          <button onClick={handleRefresh} className="btn-secondary" disabled>
            Refresh
          </button>
        </div>
        <div className="loading-state">
          <div className="spinner"></div>
          <p>Scanning for Feitian device...</p>
        </div>
      </div>
    )
  }

  if (error && !device) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Device</h2>
          <button onClick={handleRefresh} className="btn-secondary">
            Refresh
          </button>
        </div>
        <div className="error-state">
          <h3>Error</h3>
          <p>{error}</p>
          <button onClick={handleRefresh} className="btn-primary">
            Try Again
          </button>
        </div>
      </div>
    )
  }

  if (!device) {
    return (
      <div className="device-list">
        <div className="device-list-header">
          <h2>Connected Device</h2>
          <button onClick={handleRefresh} className="btn-secondary">
            Refresh
          </button>
        </div>
        <div className="empty-state">
          <h3>No Device Detected</h3>
          <p>Please insert your Feitian security key.</p>
          <p className="status-help">The device will appear automatically when connected.</p>
        </div>
      </div>
    )
  }

  return (
    <div className="device-list">
      <div className="device-list-header">
        <h2>Connected Device</h2>
        <button onClick={handleRefresh} className="btn-secondary">
          Refresh
        </button>
      </div>
      <div className="device-grid">
        <div className="device-card connected">
          <div className="device-card-header">
            <span className={`device-type-badge ${device.device_type.toLowerCase()}`}>
              {device.device_type.toUpperCase()}
            </span>
            <span className="connected-badge">Connected</span>
          </div>
          <div className="device-card-body">
            <h3 className="device-name">
              {device.product_name || `Feitian ${device.device_type} Device`}
            </h3>
            <div className="device-details">
              <div className="device-detail-row">
                <span className="detail-label">VID:</span>
                <span className="detail-value">{formatVendorId(device.vendor_id)}</span>
              </div>
              <div className="device-detail-row">
                <span className="detail-label">PID:</span>
                <span className="detail-value">{formatProductId(device.product_id)}</span>
              </div>
              {device.manufacturer && (
                <div className="device-detail-row">
                  <span className="detail-label">Manufacturer:</span>
                  <span className="detail-value">{device.manufacturer}</span>
                </div>
              )}
              {device.serial_number && (
                <div className="device-detail-row">
                  <span className="detail-label">Serial:</span>
                  <span className="detail-value">{device.serial_number}</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
