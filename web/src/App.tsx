import { useState, useEffect } from 'react'
import { Dashboard, FIDO2, Protocols, DebugConsole } from './pages'
import DeviceList from './components/DeviceList'
import { connectionManager } from './services/ConnectionManager'
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

type ViewType = 'dashboard' | 'fido2' | 'piv' | 'otp' | 'protocols' | 'debug'

function App() {
  const [activeView, setActiveView] = useState<ViewType>('dashboard')

  // Initialize connection manager at app level
  useEffect(() => {
    console.log('[App] Initializing connection manager')
    connectionManager.refreshConnections()
  }, [])

  const renderView = () => {
    switch (activeView) {
      case 'dashboard':
        return <Dashboard />
      case 'fido2':
        return <FIDO2 />
      case 'piv':
        return <PIVManager />
      case 'otp':
        return <OTPManager />
      case 'protocols':
        return <Protocols />
      case 'debug':
        return <DebugConsole />
      default:
        return <Dashboard />
    }
  }

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="sidebar-header">
          <h1>Feitian SK Manager</h1>
        </div>
        <ul className="nav-menu">
          <li>
            <button 
              className={activeView === 'dashboard' ? 'active' : ''}
              onClick={() => setActiveView('dashboard')}
            >
              Dashboard
            </button>
          </li>
          <li>
            <button 
              className={activeView === 'fido2' ? 'active' : ''}
              onClick={() => setActiveView('fido2')}
            >
              FIDO2
            </button>
          </li>
          <li>
            <button 
              className={activeView === 'piv' ? 'active' : ''}
              onClick={() => setActiveView('piv')}
            >
              PIV
            </button>
          </li>
          <li>
            <button 
              className={activeView === 'otp' ? 'active' : ''}
              onClick={() => setActiveView('otp')}
            >
              OTP
            </button>
          </li>
          <li>
            <button 
              className={activeView === 'protocols' ? 'active' : ''}
              onClick={() => setActiveView('protocols')}
            >
              Protocols
            </button>
          </li>
          <li>
            <button 
              className={activeView === 'debug' ? 'active' : ''}
              onClick={() => setActiveView('debug')}
            >
              Debug
            </button>
          </li>
        </ul>
      </nav>
      <main className="main-content">
        {/* DeviceList component persists across all views */}
        <div className="persistent-device-list">
          <DeviceList />
        </div>
        
        {/* Active view content */}
        <div className="view-content">
          {renderView()}
        </div>
      </main>
    </div>
  )
}

export default App
