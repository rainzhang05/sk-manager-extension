import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import './styles/App.css'

// Placeholder pages - will be implemented in later phases
const Dashboard = () => <div className="page"><h1>Dashboard</h1><p>Device overview and quick actions</p></div>
const DeviceManager = () => <div className="page"><h1>Device Manager</h1><p>Connect and manage devices</p></div>
const FIDO2Manager = () => <div className="page"><h1>FIDO2 Manager</h1><p>PIN and credential management</p></div>
const U2FManager = () => <div className="page"><h1>U2F Manager</h1><p>Legacy U2F support</p></div>
const PIVManager = () => <div className="page"><h1>PIV Manager</h1><p>Certificate and key management</p></div>
const OpenPGPManager = () => <div className="page"><h1>OpenPGP Manager</h1><p>OpenPGP card operations</p></div>
const OTPManager = () => <div className="page"><h1>OTP Manager</h1><p>HOTP configuration</p></div>
const NDEFManager = () => <div className="page"><h1>NDEF Manager</h1><p>NFC data management</p></div>
const DebugConsole = () => <div className="page"><h1>Debug Console</h1><p>Raw HID/APDU communication</p></div>
const Settings = () => <div className="page"><h1>Settings</h1><p>Application settings</p></div>

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
            <li><a href="/device">Device Manager</a></li>
            <li><a href="/fido2">FIDO2</a></li>
            <li><a href="/u2f">U2F</a></li>
            <li><a href="/piv">PIV</a></li>
            <li><a href="/openpgp">OpenPGP</a></li>
            <li><a href="/otp">OTP</a></li>
            <li><a href="/ndef">NDEF</a></li>
            <li><a href="/debug">Debug Console</a></li>
            <li><a href="/settings">Settings</a></li>
          </ul>
        </nav>
        <main className="main-content">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/device" element={<DeviceManager />} />
            <Route path="/fido2" element={<FIDO2Manager />} />
            <Route path="/u2f" element={<U2FManager />} />
            <Route path="/piv" element={<PIVManager />} />
            <Route path="/openpgp" element={<OpenPGPManager />} />
            <Route path="/otp" element={<OTPManager />} />
            <Route path="/ndef" element={<NDEFManager />} />
            <Route path="/debug" element={<DebugConsole />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App
