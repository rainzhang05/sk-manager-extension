import { useState, useEffect } from 'react'
import '../styles/FIDO2.css'

interface PivDiscovery {
  piv_card_application_aid: string | null
  pin_usage_policy: string | null
}

interface PivCertificate {
  slot: string
  slot_name: string
  present: boolean
  certificate_data: string | null
  subject: string | null
  issuer: string | null
  serial_number: string | null
  not_before: string | null
  not_after: string | null
}

interface PivInfo {
  selected: boolean
  chuid: string | null
  discovery: PivDiscovery | null
  certificates: PivCertificate[]
}

interface ApduLog {
  command: string
  command_hex: string
  response_hex: string
  sw1: number
  sw2: number
  status: string
  description: string
}

export default function PIV() {
  const [connectedDevice, setConnectedDevice] = useState<string | null>(null)
  const [pivInfo, setPivInfo] = useState<PivInfo | null>(null)
  const [activityLog, setActivityLog] = useState<ApduLog[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  useEffect(() => {
    console.log('[PIV] Component mounted, checking for device')

    // Check if there's already a connected device on mount
    const storedDeviceId = sessionStorage.getItem('connectedDeviceId')
    console.log('[PIV] Checking sessionStorage for device:', storedDeviceId)

    if (storedDeviceId) {
      console.log('[PIV] Found existing connected device:', storedDeviceId)
      setConnectedDevice(storedDeviceId)
      loadPivData(storedDeviceId)
    } else {
      console.log('[PIV] No device found in sessionStorage')
    }

    // Listen for device connection events
    const handleDeviceConnected = (event: Event) => {
      const customEvent = event as CustomEvent
      const deviceId = customEvent.detail.deviceId
      console.log('[PIV] Device connected event received:', deviceId)
      setConnectedDevice(deviceId)
      loadPivData(deviceId)
    }

    const handleDeviceDisconnected = () => {
      console.log('[PIV] Device disconnected event')
      setConnectedDevice(null)
      setPivInfo(null)
      setActivityLog([])
    }

    window.addEventListener('device-connected', handleDeviceConnected)
    window.addEventListener('device-disconnected', handleDeviceDisconnected)

    return () => {
      console.log('[PIV] Component unmounting, cleaning up listeners')
      window.removeEventListener('device-connected', handleDeviceConnected)
      window.removeEventListener('device-disconnected', handleDeviceDisconnected)
    }
  }, [])

  const loadPivData = async (deviceId: string) => {
    console.log('[PIV] loadPivData called with deviceId:', deviceId)
    setLoading(true)
    setError(null)
    setSuccessMessage(null)
    setActivityLog([])

    try {
      if (!window.chromeBridge) {
        console.error('[PIV] chromeBridge not available')
        setError('Chrome extension bridge not available. Please refresh the page.')
        setLoading(false)
        return
      }

      // Check device type first
      console.log('[PIV] Fetching device list to check device type...')
      const deviceListResponse = await window.chromeBridge.send('listDevices', {})
      let deviceType = 'Hid' // default

      if (deviceListResponse.status === 'ok' && deviceListResponse.result) {
        const result = deviceListResponse.result as { devices?: Array<{id: string, device_type: string}> }
        const devices = result.devices || []
        const currentDevice = devices.find(d => d.id === deviceId)
        if (currentDevice) {
          deviceType = currentDevice.device_type
          console.log('[PIV] Device type:', deviceType)

          // PIV requires CCID device
          if (deviceType !== 'Ccid') {
            setError('Error sending command')
            setLoading(false)
            return
          }
        } else {
          // Device not found in list
          setError('Device not found. Please reconnect your device.')
          setLoading(false)
          return
        }
      } else {
        // Failed to get device list
        setError('Failed to get device list. Please try reconnecting your device.')
        setLoading(false)
        return
      }

      console.log('[PIV] Sending pivGetData command...')
      console.group('[PIV] PIV Command Execution')
      console.log('Command: pivGetData')
      console.log('Device ID:', deviceId)
      console.time('[PIV] pivGetData execution time')

      const response = await Promise.race([
        window.chromeBridge!.send('pivGetData', { deviceId }),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('PIV command timeout - device not responding')), 10000)
        )
      ]) as { status: string; result?: unknown; error?: { code: string; message: string } };

      console.timeEnd('[PIV] pivGetData execution time')
      console.log('[PIV] pivGetData response:', response)

      const typedResponse = response as { status: string; result?: unknown; error?: { code: string; message: string } };
      
      if (typedResponse.status === 'ok' && typedResponse.result) {
        const result = typedResponse.result as {
          info: PivInfo
          activityLog: ApduLog[]
        }

        console.log('[PIV] PIV info loaded successfully:', result.info)
        setPivInfo(result.info)
        setActivityLog(result.activityLog || [])

        // Log detailed activity to console
        console.group('[PIV] APDU Activity Log')
        if (result.activityLog && result.activityLog.length > 0) {
          result.activityLog.forEach((log, index) => {
            console.group(`[PIV] Command ${index + 1}: ${log.command}`)
            console.log('%cAPDU Command:', 'color: #2196F3; font-weight: bold', log.command_hex)
            console.log('%cResponse:', 'color: #4CAF50; font-weight: bold', log.response_hex)
            console.log('SW1 SW2:', `0x${log.sw1.toString(16).toUpperCase().padStart(2, '0')} 0x${log.sw2.toString(16).toUpperCase().padStart(2, '0')}`)
            console.log('Status:', log.status)
            console.log('Description:', log.description)
            console.groupEnd()
          })
        } else {
          console.log('No APDU commands logged')
        }
        console.groupEnd()

        setSuccessMessage(`PIV data loaded successfully. ${result.activityLog?.length || 0} APDU commands executed.`)
      } else {
        const errorMsg = typedResponse.error?.message || 'Failed to get PIV data'
        console.error('[PIV] Failed to get PIV data:', errorMsg)

        // Provide helpful error messages
        if (errorMsg.includes('timeout')) {
          setError('Device did not respond to PIV commands. This device may not support PIV, or the card may not be present in the reader.')
        } else if (errorMsg.includes('APDU error')) {
          setError('The device returned an APDU error. The PIV application may not be available or the card needs to be reset.')
        } else if (errorMsg.includes('DEVICE_TYPE_MISMATCH')) {
          setError('PIV operations require a CCID smart card device. The connected device is a HID device which is used for FIDO2. Please connect a smart card reader or a device with CCID interface for PIV operations.')
        } else {
          setError(errorMsg)
        }
      }

      console.groupEnd() // End PIV Command Execution group
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error'
      console.error('[PIV] Exception in loadPivData:', errorMsg)
      if (errorMsg.includes('timeout')) {
        setError('Communication timeout. Please ensure the device is properly connected and supports PIV.')
      } else if (errorMsg.includes('not found')) {
        setError('Device not found. Please reconnect your device.')
      } else {
        setError(`Error loading PIV data: ${errorMsg}`)
      }
    } finally {
      setLoading(false)
    }
  }

  const handleRefresh = () => {
    if (connectedDevice) {
      console.log('[PIV] Manual refresh requested')
      loadPivData(connectedDevice)
    }
  }

  const formatHexDisplay = (hex: string | null, maxLength: number = 50): string => {
    if (!hex) return 'N/A'
    if (hex.length <= maxLength) return hex
    return hex.substring(0, maxLength) + '...'
  }

  return (
    <div className="fido2-page">
      <div className="page-header">
        <h1>PIV Manager</h1>
        <p>Personal Identity Verification card management and APDU activity logging</p>
      </div>

      {loading && (
        <div className="loading-card">
          <div className="spinner"></div>
          <p>Loading PIV data...</p>
          <p className="loading-hint">Communicating with the authenticator via APDU commands...</p>
        </div>
      )}

      {error && (
        <div className="message-card error">
          <div className="message-header">
            <span className="message-icon">!</span>
            <span>Error</span>
            <button className="close-btn" onClick={() => setError(null)}>x</button>
          </div>
          <p style={{ whiteSpace: 'pre-line' }}>{error}</p>
        </div>
      )}

      {successMessage && (
        <div className="message-card success">
          <div className="message-header">
            <span className="message-icon">+</span>
            <span>Success</span>
            <button className="close-btn" onClick={() => setSuccessMessage(null)}>x</button>
          </div>
          <p>{successMessage}</p>
        </div>
      )}

      {!connectedDevice && !loading && (
        <div className="no-device-card">
          <h3>No Device Connected</h3>
          <p>Connect a CCID smart card device to manage PIV certificates and view APDU command logs.</p>
        </div>
      )}

      {pivInfo && (
        <>
          {/* PIV Status */}
          <div className="info-card">
            <h3>PIV Application Status</h3>
            <div className="info-grid">
              <div className="info-item">
                <label>Application Selected</label>
                <span className={pivInfo.selected ? 'status-ok' : 'status-error'}>
                  {pivInfo.selected ? 'Yes' : 'No'}
                </span>
              </div>
              <div className="info-item">
                <label>CHUID (Card Holder Unique ID)</label>
                <span className="code-text">{pivInfo.chuid || 'Not available'}</span>
              </div>
            </div>
          </div>

          {/* Discovery Object */}
          {pivInfo.discovery && (
            <div className="info-card">
              <h3>Discovery Object</h3>
              <div className="info-grid">
                <div className="info-item">
                  <label>PIV Card Application AID</label>
                  <span className="code-text">
                    {pivInfo.discovery.piv_card_application_aid || 'Not available'}
                  </span>
                </div>
                <div className="info-item">
                  <label>PIN Usage Policy</label>
                  <span className="code-text">
                    {pivInfo.discovery.pin_usage_policy || 'Not available'}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Certificates */}
          <div className="info-card">
            <h3>Certificates</h3>
            <div className="certificates-list">
              {pivInfo.certificates.map((cert) => (
                <div key={cert.slot} className={`certificate-item ${cert.present ? 'present' : 'empty'}`}>
                  <div className="cert-header">
                    <span className="cert-slot">Slot {cert.slot}</span>
                    <span className="cert-name">{cert.slot_name}</span>
                    <span className={`cert-status ${cert.present ? 'present' : 'empty'}`}>
                      {cert.present ? 'Present' : 'Empty'}
                    </span>
                  </div>
                  {cert.present && cert.certificate_data && (
                    <div className="cert-details">
                      <span className="cert-data-preview">
                        Certificate data: {formatHexDisplay(cert.certificate_data, 60)}
                      </span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* APDU Activity Log */}
          <div className="info-card">
            <h3>
              APDU Activity Log
              <button
                className="refresh-btn"
                onClick={handleRefresh}
                disabled={loading}
                style={{ marginLeft: '10px', fontSize: '12px', padding: '4px 8px' }}
              >
                Refresh
              </button>
            </h3>
            <p className="section-hint">
              All PIV commands are logged to the browser console for detailed inspection.
              Open Developer Tools (F12) to view complete APDU traces.
            </p>
            {activityLog.length > 0 ? (
              <div className="activity-log">
                <table className="apdu-table">
                  <thead>
                    <tr>
                      <th>#</th>
                      <th>Command</th>
                      <th>APDU (hex)</th>
                      <th>SW</th>
                      <th>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activityLog.map((log, index) => (
                      <tr key={index} className={`status-${log.status.toLowerCase()}`}>
                        <td>{index + 1}</td>
                        <td>{log.command}</td>
                        <td className="apdu-hex">
                          <span title={log.command_hex}>
                            {formatHexDisplay(log.command_hex, 30)}
                          </span>
                        </td>
                        <td className="sw-code">
                          {log.sw1.toString(16).toUpperCase().padStart(2, '0')}
                          {' '}
                          {log.sw2.toString(16).toUpperCase().padStart(2, '0')}
                        </td>
                        <td>
                          <span className={`status-badge ${log.status.toLowerCase()}`} title={log.description}>
                            {log.status}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="no-activity">No APDU commands logged yet.</p>
            )}
          </div>
        </>
      )}

      <style>{`
        .certificates-list {
          display: flex;
          flex-direction: column;
          gap: 10px;
        }

        .certificate-item {
          border: 1px solid #ddd;
          border-radius: 6px;
          padding: 12px;
          background: #f9f9f9;
        }

        .certificate-item.present {
          border-color: #4CAF50;
          background: #f0fff0;
        }

        .certificate-item.empty {
          border-color: #ccc;
          opacity: 0.7;
        }

        .cert-header {
          display: flex;
          align-items: center;
          gap: 10px;
        }

        .cert-slot {
          font-weight: bold;
          color: #333;
        }

        .cert-name {
          flex: 1;
          color: #666;
        }

        .cert-status {
          font-size: 12px;
          padding: 2px 8px;
          border-radius: 4px;
        }

        .cert-status.present {
          background: #4CAF50;
          color: white;
        }

        .cert-status.empty {
          background: #ccc;
          color: #666;
        }

        .cert-details {
          margin-top: 8px;
          padding-top: 8px;
          border-top: 1px solid #ddd;
        }

        .cert-data-preview {
          font-family: monospace;
          font-size: 11px;
          color: #666;
          word-break: break-all;
        }

        .activity-log {
          overflow-x: auto;
        }

        .apdu-table {
          width: 100%;
          border-collapse: collapse;
          font-size: 12px;
        }

        .apdu-table th,
        .apdu-table td {
          padding: 8px;
          text-align: left;
          border-bottom: 1px solid #ddd;
        }

        .apdu-table th {
          background: #f5f5f5;
          font-weight: bold;
        }

        .apdu-hex {
          font-family: monospace;
          font-size: 11px;
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .sw-code {
          font-family: monospace;
          font-weight: bold;
        }

        .status-badge {
          display: inline-block;
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 10px;
          font-weight: bold;
        }

        .status-badge.ok {
          background: #4CAF50;
          color: white;
        }

        .status-badge.more_data {
          background: #2196F3;
          color: white;
        }

        .status-badge.error {
          background: #f44336;
          color: white;
        }

        .section-hint {
          font-size: 12px;
          color: #666;
          margin-bottom: 10px;
          font-style: italic;
        }

        .no-activity {
          color: #666;
          font-style: italic;
          text-align: center;
          padding: 20px;
        }

        .refresh-btn {
          background: #2196F3;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .refresh-btn:hover {
          background: #1976D2;
        }

        .refresh-btn:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        tr.status-ok {
          background: #f8fff8;
        }

        tr.status-error {
          background: #fff8f8;
        }

        tr.status-more_data {
          background: #f8faff;
        }

        .status-ok {
          color: #4CAF50;
          font-weight: bold;
        }

        .status-error {
          color: #f44336;
          font-weight: bold;
        }

        .code-text {
          font-family: monospace;
          font-size: 12px;
          word-break: break-all;
        }

        .loading-hint {
          font-size: 12px;
          color: #666;
          font-style: italic;
        }
      `}</style>
    </div>
  )
}
