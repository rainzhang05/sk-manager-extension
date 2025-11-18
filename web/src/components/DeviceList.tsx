import { useState, useEffect, useCallback } from 'react'
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
  const [openDeviceId, setOpenDeviceId] = useState<string | null>(null)

  // Auto-connect to device when detected
  const connectDevice = useCallback(async (deviceToConnect: Device) => {
    // Check if device is already open or we're already trying to connect to it
    if (!window.chromeBridge || connecting) {
      return
    }

    // If this is the device we're already connected to, don't try again
    if (openDeviceId === deviceToConnect.id) {
      console.log('[DeviceList] Device already tracked as open:', deviceToConnect.id)
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
        setOpenDeviceId(deviceToConnect.id)
        
        // Store connected device ID in sessionStorage for other components
        sessionStorage.setItem('connectedDeviceId', deviceToConnect.id)
        
        // Dispatch custom event to notify other components
        const event = new CustomEvent('device-connected', {
          detail: { deviceId: deviceToConnect.id }
        })
        window.dispatchEvent(event)
      } else {
        console.error('[DeviceList] Failed to connect:', response.error?.message)
        
        // Check if device is already open
        if (response.error?.message?.includes('already open')) {
          console.log('[DeviceList] Device is already open, treating as connected')
          // Device is already open, just update state
          setIsConnected(true)
          setOpenDeviceId(deviceToConnect.id)
          
          // Store connected device ID in sessionStorage for other components
          sessionStorage.setItem('connectedDeviceId', deviceToConnect.id)
          
          // Dispatch event anyway since device is usable
          const event = new CustomEvent('device-connected', {
            detail: { deviceId: deviceToConnect.id }
          })
          window.dispatchEvent(event)
        } else {
          // Real error, show it
          setError(response.error?.message || 'Failed to connect to device')
        }
      }
    } catch (err) {
      console.error('[DeviceList] Error connecting:', err)
      setError(err instanceof Error ? err.message : 'Failed to connect to device')
    } finally {
      setConnecting(false)
    }
  }, [connecting, openDeviceId])

  // Disconnect from device
  const disconnectDevice = useCallback(async (deviceId: string) => {
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
        setOpenDeviceId(null)
        
        // Clear connected device ID from sessionStorage
        sessionStorage.removeItem('connectedDeviceId')
        
        // Dispatch custom event to notify other components
        const event = new CustomEvent('device-disconnected', {
          detail: { deviceId: deviceId }
        })
        window.dispatchEvent(event)
      } else {
        console.error('[DeviceList] Failed to disconnect:', response.error?.message)
        // If device is not open, that's fine - just clear state
        if (response.error?.message?.includes('not open')) {
          setIsConnected(false)
          setOpenDeviceId(null)
        }
      }
    } catch (err) {
      console.error('[DeviceList] Error disconnecting:', err)
    }
  }, [])

  const loadDevices = useCallback(async () => {
    // Only show loading on very first load
    const isFirstLoad = device === null && !error && loading
    
    try {
      // Wait for chromeBridge to be available (with timeout)
      let retries = 0
      const maxRetries = 50 // 5 seconds max wait
      while (!window.chromeBridge && retries < maxRetries) {
        await new Promise(resolve => setTimeout(resolve, 100))
        retries++
      }

      // Check if chromeBridge exists
      if (!window.chromeBridge) {
        throw new Error('Chrome extension not connected. Please install the extension.')
      }

      const response = await window.chromeBridge!.send('listDevices')
      
      if (response.status === 'ok' && response.result) {
        const result = response.result as { devices?: Device[] }
        const devices = result.devices || []
        const currentDevice = devices.length > 0 ? devices[0] : null
        
        // Check if device changed
        const deviceChanged = !device || !currentDevice || device.id !== currentDevice.id
        
        // Update device state
        setDevice(currentDevice)
        
        // Clear error if we successfully got device list
        if (error) {
          setError(null)
        }
        
        // Handle device connection/disconnection
        if (currentDevice && deviceChanged) {
          // New device detected or device changed
          if (openDeviceId && openDeviceId !== currentDevice.id) {
            // Different device, disconnect old one first
            await disconnectDevice(openDeviceId)
          }
          
          // Try to connect (will handle "already open" internally)
          // Small delay to ensure UI updates first
          setTimeout(() => connectDevice(currentDevice), 100)
        } else if (!currentDevice && openDeviceId) {
          // Device was removed, disconnect and clear state
          await disconnectDevice(openDeviceId)
          setDevice(null)
          setIsConnected(false)
          setOpenDeviceId(null)
        } else if (!currentDevice) {
          // No device and no open device, just clear state
          setIsConnected(false)
          setOpenDeviceId(null)
        }
        // REMOVED: The problematic "else if" that restored state on every poll
        // This was causing repeated connection attempts and event dispatches
      } else {
        throw new Error(response.error?.message || 'Failed to list devices')
      }
    } catch (err) {
      console.error('Error loading devices:', err)
      const message = err instanceof Error ? err.message : 'Failed to load devices'
      // Only set error and clear device on first load
      if (isFirstLoad || device !== null) {
        setError(message)
        setDevice(null)
      }
    } finally {
      // Turn off loading after first attempt
      if (loading) {
        setLoading(false)
      }
    }
  }, [device, error, loading, openDeviceId, connectDevice, disconnectDevice])

  useEffect(() => {
    loadDevices()
    
    // Poll for devices every 2 seconds
    const interval = setInterval(loadDevices, 2000)
    
    return () => clearInterval(interval)
  }, [loadDevices])

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
