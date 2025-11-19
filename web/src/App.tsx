import { BrowserRouter as Router, Routes, Route, Navigate, NavLink } from 'react-router-dom'
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
            <li><NavLink to="/" end>Dashboard</NavLink></li>
            <li><NavLink to="/fido2">FIDO2</NavLink></li>
            <li><NavLink to="/piv">PIV</NavLink></li>
            <li><NavLink to="/otp">OTP</NavLink></li>
            <li><NavLink to="/protocols">Protocols</NavLink></li>
            <li><NavLink to="/debug">Debug</NavLink></li>
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
