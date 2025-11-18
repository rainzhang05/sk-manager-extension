import { useState, useEffect } from 'react'
import '../styles/Protocols.css'

interface ProtocolSupport {
  fido2: boolean
  u2f: boolean
  piv: boolean
  openpgp: boolean
  otp: boolean
  ndef: boolean
}

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

export default function Protocols() {
  const [device, setDevice] = useState<Device | null>(null)
  const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null)
  const [protocols, setProtocols] = useState<ProtocolSupport | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load devices and check for connected device
  const loadDevices = async () => {
    try {
      if (!window.chromeBridge) {
        return
      }

      const response = await window.chromeBridge.send('listDevices')
      if (response.status === 'ok' && response.result) {
        const result = response.result as { devices: Device[] }
        if (result.devices.length > 0) {
          setDevice(result.devices[0])
        }
      } else {
        setDevice(null)
      }
    } catch (err) {
      console.error('Failed to load devices:', err)
    }
  }

  // Detect protocols for connected device
  const detectProtocols = async (deviceId: string) => {
    if (!window.chromeBridge) {
      setError('Chrome extension not connected')
      return
    }

    setLoading(true)
    setError(null)

    try {
      console.log('[Protocols] Detecting protocols for device:', deviceId)
      const response = await window.chromeBridge.send('detectProtocols', {
        deviceId: deviceId
      })

      console.log('[Protocols] Detection response:', response)

      if (response.status === 'ok' && response.result) {
        const result = response.result as { protocols: ProtocolSupport }
        setProtocols(result.protocols)
        console.log('[Protocols] Protocols detected:', result.protocols)
      } else {
        setError(response.error?.message || 'Failed to detect protocols')
      }
    } catch (err) {
      console.error('[Protocols] Detection error:', err)
      setError('Failed to detect protocols: ' + String(err))
    } finally {
      setLoading(false)
    }
  }

  // Listen for device connection changes
  useEffect(() => {
    let detectionInProgress = false

    const handleDeviceConnected = async (event: Event) => {
      const customEvent = event as CustomEvent
      const deviceId = customEvent.detail.deviceId
      console.log('[Protocols] Device connected event:', deviceId)
      
      // Prevent duplicate detections
      if (detectionInProgress || (connectedDeviceId === deviceId && protocols !== null)) {
        console.log('[Protocols] Skipping detection - already detected for this device')
        return
      }
      
      setConnectedDeviceId(deviceId)
      
      // Auto-detect protocols when device connects (only once)
      detectionInProgress = true
      await detectProtocols(deviceId)
      detectionInProgress = false
    }

    const handleDeviceDisconnected = () => {
      console.log('[Protocols] Device disconnected')
      setConnectedDeviceId(null)
      setProtocols(null)
      setError(null)
      detectionInProgress = false
    }

    window.addEventListener('device-connected', handleDeviceConnected)
    window.addEventListener('device-disconnected', handleDeviceDisconnected)

    // Load devices on mount
    loadDevices()

    return () => {
      window.removeEventListener('device-connected', handleDeviceConnected)
      window.removeEventListener('device-disconnected', handleDeviceDisconnected)
    }
  }, [connectedDeviceId, protocols])

  const protocolList = [
    {
      id: 'fido2',
      name: 'FIDO2',
      subtitle: 'CTAP2',
      description: 'Modern authentication protocol with biometric support',
      supported: protocols?.fido2 || false,
    },
    {
      id: 'u2f',
      name: 'U2F',
      subtitle: 'CTAP1',
      description: 'Legacy universal second factor authentication',
      supported: protocols?.u2f || false,
    },
    {
      id: 'piv',
      name: 'PIV',
      subtitle: 'Smart Card',
      description: 'Personal identity verification for secure access',
      supported: protocols?.piv || false,
    },
    {
      id: 'openpgp',
      name: 'OpenPGP',
      subtitle: 'Email Security',
      description: 'Email encryption and digital signatures',
      supported: protocols?.openpgp || false,
    },
    {
      id: 'otp',
      name: 'OTP',
      subtitle: 'HOTP',
      description: 'One-time password generation',
      supported: protocols?.otp || false,
    },
    {
      id: 'ndef',
      name: 'NDEF',
      subtitle: 'NFC',
      description: 'NFC data exchange format',
      supported: protocols?.ndef || false,
    },
  ]

  return (
    <div className="page">
      <div className="page-header">
        <h1>Protocols</h1>
        <p className="page-description">
          Detect and view supported protocols on your Feitian security key
        </p>
      </div>

      {!connectedDeviceId && !loading && (
        <div className="protocols-notice">
          <div>
            <strong>Connect a device to detect protocols</strong>
            <p>
              Go to the Dashboard page and connect a Feitian security key.
              Protocol detection will run automatically.
            </p>
          </div>
        </div>
      )}

      {loading && (
        <div className="protocols-notice">
          <div>
            <strong>Detecting protocols...</strong>
            <p>Please wait while we probe the device for supported protocols.</p>
          </div>
        </div>
      )}

      {error && (
        <div className="protocols-notice" style={{ borderColor: '#EF4444' }}>
          <div>
            <strong>Detection Error</strong>
            <p>{error}</p>
          </div>
        </div>
      )}

      {connectedDeviceId && device && (
        <div className="protocols-device-info">
          <h3>Connected Device</h3>
          <p>
            <strong>{device.product_name || 'Unknown Device'}</strong>
            {' • '}
            Type: {device.device_type}
            {' • '}
            ID: {device.id}
          </p>
          {protocols && (
            <p style={{ marginTop: '8px', fontSize: '14px', color: '#666' }}>
              Protocol detection completed automatically
            </p>
          )}
        </div>
      )}

      <div className="protocols-grid">
        {protocolList.map((protocol) => (
          <div key={protocol.id} className={`protocol-card ${protocol.supported ? 'supported' : 'unsupported'}`}>
            <div className="protocol-header">
              <div className="protocol-badge">
                {connectedDeviceId ? (protocol.supported ? 'Supported' : 'Not Supported') : 'Unknown'}
              </div>
            </div>
            <div className="protocol-body">
              <h3 className="protocol-name">{protocol.name}</h3>
              <p className="protocol-subtitle">{protocol.subtitle}</p>
              <p className="protocol-description">{protocol.description}</p>
            </div>
          </div>
        ))}
      </div>

      {connectedDeviceId && protocols && (
        <div className="protocols-help">
          <h3>Protocol Detection Details</h3>
          <p>
            Protocol detection is performed using the following methods:
          </p>
          <ul>
            <li><strong>FIDO2:</strong> CTAP2 getInfo command via HID</li>
            <li><strong>U2F:</strong> CTAP1 version command via HID</li>
            <li><strong>PIV:</strong> SELECT APDU (A0 00 00 03 08) via CCID</li>
            <li><strong>OpenPGP:</strong> SELECT APDU (D2 76 00 01 24 01) via CCID</li>
            <li><strong>OTP:</strong> Vendor-specific command via HID</li>
            <li><strong>NDEF:</strong> SELECT APDU (D2 76 00 00 85 01 01) via CCID</li>
          </ul>
          <p style={{ marginTop: '16px', fontSize: '14px', color: '#666' }}>
            Note: Some protocols may not be detected if the device doesn't support
            the detection method (e.g., CCID-only protocols on HID-only devices).
          </p>
        </div>
      )}
    </div>
  )
}
