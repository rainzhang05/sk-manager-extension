import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import './styles/App.css'

// Placeholder pages - will be implemented in later phases
const Dashboard = () => (
  <div className="page">
    <h1>Dashboard</h1>
    <p>Device overview, connection status, and quick actions</p>
  </div>
)

const FIDO2Manager = () => (
  <div className="page">
    <h1>FIDO2 Manager</h1>
    <p>PIN and credential management, U2F support</p>
  </div>
)

const PIVManager = () => (
  <div className="page">
    <h1>PIV Manager</h1>
    <p>Certificate and key management</p>
  </div>
)

const OTPManager = () => (
  <div className="page">
    <h1>OTP Manager</h1>
    <p>HOTP configuration</p>
  </div>
)

const Protocols = () => (
  <div className="page">
    <h1>Protocols</h1>
    <p>Detect and enable/disable supported protocols</p>
    <div className="card" style={{ marginTop: '24px' }}>
      <h2>Protocol Detection</h2>
      <p style={{ marginBottom: '16px' }}>
        Connect a Feitian security key to detect supported protocols.
      </p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>FIDO2 (CTAP2)</strong> - Modern authentication protocol
        </div>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>U2F (CTAP1)</strong> - Legacy authentication protocol
        </div>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>PIV</strong> - Smart card authentication
        </div>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>OpenPGP</strong> - Email encryption and signing
        </div>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>OTP</strong> - One-time password generation
        </div>
        <div style={{ padding: '12px', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-sm)' }}>
          <strong>NDEF</strong> - NFC data exchange
        </div>
      </div>
    </div>
  </div>
)

function App() {
  return (
    <Router>
      <div className="app">
        <nav className="sidebar">
          <div className="sidebar-header">
            <h1>Feitian SK Manager</h1>
          </div>
          <ul className="nav-menu">
            <li><a href="/">Dashboard</a></li>
            <li><a href="/fido2">FIDO2</a></li>
            <li><a href="/piv">PIV</a></li>
            <li><a href="/otp">OTP</a></li>
            <li><a href="/protocols">Protocols</a></li>
          </ul>
        </nav>
        <main className="main-content">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/fido2" element={<FIDO2Manager />} />
            <Route path="/piv" element={<PIVManager />} />
            <Route path="/otp" element={<OTPManager />} />
            <Route path="/protocols" element={<Protocols />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App
