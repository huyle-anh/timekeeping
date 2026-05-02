/// <reference types="vite/client" />
import { useState, useRef, useEffect } from 'react';
import { getApiBase, parseApiError, parseJsonResponse } from '../api';

const API_BASE = getApiBase();

interface LoginPopupProps {
  onLogin: (token: string) => void;
}

export default function LoginPopup({ onLogin }: LoginPopupProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const popupRef = useRef<HTMLDivElement>(null);

  // Close popup when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (popupRef.current && !popupRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setError('');
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await fetch(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        throw new Error(await parseApiError(response));
      }

      const data = await parseJsonResponse<{ token: string }>(response);
      onLogin(data.token);
      setIsOpen(false);
      setUsername('');
      setPassword('');
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ position: 'relative' }}>
      {/* Login button - small icon in top right */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        style={{
          background: 'none',
          border: '1px solid #ccc',
          borderRadius: '4px',
          padding: '4px 8px',
          cursor: 'pointer',
          fontSize: '14px',
          color: '#333',
        }}
        title="Đăng nhập quản trị"
      >
        🔑
      </button>

      {/* Popup */}
      {isOpen && (
        <div
          ref={popupRef}
          style={{
            position: 'absolute',
            top: '100%',
            right: 0,
            marginTop: '8px',
            background: '#fff',
            border: '1px solid #ddd',
            borderRadius: '8px',
            boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
            padding: '16px',
            minWidth: '280px',
            zIndex: 1000,
          }}
        >
          <h3 style={{ margin: '0 0 12px', fontSize: '16px', color: '#333' }}>
            Đăng nhập quản trị
          </h3>

          <form onSubmit={handleLogin}>
            <div style={{ marginBottom: '12px' }}>
              <label
                htmlFor="username"
                style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}
              >
                Tên đăng nhập
              </label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="admin"
                required
                style={{
                  width: '100%',
                  padding: '8px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  fontSize: '14px',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            <div style={{ marginBottom: '12px' }}>
              <label
                htmlFor="password"
                style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}
              >
                Mật khẩu
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                required
                style={{
                  width: '100%',
                  padding: '8px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  fontSize: '14px',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            {error && (
              <div
                style={{
                  color: '#d32f2f',
                  fontSize: '13px',
                  marginBottom: '8px',
                  padding: '6px',
                  background: '#fce4ec',
                  borderRadius: '4px',
                }}
              >
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              style={{
                width: '100%',
                padding: '8px',
                background: loading ? '#90caf9' : '#1976d2',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                fontSize: '14px',
                cursor: loading ? 'not-allowed' : 'pointer',
              }}
            >
              {loading ? 'Đang đăng nhập...' : 'Đăng nhập'}
            </button>
          </form>

          <div
            style={{
              marginTop: '12px',
              fontSize: '11px',
              color: '#999',
              textAlign: 'center',
              borderTop: '1px solid #eee',
              paddingTop: '8px',
            }}
          >
            Mặc định: admin / admin123
          </div>
        </div>
      )}
    </div>
  );
}
