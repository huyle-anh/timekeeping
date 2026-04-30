/// <reference types="vite/client" />
import { useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import Navbar from './components/Navbar'
import AttendancePage from './pages/AttendancePage'
import EmployeePage from './pages/EmployeePage'

function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'))

  const handleLogin = (newToken: string) => {
    setToken(newToken)
    localStorage.setItem('token', newToken)
  }

  const handleLogout = () => {
    setToken(null)
    localStorage.removeItem('token')
  }

  return (
    <BrowserRouter>
      <div style={{ fontFamily: 'sans-serif' }}>
        <Navbar
          token={token}
          onLogin={handleLogin}
          onLogout={handleLogout}
        />
        <Routes>
          <Route path="/" element={<AttendancePage />} />
          <Route path="/employees" element={<EmployeePage token={token} />} />
        </Routes>
      </div>
    </BrowserRouter>
  )
}

export default App
