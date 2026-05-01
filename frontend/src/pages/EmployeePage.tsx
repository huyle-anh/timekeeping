import { useState, useEffect } from 'react'
import EditEmployeeModal from '../components/EditEmployeeModal'
import DeleteConfirmModal from '../components/DeleteConfirmModal'

interface Employee {
  id: number
  name: string
  role: string
  device_id: string | null
  pay_type: string
  hourly_rate: string | null
  monthly_salary: string | null
  hours_worked_this_month: number | null
  total_salary_this_month: string | null
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
  const [payType, setPayType] = useState('Hourly')
  const [hourlyRate, setHourlyRate] = useState('')
  const [monthlySalary, setMonthlySalary] = useState('')

  // Edit modal state
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null)
  const [editName, setEditName] = useState('')
  const [editRole, setEditRole] = useState('Staff')
  const [editPayType, setEditPayType] = useState('Hourly')
  const [editHourlyRate, setEditHourlyRate] = useState('')
  const [editMonthlySalary, setEditMonthlySalary] = useState('')

  // Month selector
  const todayYM = new Date().toISOString().slice(0, 7)
  const [hoursMonth, setHoursMonth] = useState<string>(todayYM)

  // Delete confirm state
  const [deletingEmployee, setDeletingEmployee] = useState<Employee | null>(null)

  useEffect(() => {
    fetchEmployees(hoursMonth)
  }, [hoursMonth])

  const fetchEmployees = async (month?: string) => {
    try {
      setLoading(true)
      const url = month
        ? `${API_BASE}/employees?month=${encodeURIComponent(month)}`
        : `${API_BASE}/employees`
      const res = await fetch(url)
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
    setEditPayType(emp.pay_type)
    setEditHourlyRate(emp.hourly_rate ?? '')
    setEditMonthlySalary(emp.monthly_salary ?? '')
  }

  const handleSaveEdit = async () => {
    if (!editingEmployee || !token) return
    try {
      const body: Record<string, unknown> = {
        name: editName,
        role: editRole,
        pay_type: editPayType,
      }
      if (editPayType === 'Hourly') {
        body.hourly_rate = editHourlyRate
      } else {
        body.monthly_salary = editMonthlySalary
      }
      const res = await fetch(`${API_BASE}/employees/${editingEmployee.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify(body),
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
      const body: Record<string, unknown> = {
        name,
        role,
        pay_type: payType,
      }
      if (payType === 'Hourly') {
        body.hourly_rate = hourlyRate
      } else {
        body.monthly_salary = monthlySalary
      }
      const res = await fetch(`${API_BASE}/employees`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify(body),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      setName('')
      setHourlyRate('')
      setMonthlySalary('')
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
          <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', alignItems: 'flex-end' }}>
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
            <select value={payType} onChange={(e) => setPayType(e.target.value)}>
              <option value="Hourly">Theo giờ</option>
              <option value="Salary">Lương tháng</option>
            </select>
            {payType === 'Hourly' ? (
              <input
                placeholder="Lương/giờ"
                type="number"
                step="0.01"
                value={hourlyRate}
                onChange={(e) => setHourlyRate(e.target.value)}
                required
              />
            ) : (
              <input
                placeholder="Lương tháng"
                type="number"
                step="0.01"
                value={monthlySalary}
                onChange={(e) => setMonthlySalary(e.target.value)}
                required
              />
            )}
            <button type="submit">Tạo mới</button>
          </div>
        </form>
      )}

      {/* Employees Table */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', marginBottom: '0.5rem' }}>
        <h2 style={{ margin: 0 }}>Danh sách nhân viên</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <label style={{ fontSize: '13px', color: '#666' }}>Tháng:</label>
          <input
            type="month"
            value={hoursMonth}
            onChange={(e) => setHoursMonth(e.target.value)}
            style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid #ccc', fontSize: '13px' }}
          />
        </div>
      </div>
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
              <th>Loại</th>
              <th>Mức lương</th>
              <th>Giờ làm tháng này</th>
              <th>Tổng lương tháng này</th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            {employees.map((emp) => {
              const hoursColor = emp.hours_worked_this_month !== null && emp.hours_worked_this_month >= 160
                ? '#2e7d32'
                : emp.hours_worked_this_month !== null && emp.hours_worked_this_month >= 80
                ? '#e65100'
                : emp.hours_worked_this_month !== null && emp.hours_worked_this_month > 0
                ? '#c62828'
                : '#aaa'
              return (
                <tr key={emp.id}>
                  <td>{emp.id}</td>
                  <td><strong>{emp.name}</strong></td>
                  <td>{emp.role}</td>
                  <td>{emp.pay_type === 'Salary' ? 'Lương tháng' : 'Theo giờ'}</td>
                  <td>
                    {emp.pay_type === 'Salary'
                      ? (emp.monthly_salary ? `${Number(emp.monthly_salary).toLocaleString()}₫/tháng` : <span style={{ color: '#aaa' }}>—</span>)
                      : (emp.hourly_rate ? `${Number(emp.hourly_rate).toLocaleString()}₫/giờ` : <span style={{ color: '#aaa' }}>—</span>)
                    }
                  </td>
                  <td style={{ color: hoursColor, fontWeight: 'bold' }}>
                    {emp.pay_type === 'Salary'
                      ? <span style={{ color: '#aaa', fontWeight: 'normal' }}>—</span>
                      : (emp.hours_worked_this_month !== null
                          ? `${emp.hours_worked_this_month.toFixed(1)}h`
                          : <span style={{ color: '#aaa', fontWeight: 'normal' }}>—</span>)
                    }
                  </td>
                  <td style={{ fontWeight: 'bold' }}>
                    {emp.total_salary_this_month
                      ? `${Number(emp.total_salary_this_month).toLocaleString()}₫`
                      : <span style={{ color: '#aaa', fontWeight: 'normal' }}>—</span>
                    }
                  </td>
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
              )
            })}
          </tbody>
        </table>
      )}

      {/* Edit Employee Modal */}
      {editingEmployee && (
        <EditEmployeeModal
          employee={editingEmployee}
          name={editName}
          role={editRole}
          payType={editPayType}
          hourlyRate={editHourlyRate}
          monthlySalary={editMonthlySalary}
          onNameChange={setEditName}
          onRoleChange={setEditRole}
          onPayTypeChange={setEditPayType}
          onHourlyRateChange={setEditHourlyRate}
          onMonthlySalaryChange={setEditMonthlySalary}
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
