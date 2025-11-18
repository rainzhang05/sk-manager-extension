import '../styles/Protocols.css'

export default function Protocols() {
  const protocols = [
    {
      id: 'fido2',
      name: 'FIDO2',
      subtitle: 'CTAP2',
      description: 'Modern authentication protocol with biometric support',
      icon: '🛡️',
      supported: false,
    },
    {
      id: 'u2f',
      name: 'U2F',
      subtitle: 'CTAP1',
      description: 'Legacy universal second factor authentication',
      icon: '🔐',
      supported: false,
    },
    {
      id: 'piv',
      name: 'PIV',
      subtitle: 'Smart Card',
      description: 'Personal identity verification for secure access',
      icon: '🎫',
      supported: false,
    },
    {
      id: 'openpgp',
      name: 'OpenPGP',
      subtitle: 'Email Security',
      description: 'Email encryption and digital signatures',
      icon: '📧',
      supported: false,
    },
    {
      id: 'otp',
      name: 'OTP',
      subtitle: 'HOTP',
      description: 'One-time password generation',
      icon: '🔢',
      supported: false,
    },
    {
      id: 'ndef',
      name: 'NDEF',
      subtitle: 'NFC',
      description: 'NFC data exchange format',
      icon: '📡',
      supported: false,
    },
  ]

  return (
    <div className="page">
      <div className="page-header">
        <h1>Protocols</h1>
        <p className="page-description">
          Detect and manage supported protocols on your Feitian security key
        </p>
      </div>

      <div className="protocols-notice">
        <span className="notice-icon">ℹ️</span>
        <div>
          <strong>Connect a device to detect protocols</strong>
          <p>
            Protocol detection will be available after connecting a Feitian security key.
            Toggle switches will allow you to enable/disable supported protocols.
          </p>
        </div>
      </div>

      <div className="protocols-grid">
        {protocols.map((protocol) => (
          <div key={protocol.id} className={`protocol-card ${protocol.supported ? 'supported' : 'unsupported'}`}>
            <div className="protocol-header">
              <span className="protocol-icon">{protocol.icon}</span>
              <div className="protocol-badge">
                {protocol.supported ? 'Supported' : 'Not Supported'}
              </div>
            </div>
            <div className="protocol-body">
              <h3 className="protocol-name">{protocol.name}</h3>
              <p className="protocol-subtitle">{protocol.subtitle}</p>
              <p className="protocol-description">{protocol.description}</p>
            </div>
            <div className="protocol-footer">
              <label className="toggle-switch" title="Connect a device first">
                <input type="checkbox" disabled={true} />
                <span className="toggle-slider"></span>
              </label>
              <span className="toggle-label">
                {protocol.supported ? 'Enabled' : 'Disabled'}
              </span>
            </div>
          </div>
        ))}
      </div>

      <div className="protocols-help">
        <h3>About Protocol Detection</h3>
        <p>
          Actual protocol detection will be implemented in Phase 5. The system will:
        </p>
        <ul>
          <li><strong>FIDO2:</strong> Try CTAP2 getInfo command</li>
          <li><strong>U2F:</strong> Try CTAP1 version command</li>
          <li><strong>PIV:</strong> Try PIV SELECT APDU</li>
          <li><strong>OpenPGP:</strong> Try OpenPGP SELECT APDU</li>
          <li><strong>OTP:</strong> Try OTP vendor command</li>
          <li><strong>NDEF:</strong> Try NDEF read command</li>
        </ul>
      </div>
    </div>
  )
}
