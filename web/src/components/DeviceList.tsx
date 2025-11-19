import { useState, useEffect, useCallback, useRef } from 'react'
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

  // Use refs to track state without causing dependency changes
  const openDeviceIdRef = useRef<string | null>(null)
  const connectingRef = useRef(false)
  const deviceRef = useRef<Device | null>(null)

  // Update refs when state changes
  useEffect(() => {
    deviceRef.current = device
  }, [device])

  useEffect(() => {
    connectingRef.current = connecting
  }, [connecting])

  // Auto-connect to device when detected
  const connectDevice = useCallback(async (deviceToConnect: Device) => {
    // Check if device is already open or we're already trying to connect
    if (!window.chromeBridge || connectingRef.current) {
      return
    }

    // If this is the device we're already connected to, don't try again
    if (openDeviceIdRef.current === deviceToConnect.id) {
      console.log('[DeviceList] Device already tracked as open:', deviceToConnect.id)
      return
    }

    setConnecting(true)
    connectingRef.current = true
    try {
      console.log('[DeviceList] Connecting to device:', deviceToConnect.id)
      const response = await window.chromeBridge.send('openDevice', {
        deviceId: deviceToConnect.id
      })

      if (response.status === 'ok') {
        console.log('[DeviceList] Device connected successfully')
        setIsConnected(true)
        openDeviceIdRef.current = deviceToConnect.id

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
          openDeviceIdRef.current = deviceToConnect.id

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
      connectingRef.current = false
    }
  }, []) // No dependencies - stable callback

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
        openDeviceIdRef.current = null

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
          openDeviceIdRef.current = null
        }
      }
    } catch (err) {
      console.error('[DeviceList] Error disconnecting:', err)
    }
  }, []) // No dependencies - stable callback

  const loadDevices = useCallback(async () => {
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

        // Check if device changed (comparing IDs only)
        const previousDevice = deviceRef.current
        const deviceChanged = !previousDevice || !currentDevice || previousDevice.id !== currentDevice.id

        // Update device state
        setDevice(currentDevice)

        // Clear error if we successfully got device list
        setError(null)

        // Check if device is already connected via sessionStorage
        const storedDeviceId = sessionStorage.getItem('connectedDeviceId')

        // Handle device connection/disconnection
        if (currentDevice && deviceChanged) {
          // New device detected or device changed
          if (openDeviceIdRef.current && openDeviceIdRef.current !== currentDevice.id) {
            // Different device, disconnect old one first
            await disconnectDevice(openDeviceIdRef.current)
          }

          // Check if already connected
          if (storedDeviceId === currentDevice.id) {
            // Device is already connected from previous session, just update local state
            console.log('[DeviceList] Device already connected from previous session:', currentDevice.id)
            setIsConnected(true)
            openDeviceIdRef.current = currentDevice.id
          } else if (!openDeviceIdRef.current) {
            // Not connected yet, try to connect
            // Small delay to ensure UI updates first
            setTimeout(() => connectDevice(currentDevice), 100)
          }
        } else if (!currentDevice && openDeviceIdRef.current) {
          // Device was removed, disconnect and clear state
          await disconnectDevice(openDeviceIdRef.current)
          setDevice(null)
          setIsConnected(false)
          openDeviceIdRef.current = null
        } else if (!currentDevice) {
          // No device and no open device, just clear state
          setIsConnected(false)
          openDeviceIdRef.current = null
        }
      } else {
        throw new Error(response.error?.message || 'Failed to list devices')
      }
    } catch (err) {
      console.error('Error loading devices:', err)
      const message = err instanceof Error ? err.message : 'Failed to load devices'

      // Only set error if we don't have a device already
      if (!deviceRef.current) {
        setError(message)
        setDevice(null)
      }
    } finally {
      // Turn off loading after first attempt
      setLoading(false)
    }
  }, [connectDevice, disconnectDevice]) // Minimal stable dependencies

  useEffect(() => {
    // Initial load
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
