import { useState, useEffect } from 'react'
import '../styles/FIDO2.css'

interface Fido2Info {
  versions: string[]
  extensions: string[]
  aaguid: string
  options: {
    plat: boolean
    rk: boolean
    client_pin: boolean | null
    up: boolean
    uv: boolean | null
  }
  max_msg_size: number | null
  pin_protocols: number[]
  max_credential_count_in_list: number | null
  max_credential_id_length: number | null
  transports: string[]
  algorithms: string[]
  max_authenticator_config_length: number | null
  default_cred_protect: number | null
}

interface PinRetries {
  retries: number
  power_cycle_required: boolean
}

interface Credential {
  rp_id: string
  rp_name: string
  user_id: string
  user_name: string
  user_display_name: string
  credential_id: string
  public_key: string | null
  cred_protect: number | null
}

export default function FIDO2() {
  const [connectedDevice, setConnectedDevice] = useState<string | null>(null)
  const [deviceInfo, setDeviceInfo] = useState<Fido2Info | null>(null)
  const [pinRetries, setPinRetries] = useState<PinRetries | null>(null)
  const [credentials, setCredentials] = useState<Credential[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  
  // PIN management state
  const [showSetPin, setShowSetPin] = useState(false)
  const [showChangePin, setShowChangePin] = useState(false)
  const [newPin, setNewPin] = useState('')
  const [currentPin, setCurrentPin] = useState('')
  const [confirmPin, setConfirmPin] = useState('')
  
  // Reset confirmation
  const [showResetConfirm, setShowResetConfirm] = useState(false)
  const [resetConfirmText, setResetConfirmText] = useState('')
  
  // Device re-insertion prompt
  const [showReinsertPrompt, setShowReinsertPrompt] = useState(false)
  const [reinsertAction, setReinsertAction] = useState<'setPin' | 'changePin' | 'reset' | null>(null)

  useEffect(() => {
    // Check if there's already a connected device on mount
    const checkExistingDevice = () => {
      const storedDeviceId = sessionStorage.getItem('connectedDeviceId')
      if (storedDeviceId && !connectedDevice) {
        console.log('[FIDO2] Found existing connected device:', storedDeviceId)
        setConnectedDevice(storedDeviceId)
        loadDeviceInfo(storedDeviceId)
        loadPinRetries(storedDeviceId)
        loadCredentials(storedDeviceId)
      }
    }
    
    // Check for existing device immediately
    checkExistingDevice()
    
    // Listen for device connection events
    const handleDeviceConnected = (event: Event) => {
      const customEvent = event as CustomEvent
      const deviceId = customEvent.detail.deviceId
      console.log('[FIDO2] Device connected event:', deviceId)
      setConnectedDevice(deviceId)
      loadDeviceInfo(deviceId)
      loadPinRetries(deviceId)
      loadCredentials(deviceId)
    }

    const handleDeviceDisconnected = () => {
      console.log('[FIDO2] Device disconnected event')
      setConnectedDevice(null)
      setDeviceInfo(null)
      setPinRetries(null)
      setCredentials([])
    }

    window.addEventListener('device-connected', handleDeviceConnected)
    window.addEventListener('device-disconnected', handleDeviceDisconnected)

    return () => {
      window.removeEventListener('device-connected', handleDeviceConnected)
      window.removeEventListener('device-disconnected', handleDeviceDisconnected)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const loadDeviceInfo = async (deviceId: string) => {
    setLoading(true)
    setError(null)

    try {
      // First check device type from sessionStorage or device list
      const deviceListResponse = await window.chromeBridge!.send('listDevices', {})
      let deviceType = 'Hid' // default

      if (deviceListResponse.status === 'ok' && deviceListResponse.result) {
        const result = deviceListResponse.result as { devices?: Array<{id: string, device_type: string}> }
        const devices = result.devices || []
        const currentDevice = devices.find(d => d.id === deviceId)
        if (currentDevice) {
          deviceType = currentDevice.device_type
          console.log('[FIDO2] Device type:', deviceType)

          // If device is CCID, show helpful message
          if (deviceType === 'Ccid') {
            setError('This device is a smart card reader (CCID). FIDO2 management for smart card readers is not yet supported. Please use a FIDO2 HID security key.')
            setLoading(false)
            return
          }
        }
      }

      const response = await window.chromeBridge!.send('fido2GetInfo', { deviceId })

      if (response.status === 'ok' && response.result) {
        const result = response.result as { info: Fido2Info }
        setDeviceInfo(result.info)
      } else {
        const errorMsg = response.error?.message || 'Failed to get device info'

        // Provide helpful error messages
        if (errorMsg.includes('timeout')) {
          setError('Device did not respond to FIDO2 commands. This device may not support FIDO2/CTAP2, or it may be locked. Try:\n\n1. Removing and re-inserting the device\n2. Verifying this is a FIDO2-capable security key\n3. Checking if the device requires a button press')
        } else if (errorMsg.includes('CTAP2 error')) {
          setError('The device returned a CTAP2 error. This may indicate the device is in an error state or does not support the requested operation.')
        } else {
          setError(errorMsg)
        }
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error'
      if (errorMsg.includes('timeout')) {
        setError('Communication timeout. Please ensure the device is properly connected and supports FIDO2.')
      } else {
        setError(errorMsg)
      }
    } finally {
      setLoading(false)
    }
  }

  const loadPinRetries = async (deviceId: string) => {
    try {
      const response = await window.chromeBridge!.send('fido2GetPinRetries', { deviceId })
      
      if (response.status === 'ok' && response.result) {
        const result = response.result as { retries: PinRetries }
        setPinRetries(result.retries)
      }
    } catch (err) {
      console.error('Failed to get PIN retries:', err)
    }
  }

  const loadCredentials = async (deviceId: string) => {
    try {
      const response = await window.chromeBridge!.send('fido2ListCredentials', { deviceId })
      
      if (response.status === 'ok' && response.result) {
        const result = response.result as { credentials: Credential[] }
        setCredentials(result.credentials)
      }
    } catch (err) {
      console.error('Failed to list credentials:', err)
    }
  }

  const handleSetPin = async () => {
    if (!connectedDevice) return
    
    if (newPin !== confirmPin) {
      setError('PINs do not match')
      return
    }
    
    if (newPin.length < 4) {
      setError('PIN must be at least 4 characters')
      return
    }
    
    // Show re-insert prompt
    setReinsertAction('setPin')
    setShowReinsertPrompt(true)
  }
  
  const executeSetPin = async () => {
    if (!connectedDevice) return
    
    setLoading(true)
    setError(null)
    setSuccessMessage(null)
    setShowReinsertPrompt(false)
    
    try {
      const response = await window.chromeBridge!.send('fido2SetPin', {
        deviceId: connectedDevice,
        newPin
      })
      
      if (response.status === 'ok') {
        setSuccessMessage('PIN set successfully. You can now use your security key with PIN protection.')
        setShowSetPin(false)
        setNewPin('')
        setConfirmPin('')
        // Reload device info to update PIN status
        loadDeviceInfo(connectedDevice)
        loadPinRetries(connectedDevice)
      } else {
        const errorMsg = response.error?.message || 'Failed to set PIN'
        if (errorMsg.includes('not yet implemented') || errorMsg.includes('timeout')) {
          setError('PIN management requires full CTAP2 PIN protocol implementation. This feature is currently in development. The authenticator supports PIN operations, but this manager needs additional implementation to handle encrypted PIN operations.')
        } else {
          setError(errorMsg)
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  const handleChangePin = async () => {
    if (!connectedDevice) return
    
    if (newPin !== confirmPin) {
      setError('New PINs do not match')
      return
    }
    
    if (newPin.length < 4) {
      setError('PIN must be at least 4 characters')
      return
    }
    
    if (!currentPin) {
      setError('Current PIN is required')
      return
    }
    
    // Show re-insert prompt
    setReinsertAction('changePin')
    setShowReinsertPrompt(true)
  }
  
  const executeChangePin = async () => {
    if (!connectedDevice) return
    
    setLoading(true)
    setError(null)
    setSuccessMessage(null)
    setShowReinsertPrompt(false)
    
    try {
      const response = await window.chromeBridge!.send('fido2ChangePin', {
        deviceId: connectedDevice,
        currentPin,
        newPin
      })
      
      if (response.status === 'ok') {
        setSuccessMessage('PIN changed successfully')
        setShowChangePin(false)
        setCurrentPin('')
        setNewPin('')
        setConfirmPin('')
        loadPinRetries(connectedDevice)
      } else {
        const errorMsg = response.error?.message || 'Failed to change PIN'
        if (errorMsg.includes('not yet implemented') || errorMsg.includes('timeout')) {
          setError('PIN management requires full CTAP2 PIN protocol implementation. This feature is currently in development. The authenticator supports PIN operations, but this manager needs additional implementation to handle encrypted PIN operations.')
        } else {
          setError(errorMsg)
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  const handleDeleteCredential = async (credentialId: string) => {
    if (!connectedDevice) return
    
    if (!confirm('Are you sure you want to delete this credential?')) {
      return
    }
    
    setLoading(true)
    setError(null)
    setSuccessMessage(null)
    
    try {
      const response = await window.chromeBridge!.send('fido2DeleteCredential', {
        deviceId: connectedDevice,
        credentialId
      })
      
      if (response.status === 'ok') {
        setSuccessMessage('Credential deleted successfully')
        loadCredentials(connectedDevice)
      } else {
        setError(response.error?.message || 'Failed to delete credential')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  const handleResetDevice = async () => {
    if (!connectedDevice) return
    
    if (resetConfirmText !== 'RESET') {
      setError('Please type RESET to confirm')
      return
    }
    
    setLoading(true)
    setError(null)
    setSuccessMessage(null)
    
    try {
      const response = await window.chromeBridge!.send('fido2ResetDevice', {
        deviceId: connectedDevice
      })
      
      if (response.status === 'ok') {
        setSuccessMessage('Device reset successfully. All credentials and PIN have been cleared.')
        setShowResetConfirm(false)
        setResetConfirmText('')
        loadDeviceInfo(connectedDevice)
        loadPinRetries(connectedDevice)
        loadCredentials(connectedDevice)
      } else {
        setError(response.error?.message || 'Failed to reset device')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  if (!connectedDevice) {
    return (
      <div className="page">
        <h1>FIDO2 Manager</h1>
        <div className="empty-state">
          <div className="empty-icon">🔐</div>
          <p>No device connected</p>
          <p className="empty-hint">Connect a Feitian security key from the Dashboard to manage FIDO2 settings</p>
        </div>
      </div>
    )
  }

  return (
    <div className="page">
      <h1>FIDO2 Manager</h1>
      
      {error && (
        <div className="message error">
          <span>❌</span>
          <span>{error}</span>
          <button onClick={() => setError(null)}>✕</button>
        </div>
      )}
      
      {successMessage && (
        <div className="message success">
          <span>✓</span>
          <span>{successMessage}</span>
          <button onClick={() => setSuccessMessage(null)}>✕</button>
        </div>
      )}

      {/* Device Info */}
      {deviceInfo && (
        <div className="card">
          <h2>Device Information</h2>
          <div className="info-grid">
            <div className="info-item">
              <span className="info-label">Versions:</span>
              <span className="info-value">{deviceInfo.versions.join(', ')}</span>
            </div>
            <div className="info-item">
              <span className="info-label">AAGUID:</span>
              <span className="info-value">{deviceInfo.aaguid}</span>
            </div>
            <div className="info-item">
              <span className="info-label">Transports:</span>
              <span className="info-value">{deviceInfo.transports.join(', ')}</span>
            </div>
            <div className="info-item">
              <span className="info-label">Algorithms:</span>
              <span className="info-value">{deviceInfo.algorithms.join(', ')}</span>
            </div>
            <div className="info-item">
              <span className="info-label">Max Message Size:</span>
              <span className="info-value">{deviceInfo.max_msg_size || 'N/A'} bytes</span>
            </div>
            <div className="info-item">
              <span className="info-label">Resident Key:</span>
              <span className="info-value">{deviceInfo.options.rk ? '✓ Yes' : '✗ No'}</span>
            </div>
            <div className="info-item">
              <span className="info-label">User Verification:</span>
              <span className="info-value">
                {deviceInfo.options.uv === true ? '✓ Yes' : deviceInfo.options.uv === false ? '✗ No' : 'N/A'}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* PIN Management */}
      <div className="card">
        <h2>PIN Management</h2>
        
        {pinRetries && (
          <div className="pin-status">
            <span>PIN Retries Remaining: <strong>{pinRetries.retries}</strong></span>
            {pinRetries.power_cycle_required && (
              <span className="warning">Power cycle required</span>
            )}
          </div>
        )}
        
        {/* Show only one button based on whether PIN is set */}
        <div className="button-group">
          {deviceInfo?.options.client_pin ? (
            <button onClick={() => setShowChangePin(!showChangePin)} className="btn btn-primary">
              Change PIN
            </button>
          ) : (
            <button onClick={() => setShowSetPin(!showSetPin)} className="btn btn-primary">
              Set New PIN
            </button>
          )}
        </div>

        {showSetPin && (
          <div className="pin-form">
            <h3>Set New PIN</h3>
            <input
              type="password"
              placeholder="New PIN (min 4 characters)"
              value={newPin}
              onChange={(e) => setNewPin(e.target.value)}
              disabled={loading}
            />
            <input
              type="password"
              placeholder="Confirm PIN"
              value={confirmPin}
              onChange={(e) => setConfirmPin(e.target.value)}
              disabled={loading}
            />
            <div className="button-group">
              <button onClick={handleSetPin} disabled={loading} className="btn btn-primary">
                {loading ? 'Setting...' : 'Set PIN'}
              </button>
              <button onClick={() => setShowSetPin(false)} disabled={loading} className="btn btn-secondary">
                Cancel
              </button>
            </div>
          </div>
        )}

        {showChangePin && (
          <div className="pin-form">
            <h3>Change PIN</h3>
            <input
              type="password"
              placeholder="Current PIN"
              value={currentPin}
              onChange={(e) => setCurrentPin(e.target.value)}
              disabled={loading}
            />
            <input
              type="password"
              placeholder="New PIN (min 4 characters)"
              value={newPin}
              onChange={(e) => setNewPin(e.target.value)}
              disabled={loading}
            />
            <input
              type="password"
              placeholder="Confirm New PIN"
              value={confirmPin}
              onChange={(e) => setConfirmPin(e.target.value)}
              disabled={loading}
            />
            <div className="button-group">
              <button onClick={handleChangePin} disabled={loading} className="btn btn-primary">
                {loading ? 'Changing...' : 'Change PIN'}
              </button>
              <button onClick={() => setShowChangePin(false)} disabled={loading} className="btn btn-secondary">
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Credentials */}
      <div className="card">
        <h2>Stored Credentials</h2>
        
        {credentials.length === 0 ? (
          <div className="empty-state-small">
            <p>No credentials stored</p>
            <p className="empty-hint">Credentials will appear here after you register with websites using FIDO2</p>
          </div>
        ) : (
          <div className="credential-list">
            {credentials.map((cred, index) => (
              <div key={index} className="credential-item">
                <div className="credential-info">
                  <div className="credential-main">
                    <span className="credential-rp">{cred.rp_name || cred.rp_id}</span>
                    <span className="credential-user">{cred.user_display_name || cred.user_name}</span>
                  </div>
                  <div className="credential-id">{cred.credential_id.substring(0, 32)}...</div>
                </div>
                <button
                  onClick={() => handleDeleteCredential(cred.credential_id)}
                  className="btn btn-danger btn-small"
                  disabled={loading}
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Device Reset */}
      <div className="card danger-zone">
        <h2>Danger Zone</h2>
        <p>Resetting the device will permanently delete all credentials and the PIN.</p>
        
        {!showResetConfirm ? (
          <button onClick={() => setShowResetConfirm(true)} className="btn btn-danger">
            Reset Device
          </button>
        ) : (
          <div className="reset-confirm">
            <p>Type <strong>RESET</strong> to confirm:</p>
            <input
              type="text"
              value={resetConfirmText}
              onChange={(e) => setResetConfirmText(e.target.value)}
              placeholder="Type RESET"
              disabled={loading}
            />
            <div className="button-group">
              <button
                onClick={handleResetDevice}
                disabled={loading || resetConfirmText !== 'RESET'}
                className="btn btn-danger"
              >
                {loading ? 'Resetting...' : 'Confirm Reset'}
              </button>
              <button
                onClick={() => {
                  setShowResetConfirm(false)
                  setResetConfirmText('')
                }}
                disabled={loading}
                className="btn btn-secondary"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
      
      {/* Device Re-insertion Prompt Modal */}
      {showReinsertPrompt && (
        <div className="modal-overlay" onClick={() => setShowReinsertPrompt(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h2>Device Re-insertion Required</h2>
            <p>
              For security purposes, PIN operations require you to re-insert your security key.
            </p>
            <ol>
              <li>Remove your security key from the USB port</li>
              <li>Wait 2 seconds</li>
              <li>Re-insert the security key</li>
              <li>Click "Continue" below</li>
            </ol>
            <div className="button-group">
              <button 
                onClick={() => {
                  if (reinsertAction === 'setPin') {
                    executeSetPin()
                  } else if (reinsertAction === 'changePin') {
                    executeChangePin()
                  }
                  setReinsertAction(null)
                }}
                className="btn btn-primary"
                disabled={loading}
              >
                {loading ? 'Processing...' : 'Continue'}
              </button>
              <button 
                onClick={() => {
                  setShowReinsertPrompt(false)
                  setReinsertAction(null)
                }}
                className="btn btn-secondary"
                disabled={loading}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
