import { useNavigate, useLocation } from 'react-router-dom'
import LoginPopup from './LoginPopup'

interface NavbarProps {
  token: string | null
  onLogin: (token: string) => void
  onLogout: () => void
}

function Navbar({ token, onLogin, onLogout }: NavbarProps) {
  const navigate = useNavigate()
  const location = useLocation()

  const isActive = (path: string) => location.pathname === path

  return (
    <header
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '8px 16px',
        background: '#f5f5f5',
        borderBottom: '1px solid #ddd',
        position: 'sticky',
        top: 0,
        zIndex: 100,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
        <h1 style={{ margin: 0, fontSize: '18px', color: '#333', cursor: 'pointer' }}
            onClick={() => navigate('/')}>
          TimeKeeping
        </h1>
        <nav style={{ display: 'flex', gap: '8px' }}>
          <button
            onClick={() => navigate('/')}
            style={{
              padding: '4px 12px',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px',
              backgroundColor: isActive('/') ? '#1976D2' : 'transparent',
              color: isActive('/') ? 'white' : '#333',
            }}
          >
            Chấm công
          </button>
          {token && (
            <button
              onClick={() => navigate('/employees')}
              style={{
                padding: '4px 12px',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '14px',
                backgroundColor: isActive('/employees') ? '#1976D2' : 'transparent',
                color: isActive('/employees') ? 'white' : '#333',
              }}
            >
              Nhân viên
            </button>
          )}
        </nav>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        {token && (
          <>
            <span style={{ fontSize: '12px', color: '#4CAF50' }}>Admin ✓</span>
            <button
              onClick={onLogout}
              style={{
                background: 'none',
                border: '1px solid #ccc',
                borderRadius: '4px',
                padding: '4px 8px',
                cursor: 'pointer',
                fontSize: '12px',
                color: '#666',
              }}
            >
              Đăng xuất
            </button>
          </>
        )}
        {!token && <LoginPopup onLogin={onLogin} />}
      </div>
    </header>
  )
}

export default Navbar
