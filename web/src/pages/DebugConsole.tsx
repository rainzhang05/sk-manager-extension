import { useState } from 'react'
import '../styles/DebugConsole.css'

export default function DebugConsole() {
  const [connectedDeviceId, setConnectedDeviceId] = useState('')
  const [hidData, setHidData] = useState('')
  const [apduData, setApduData] = useState('00 A4 04 00')
  const [timeout, setTimeout] = useState('5000')
  const [response, setResponse] = useState('')
  const [loading, setLoading] = useState(false)

  const parseHexString = (hex: string): number[] => {
    // Remove spaces and split into pairs
    const cleaned = hex.replace(/\s+/g, '')
    const bytes: number[] = []
    
    for (let i = 0; i < cleaned.length; i += 2) {
      const byte = cleaned.substr(i, 2)
      if (byte.length === 2) {
        const parsed = parseInt(byte, 16)
        if (!isNaN(parsed)) {
          bytes.push(parsed)
        }
      }
    }
    
    return bytes
  }

  const formatHexResponse = (data: number[]): string => {
    if (!data || data.length === 0) return 'No data'
    
    // Format as hex pairs
    const hexPairs = data.map(b => b.toString(16).padStart(2, '0').toUpperCase())
    
    // Group in rows of 16 bytes
    const rows: string[] = []
    for (let i = 0; i < hexPairs.length; i += 16) {
      const row = hexPairs.slice(i, i + 16).join(' ')
      rows.push(row)
    }
    
    return rows.join('\n')
  }

  const sendHid = async () => {
    if (!connectedDeviceId) {
      setResponse('ERROR: Please enter a connected device ID')
      return
    }

    if (!hidData) {
      setResponse('ERROR: Please enter HID data')
      return
    }

    try {
      setLoading(true)
      setResponse('Sending HID packet...')

      const dataBytes = parseHexString(hidData)
      if (dataBytes.length === 0) {
        setResponse('ERROR: Invalid hex data')
        return
      }

      const result = await window.chromeBridge.send('sendHid', {
        deviceId: connectedDeviceId,
        data: dataBytes
      })

      if (result.status === 'ok') {
        setResponse(`✅ HID packet sent successfully\nBytes sent: ${result.result.bytesSent}`)
      } else {
        setResponse(`❌ Error: ${result.error?.message || 'Unknown error'}`)
      }
    } catch (err) {
      setResponse(`❌ Error: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setLoading(false)
    }
  }

  const receiveHid = async () => {
    if (!connectedDeviceId) {
      setResponse('ERROR: Please enter a connected device ID')
      return
    }

    try {
      setLoading(true)
      setResponse('Receiving HID packet...')

      const timeoutMs = parseInt(timeout) || 5000

      const result = await window.chromeBridge.send('receiveHid', {
        deviceId: connectedDeviceId,
        timeout: timeoutMs
      })

      if (result.status === 'ok') {
        const formatted = formatHexResponse(result.result.data)
        setResponse(`✅ HID packet received\nBytes: ${result.result.data.length}\n\n${formatted}`)
      } else {
        setResponse(`❌ Error: ${result.error?.message || 'Unknown error'}`)
      }
    } catch (err) {
      setResponse(`❌ Error: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setLoading(false)
    }
  }

  const sendApdu = async () => {
    if (!connectedDeviceId) {
      setResponse('ERROR: Please enter a connected device ID')
      return
    }

    if (!apduData) {
      setResponse('ERROR: Please enter APDU data')
      return
    }

    try {
      setLoading(true)
      setResponse('Transmitting APDU...')

      const apduBytes = parseHexString(apduData)
      if (apduBytes.length < 4) {
        setResponse('ERROR: APDU must be at least 4 bytes (CLA INS P1 P2)')
        return
      }

      const result = await window.chromeBridge.send('transmitApdu', {
        deviceId: connectedDeviceId,
        apdu: apduBytes
      })

      if (result.status === 'ok') {
        const responseData = result.result.response
        const formatted = formatHexResponse(responseData)
        
        // Extract status word (last 2 bytes)
        const sw1 = responseData[responseData.length - 2]
        const sw2 = responseData[responseData.length - 1]
        const statusWord = `${sw1.toString(16).padStart(2, '0')}${sw2.toString(16).padStart(2, '0')}`.toUpperCase()
        const success = sw1 === 0x90 && sw2 === 0x00
        
        setResponse(
          `${success ? '✅' : '⚠️'} APDU Response\n` +
          `Bytes: ${responseData.length}\n` +
          `Status Word: ${statusWord} ${success ? '(Success)' : '(Error)'}\n\n${formatted}`
        )
      } else {
        setResponse(`❌ Error: ${result.error?.message || 'Unknown error'}`)
      }
    } catch (err) {
      setResponse(`❌ Error: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>Debug Console</h1>
        <p className="page-description">
          Send raw HID packets and APDUs to connected devices for testing and debugging
        </p>
      </div>

      <div className="debug-grid">
        <div className="debug-card">
          <h2>Device Connection</h2>
          <div className="form-group">
            <label htmlFor="deviceId">Device ID</label>
            <input
              id="deviceId"
              type="text"
              value={connectedDeviceId}
              onChange={(e) => setConnectedDeviceId(e.target.value)}
              placeholder="e.g., hid_1 or ccid_1"
              className="input-text"
            />
            <p className="help-text">
              Enter the device ID from the Dashboard (e.g., hid_1 for first HID device, ccid_1 for first CCID device).
              Make sure to connect the device first on the Dashboard page.
            </p>
          </div>
        </div>

        <div className="debug-card">
          <h2>HID Operations</h2>
          <div className="form-group">
            <label htmlFor="hidData">HID Data (hex)</label>
            <textarea
              id="hidData"
              value={hidData}
              onChange={(e) => setHidData(e.target.value)}
              placeholder="01 02 03 04 05 06 07 08"
              className="input-textarea"
              rows={3}
            />
            <p className="help-text">
              Enter hex bytes separated by spaces (max 64 bytes). Will be padded to 64 bytes automatically.
            </p>
          </div>
          
          <div className="form-group">
            <label htmlFor="timeout">Timeout (ms)</label>
            <input
              id="timeout"
              type="number"
              value={timeout}
              onChange={(e) => setTimeout(e.target.value)}
              className="input-text"
              min="100"
              max="30000"
            />
          </div>

          <div className="button-group">
            <button 
              onClick={sendHid} 
              disabled={loading || !connectedDeviceId}
              className="btn-primary"
            >
              Send HID
            </button>
            <button 
              onClick={receiveHid} 
              disabled={loading || !connectedDeviceId}
              className="btn-secondary"
            >
              Receive HID
            </button>
          </div>
        </div>

        <div className="debug-card">
          <h2>APDU Operations</h2>
          <div className="form-group">
            <label htmlFor="apduData">APDU Command (hex)</label>
            <textarea
              id="apduData"
              value={apduData}
              onChange={(e) => setApduData(e.target.value)}
              placeholder="00 A4 04 00"
              className="input-textarea"
              rows={3}
            />
            <p className="help-text">
              Enter APDU command as hex bytes (minimum: CLA INS P1 P2). Example: 00 A4 04 00 selects an application.
            </p>
          </div>

          <div className="button-group">
            <button 
              onClick={sendApdu} 
              disabled={loading || !connectedDeviceId}
              className="btn-primary"
            >
              Transmit APDU
            </button>
          </div>
        </div>

        <div className="debug-card response-card">
          <h2>Response</h2>
          <pre className="response-output">{response || 'No response yet. Send a command to see the response.'}</pre>
        </div>
      </div>
    </div>
  )
}
