import { useState, useEffect } from 'react'
import EditEmployeeModal from '../components/EditEmployeeModal'
import DeleteConfirmModal from '../components/DeleteConfirmModal'

interface Employee {
  id: number
  name: string
  role: string
  device_id: string | null
  hourly_rate: string
  created_at: string
  updated_at: string
}

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3001'

interface EmployeePageProps {
  token: string | null
}

function EmployeePage({ token }: EmployeePageProps) {
  const [employees, setEmployees] = useState<Employee[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Employee form
  const [name, setName] = useState('')
  const [role, setRole] = useState('Staff')
  const [hourlyRate, setHourlyRate] = useState('')

  // Edit modal state
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null)
  const [editName, setEditName] = useState('')
  const [editRole, setEditRole] = useState('Staff')
  const [editHourlyRate, setEditHourlyRate] = useState('')

  // Delete confirm state
  const [deletingEmployee, setDeletingEmployee] = useState<Employee | null>(null)

  useEffect(() => {
    fetchEmployees()
  }, [])

  const fetchEmployees = async () => {
    try {
      setLoading(true)
      const res = await fetch(`${API_BASE}/employees`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      setEmployees(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể tải danh sách nhân viên')
    } finally {
      setLoading(false)
    }
  }

  const handleEditEmployee = (emp: Employee) => {
    setEditingEmployee(emp)
    setEditName(emp.name)
    setEditRole(emp.role)
    setEditHourlyRate(emp.hourly_rate)
  }

  const handleSaveEdit = async () => {
    if (!editingEmployee || !token) return
    try {
      const res = await fetch(`${API_BASE}/employees/${editingEmployee.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({
          name: editName,
          role: editRole,
          hourly_rate: editHourlyRate,
        }),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      setEditingEmployee(null)
      await fetchEmployees()
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể cập nhật nhân viên')
    }
  }

  const handleDeleteEmployee = async () => {
    if (!deletingEmployee || !token) return
    try {
      const res = await fetch(`${API_BASE}/employees/${deletingEmployee.id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      setDeletingEmployee(null)
      await fetchEmployees()
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể xóa nhân viên')
    }
  }

  const createEmployee = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!token) {
      setError('Vui lòng đăng nhập trước')
      return
    }
    try {
      const res = await fetch(`${API_BASE}/employees`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({ name, role, hourly_rate: hourlyRate }),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      setName('')
      setHourlyRate('')
      await fetchEmployees()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể tạo nhân viên')
    }
  }

  return (
    <div style={{ padding: '2rem', paddingTop: '1.5rem' }}>
      {error && (
        <div style={{ color: 'red', marginBottom: '1rem' }}>
          Lỗi: {error}
          <button onClick={() => setError(null)} style={{ marginLeft: '1rem' }}>
            Đóng
          </button>
        </div>
      )}

      {/* Add Employee Section (only visible when logged in) */}
      {token && (
        <form onSubmit={createEmployee} style={{ marginBottom: '2rem' }}>
          <h2>Thêm nhân viên</h2>
          <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
            <input
              placeholder="Họ và tên"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
            <select value={role} onChange={(e) => setRole(e.target.value)}>
              <option value="Staff">Nhân viên</option>
              <option value="Manager">Quản lý</option>
              <option value="Admin">Quản trị viên</option>
            </select>
            <input
              placeholder="Lương/giờ"
              type="number"
              step="0.01"
              value={hourlyRate}
              onChange={(e) => setHourlyRate(e.target.value)}
              required
            />
            <button type="submit">Tạo mới</button>
          </div>
        </form>
      )}

      {/* Employees Table */}
      <h2 style={{ margin: 0, marginBottom: '0.5rem' }}>Danh sách nhân viên</h2>
      {loading ? (
        <p>Đang tải...</p>
      ) : employees.length === 0 ? (
        <p>Chưa có nhân viên nào.</p>
      ) : (
        <table border={1} cellPadding={8} style={{ borderCollapse: 'collapse', width: '100%' }}>
          <thead>
            <tr>
              <th>ID</th>
              <th>Họ và tên</th>
              <th>Vai trò</th>
              <th>Lương/giờ</th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            {employees.map((emp) => (
              <tr key={emp.id}>
                <td>{emp.id}</td>
                <td><strong>{emp.name}</strong></td>
                <td>{emp.role}</td>
                <td>{emp.hourly_rate}</td>
                <td>
                  <div style={{ display: 'flex', gap: '4px' }}>
                    {token && (
                      <>
                        <button
                          onClick={() => handleEditEmployee(emp)}
                          style={{
                            padding: '0.25rem 0.75rem',
                            backgroundColor: '#4CAF50',
                            color: 'white',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                          }}
                        >
                          Sửa
                        </button>
                        <button
                          onClick={() => setDeletingEmployee(emp)}
                          style={{
                            padding: '0.25rem 0.75rem',
                            backgroundColor: '#f44336',
                            color: 'white',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                          }}
                        >
                          Xóa
                        </button>
                      </>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {/* Edit Employee Modal */}
      {editingEmployee && (
        <EditEmployeeModal
          employee={editingEmployee}
          name={editName}
          role={editRole}
          hourlyRate={editHourlyRate}
          onNameChange={setEditName}
          onRoleChange={setEditRole}
          onHourlyRateChange={setEditHourlyRate}
          onSave={handleSaveEdit}
          onClose={() => setEditingEmployee(null)}
        />
      )}

      {/* Delete Confirm Modal */}
      {deletingEmployee && (
        <DeleteConfirmModal
          employee={deletingEmployee}
          onConfirm={handleDeleteEmployee}
          onCancel={() => setDeletingEmployee(null)}
        />
      )}
    </div>
  )
}

export default EmployeePage
