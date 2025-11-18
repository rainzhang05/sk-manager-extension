import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Dashboard, FIDO2, Protocols, DebugConsole } from './pages'
import './styles/App.css'

// Placeholder pages - will be implemented in later phases
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
            <li><a href="/debug">Debug</a></li>
          </ul>
        </nav>
        <main className="main-content">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/fido2" element={<FIDO2 />} />
            <Route path="/piv" element={<PIVManager />} />
            <Route path="/otp" element={<OTPManager />} />
            <Route path="/protocols" element={<Protocols />} />
            <Route path="/debug" element={<DebugConsole />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App
