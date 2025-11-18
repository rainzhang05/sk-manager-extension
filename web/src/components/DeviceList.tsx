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
  const [isConnected, setIsConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)

  // Auto-connect to device when detected
  const connectDevice = async (deviceToConnect: Device) => {
    if (!window.chromeBridge || connecting || isConnected) {
      return
    }

    setConnecting(true)
    try {
      console.log('[DeviceList] Connecting to device:', deviceToConnect.id)
      const response = await window.chromeBridge.send('openDevice', {
        deviceId: deviceToConnect.id
      })

      if (response.status === 'ok') {
        console.log('[DeviceList] Device connected successfully')
        setIsConnected(true)
        
        // Dispatch custom event to notify other components
        const event = new CustomEvent('device-connected', {
          detail: { deviceId: deviceToConnect.id }
        })
        window.dispatchEvent(event)
      } else {
        console.error('[DeviceList] Failed to connect:', response.error?.message)
        setError(response.error?.message || 'Failed to connect to device')
      }
    } catch (err) {
      console.error('[DeviceList] Error connecting:', err)
      setError(err instanceof Error ? err.message : 'Failed to connect to device')
    } finally {
      setConnecting(false)
    }
  }

  // Disconnect from device
  const disconnectDevice = async (deviceId: string) => {
    if (!window.chromeBridge) {
      return
    }

    try {
      console.log('[DeviceList] Disconnecting from device:', deviceId)
      const response = await window.chromeBridge.send('closeDevice', {
        deviceId: deviceId
      })

      if (response.status === 'ok') {
        console.log('[DeviceList] Device disconnected successfully')
        setIsConnected(false)
        
        // Dispatch custom event to notify other components
        const event = new CustomEvent('device-disconnected', {
          detail: { deviceId: deviceId }
        })
        window.dispatchEvent(event)
      } else {
        console.error('[DeviceList] Failed to disconnect:', response.error?.message)
      }
    } catch (err) {
      console.error('[DeviceList] Error disconnecting:', err)
    }
  }

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
        const currentDevice = devices.length > 0 ? devices[0] : null
        
        // Check if device changed
        const deviceChanged = !device || !currentDevice || device.id !== currentDevice.id
        
        setDevice(currentDevice)
        
        // Auto-connect if new device detected and not already connected
        if (currentDevice && deviceChanged && !isConnected) {
          // Small delay to ensure UI updates first
          setTimeout(() => connectDevice(currentDevice), 100)
        } else if (!currentDevice && isConnected) {
          // Device was removed, disconnect
          if (device) {
            disconnectDevice(device.id)
          }
        }
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
      </div>
      <div className="device-grid">
        <div className={`device-card ${isConnected ? 'connected' : 'detected'}`}>
          <div className="device-card-header">
            <span className={`device-type-badge ${device.device_type.toLowerCase()}`}>
              {device.device_type.toUpperCase()}
            </span>
            <span className={isConnected ? 'connected-badge' : 'connecting-badge'}>
              {connecting ? 'Connecting...' : isConnected ? 'Connected' : 'Detected'}
            </span>
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
