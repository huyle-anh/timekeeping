import { useState, useEffect, useMemo, useRef } from 'react'
import { getApiBase, parseApiError, parseJsonResponse } from '../api'

interface Employee {
  id: number
  name: string
  role: string
  device_id: string | null
  hourly_rate: string
  created_at: string
  updated_at: string
}

interface AttendanceRecord {
  id: number
  employee_id: number
  event_type: string
  timestamp: string
  correlation_id: string
}

const API_BASE = getApiBase()

function parseTimestamp(ts: string): Date {
  if (!ts.includes('T')) {
    return new Date(ts.replace(' ', 'T') + 'Z')
  }
  return new Date(ts)
}

function AttendancePage() {
  const [employees, setEmployees] = useState<Employee[]>([])
  const [attendance, setAttendance] = useState<AttendanceRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Attendance form
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<number | ''>('')
  const [employeeSearchTerm, setEmployeeSearchTerm] = useState('')
  const [showDropdown, setShowDropdown] = useState(false)

  // Selected employee for viewing attendance
  const [selectedEmployeeForAttendance, setSelectedEmployeeForAttendance] = useState<number | null>(null)
  const [employeeAttendance, setEmployeeAttendance] = useState<AttendanceRecord[]>([])
  const [employeeAttendanceLoading, setEmployeeAttendanceLoading] = useState(false)
  const [employeeAttendanceDate, setEmployeeAttendanceDate] = useState<string>('')
  const [attendanceDate, setAttendanceDate] = useState<string>('')

  // Monthly attendance data
  const todayYM = new Date().toISOString().slice(0, 7)
  const [hoursMonth, setHoursMonth] = useState<string>(todayYM)
  const [monthAttendance, setMonthAttendance] = useState<AttendanceRecord[]>([])

  const dropdownRef = useRef<HTMLDivElement>(null)

  const filteredEmployees = useMemo(() => {
    if (!employeeSearchTerm.trim()) return employees
    const term = employeeSearchTerm.toLowerCase()
    return employees.filter(
      emp =>
        emp.name.toLowerCase().includes(term) ||
        emp.id.toString().includes(term)
    )
  }, [employees, employeeSearchTerm])

  const lastAttendanceByEmployee = useMemo(() => {
    const map: Record<number, { checkIn?: string; checkOut?: string }> = {}
    for (const rec of attendance) {
      if (!map[rec.employee_id]) map[rec.employee_id] = {}
      if (rec.event_type === 'check_in') map[rec.employee_id].checkIn = rec.timestamp
      if (rec.event_type === 'check_out') map[rec.employee_id].checkOut = rec.timestamp
    }
    return map
  }, [attendance])

  const monthlyMinutesByEmployee = useMemo(() => {
    const map: Record<number, number> = {}
    const byEmp: Record<number, AttendanceRecord[]> = {}
    for (const rec of monthAttendance) {
      if (!byEmp[rec.employee_id]) byEmp[rec.employee_id] = []
      byEmp[rec.employee_id].push(rec)
    }
    for (const [empId, recs] of Object.entries(byEmp)) {
      let total = 0
      let pendingCheckIn: Date | null = null
      for (const rec of recs) {
        if (rec.event_type === 'check_in') {
          pendingCheckIn = parseTimestamp(rec.timestamp)
        } else if (rec.event_type === 'check_out' && pendingCheckIn) {
          const diff = (parseTimestamp(rec.timestamp).getTime() - pendingCheckIn.getTime()) / 60000
          if (diff > 0) total += diff
          pendingCheckIn = null
        }
      }
      map[Number(empId)] = total
    }
    return map
  }, [monthAttendance])

  useEffect(() => {
    fetchEmployees()
    fetchAttendance(attendanceDate)
    fetchMonthlyAttendance(hoursMonth)
  }, [])

  useEffect(() => {
    fetchAttendance(attendanceDate)
  }, [attendanceDate])

  useEffect(() => {
    fetchMonthlyAttendance(hoursMonth)
  }, [hoursMonth])

  useEffect(() => {
    if (selectedEmployeeForAttendance !== null) {
      fetchEmployeeAttendance(selectedEmployeeForAttendance, employeeAttendanceDate)
    }
  }, [selectedEmployeeForAttendance, employeeAttendanceDate])

  const fetchEmployees = async () => {
    try {
      setLoading(true)
      const res = await fetch(`${API_BASE}/employees`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await parseJsonResponse<Employee[]>(res)
      setEmployees(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể tải danh sách nhân viên')
    } finally {
      setLoading(false)
    }
  }

  const fetchAttendance = async (date?: string) => {
    try {
      const url = date
        ? `${API_BASE}/attendance?date=${encodeURIComponent(date)}`
        : `${API_BASE}/attendance`
      const res = await fetch(url)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await parseJsonResponse<AttendanceRecord[]>(res)
      setAttendance(data)
    } catch (_e) {
      // Don't block UI on attendance fetch failure
    }
  }

  const fetchMonthlyAttendance = async (month: string) => {
    try {
      const res = await fetch(`${API_BASE}/attendance?month=${encodeURIComponent(month)}`)
      if (!res.ok) return
      const data = await parseJsonResponse<AttendanceRecord[]>(res)
      setMonthAttendance(data)
    } catch (_e) {
      // silent
    }
  }

  const fetchEmployeeAttendance = async (employeeId: number, date?: string) => {
    setEmployeeAttendanceLoading(true)
    try {
      const url = date
        ? `${API_BASE}/employees/${employeeId}/attendance?date=${encodeURIComponent(date)}`
        : `${API_BASE}/employees/${employeeId}/attendance`
      const res = await fetch(url)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await parseJsonResponse<AttendanceRecord[]>(res)
      setEmployeeAttendance(data)
    } catch (_e) {
      setEmployeeAttendance([])
    } finally {
      setEmployeeAttendanceLoading(false)
    }
  }

  const handleCheckIn = async () => {
    if (selectedEmployeeId === '') return
    try {
      const body: Record<string, unknown> = { employee_id: selectedEmployeeId }
      const res = await fetch(`${API_BASE}/attendance/check-in`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (!res.ok) {
        throw new Error(await parseApiError(res))
      }
      await fetchAttendance(attendanceDate)
      await fetchMonthlyAttendance(hoursMonth)
      if (selectedEmployeeForAttendance === selectedEmployeeId) {
        await fetchEmployeeAttendance(selectedEmployeeId, employeeAttendanceDate)
      }
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể chấm công vào')
    }
  }

  const handleCheckOut = async () => {
    if (selectedEmployeeId === '') return
    try {
      const body: Record<string, unknown> = { employee_id: selectedEmployeeId }
      const res = await fetch(`${API_BASE}/attendance/check-out`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (!res.ok) {
        throw new Error(await parseApiError(res))
      }
      await fetchAttendance(attendanceDate)
      await fetchMonthlyAttendance(hoursMonth)
      if (selectedEmployeeForAttendance === selectedEmployeeId) {
        await fetchEmployeeAttendance(selectedEmployeeId, employeeAttendanceDate)
      }
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể chấm công ra')
    }
  }

  const getEmployeeName = (id: number): string => {
    const emp = employees.find(e => e.id === id)
    return emp ? emp.name : `Nhân viên #${id}`
  }

  const handleEmployeeClick = (employeeId: number) => {
    if (selectedEmployeeForAttendance === employeeId) {
      setSelectedEmployeeForAttendance(null)
      setEmployeeAttendance([])
    } else {
      setSelectedEmployeeForAttendance(employeeId)
    }
  }

  const handleSelectEmployee = (emp: Employee) => {
    setSelectedEmployeeId(emp.id)
    setEmployeeSearchTerm(`${emp.id} - ${emp.name}`)
    setShowDropdown(false)
  }

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setEmployeeSearchTerm(value)
    setShowDropdown(true)
    const parsedId = parseInt(value, 10)
    if (!isNaN(parsedId) && employees.find(emp => emp.id === parsedId)) {
      setSelectedEmployeeId(parsedId)
    } else {
      const currentLabel = selectedEmployeeId !== ''
        ? `${selectedEmployeeId} - ${employees.find(e => e.id === selectedEmployeeId)?.name ?? ''}`
        : ''
      if (value !== currentLabel) {
        // Don't clear selectedEmployeeId here
      }
    }
  }

  const handleSearchBlur = () => {
    setTimeout(() => setShowDropdown(false), 500)
  }

  const handleSearchFocus = () => {
    setShowDropdown(true)
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

      {/* Check-in / Check-out Section */}
      <div style={{ marginBottom: '2rem', padding: '1rem', border: '1px solid #ccc', borderRadius: '8px' }}>
        <h2>Chấm công</h2>
        <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', alignItems: 'flex-end' }}>
          <div style={{ position: 'relative' }}>
            <label style={{ display: 'block', marginBottom: '0.25rem' }}>ID hoặc Tên nhân viên</label>
            <input
              placeholder="Nhập ID hoặc tên nhân viên..."
              value={employeeSearchTerm}
              onChange={handleSearchChange}
              onFocus={handleSearchFocus}
              onBlur={handleSearchBlur}
              autoComplete="off"
              style={{ padding: '0.5rem', minWidth: '250px' }}
            />
            {showDropdown && filteredEmployees.length > 0 && (
              <div
                ref={dropdownRef}
                onMouseDown={(e) => e.preventDefault()}
                style={{
                  position: 'absolute',
                  top: '100%',
                  left: 0,
                  right: 0,
                  backgroundColor: 'white',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  maxHeight: '200px',
                  overflowY: 'auto',
                  zIndex: 1000,
                }}
              >
                {filteredEmployees.map(emp => (
                  <div
                    key={emp.id}
                    onMouseDown={(e) => {
                      e.preventDefault()
                      handleSelectEmployee(emp)
                    }}
                    style={{
                      padding: '0.5rem',
                      cursor: 'pointer',
                      borderBottom: '1px solid #eee',
                      backgroundColor: selectedEmployeeId === emp.id ? '#e3f2fd' : 'white',
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#f5f5f5')}
                    onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = selectedEmployeeId === emp.id ? '#e3f2fd' : 'white')}
                  >
                    <strong>{emp.name}</strong> (ID: {emp.id}) - {emp.role}
                  </div>
                ))}
              </div>
            )}
            {selectedEmployeeId !== '' && (
              <div style={{ marginTop: '0.25rem', fontSize: '0.85rem', color: '#666' }}>
                Đã chọn: {getEmployeeName(selectedEmployeeId as number)} (ID: {selectedEmployeeId})
              </div>
            )}
          </div>
          <button
            onClick={handleCheckIn}
            disabled={selectedEmployeeId === ''}
            style={{
              padding: '0.5rem 1.5rem',
              backgroundColor: '#4CAF50',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: selectedEmployeeId === '' ? 'not-allowed' : 'pointer',
              opacity: selectedEmployeeId === '' ? 0.6 : 1,
            }}
          >
            Vào ca
          </button>
          <button
            onClick={handleCheckOut}
            disabled={selectedEmployeeId === ''}
            style={{
              padding: '0.5rem 1.5rem',
              backgroundColor: '#f44336',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: selectedEmployeeId === '' ? 'not-allowed' : 'pointer',
              opacity: selectedEmployeeId === '' ? 0.6 : 1,
            }}
          >
            Ra ca
          </button>
        </div>
      </div>

      {/* Employees Table with attendance info */}
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
        <table border={1} cellPadding={8} style={{ borderCollapse: 'collapse', width: '100%', marginBottom: '2rem' }}>
          <thead>
            <tr>
              <th>ID</th>
              <th>Họ và tên</th>
              <th>Lương/giờ</th>
              <th style={{ color: '#4CAF50' }}>Vào ca cuối</th>
              <th style={{ color: '#f44336' }}>Ra ca cuối</th>
              <th style={{ minWidth: '140px' }}>
                Giờ làm ({hoursMonth.slice(5)}/{hoursMonth.slice(0, 4)})
              </th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            {employees.map((emp) => {
              const att = lastAttendanceByEmployee[emp.id] ?? {}
              const mins = monthlyMinutesByEmployee[emp.id] ?? 0
              const hours = Math.floor(mins / 60)
              const remainMins = Math.round(mins % 60)
              const STANDARD_HOURS = 176
              const pct = Math.min(100, Math.round((hours + remainMins / 60) / STANDARD_HOURS * 100))
              const hoursColor = hours >= 160 ? '#2e7d32' : hours >= 80 ? '#e65100' : mins > 0 ? '#c62828' : '#aaa'
              const barColor = hours >= 160 ? '#4CAF50' : hours >= 80 ? '#ff9800' : '#f44336'
              return (
                <tr key={emp.id}>
                  <td>{emp.id}</td>
                  <td><strong>{emp.name}</strong></td>
                  <td>{emp.hourly_rate}</td>
                  <td style={{ color: '#4CAF50', whiteSpace: 'nowrap', fontSize: '13px' }}>
                    {att.checkIn ?? <span style={{ color: '#aaa' }}>—</span>}
                  </td>
                  <td style={{ color: '#f44336', whiteSpace: 'nowrap', fontSize: '13px' }}>
                    {att.checkOut ?? <span style={{ color: '#aaa' }}>—</span>}
                  </td>
                  <td style={{ minWidth: '140px' }}>
                    <div style={{ fontWeight: 'bold', color: hoursColor, marginBottom: '4px' }}>
                      {mins > 0 ? `${hours}h ${remainMins}m` : <span style={{ color: '#aaa', fontWeight: 'normal' }}>—</span>}
                    </div>
                    {mins > 0 && (
                      <div style={{ background: '#eee', borderRadius: '4px', height: '6px', width: '100%' }}>
                        <div style={{
                          background: barColor,
                          borderRadius: '4px',
                          height: '6px',
                          width: `${pct}%`,
                          transition: 'width 0.3s',
                        }} />
                      </div>
                    )}
                    {mins > 0 && (
                      <div style={{ fontSize: '11px', color: '#999', marginTop: '2px' }}>{pct}% / {STANDARD_HOURS}h</div>
                    )}
                  </td>
                  <td>
                    <button
                      onClick={() => handleEmployeeClick(emp.id)}
                      style={{
                        padding: '0.25rem 0.75rem',
                        backgroundColor: selectedEmployeeForAttendance === emp.id ? '#ff9800' : '#2196F3',
                        color: 'white',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '12px',
                      }}
                    >
                      {selectedEmployeeForAttendance === emp.id ? 'Ẩn' : 'Xem'}
                    </button>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}

      {/* Employee Attendance Detail */}
      {selectedEmployeeForAttendance !== null && (
        <div style={{ marginBottom: '2rem', padding: '1rem', border: '1px solid #e3f2fd', borderRadius: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', marginBottom: '0.75rem' }}>
            <h3 style={{ margin: 0 }}>Chấm công — {getEmployeeName(selectedEmployeeForAttendance)}</h3>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <label style={{ fontSize: '13px', color: '#666' }}>Ngày:</label>
              <input
                type="date"
                value={employeeAttendanceDate}
                onChange={(e) => setEmployeeAttendanceDate(e.target.value)}
                style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid #ccc', fontSize: '13px' }}
              />
              {employeeAttendanceDate && (
                <button
                  onClick={() => setEmployeeAttendanceDate('')}
                  style={{ padding: '4px 8px', fontSize: '12px', cursor: 'pointer', borderRadius: '4px', border: '1px solid #ccc' }}
                >
                  Xóa bộ lọc
                </button>
              )}
            </div>
          </div>
          {employeeAttendanceLoading ? (
            <p>Đang tải...</p>
          ) : employeeAttendance.length === 0 ? (
            <p style={{ color: '#888' }}>Không có dữ liệu chấm công{employeeAttendanceDate ? ` ngày ${employeeAttendanceDate}` : ''} cho nhân viên này.</p>
          ) : (
            <table border={1} cellPadding={8} style={{ borderCollapse: 'collapse', width: '100%' }}>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Sự kiện</th>
                  <th>Thời gian</th>
                </tr>
              </thead>
              <tbody>
                {employeeAttendance.map((rec) => (
                  <tr key={rec.id}>
                    <td>{rec.id}</td>
                    <td style={{ color: rec.event_type === 'check_in' ? '#4CAF50' : '#f44336' }}>
                      {rec.event_type === 'check_in' ? '✅ Vào ca' : '❌ Ra ca'}
                    </td>
                    <td>{rec.timestamp}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {/* Attendance History */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', marginBottom: '0.5rem' }}>
        <h2 style={{ margin: 0 }}>Lịch sử chấm công (Tất cả)</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <label style={{ fontSize: '13px', color: '#666' }}>Ngày:</label>
          <input
            type="date"
            value={attendanceDate}
            onChange={(e) => setAttendanceDate(e.target.value)}
            style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid #ccc', fontSize: '13px' }}
          />
          {attendanceDate && (
            <button
              onClick={() => setAttendanceDate('')}
              style={{ padding: '4px 8px', fontSize: '12px', cursor: 'pointer', borderRadius: '4px', border: '1px solid #ccc' }}
            >
              Xóa bộ lọc
            </button>
          )}
        </div>
      </div>
      {attendance.length === 0 ? (
        <p style={{ color: '#888' }}>Chưa có dữ liệu chấm công{attendanceDate ? ` ngày ${attendanceDate}` : ''}.</p>
      ) : (
        <table border={1} cellPadding={8} style={{ borderCollapse: 'collapse', width: '100%' }}>
          <thead>
            <tr>
              <th>ID</th>
              <th>Nhân viên</th>
              <th>Sự kiện</th>
              <th>Thời gian</th>
            </tr>
          </thead>
          <tbody>
            {attendance.map((rec) => (
              <tr key={rec.id}>
                <td>{rec.id}</td>
                <td>{getEmployeeName(rec.employee_id)}</td>
                <td style={{ color: rec.event_type === 'check_in' ? '#4CAF50' : '#f44336' }}>
                  {rec.event_type === 'check_in' ? '✅ Vào ca' : '❌ Ra ca'}
                </td>
                <td>{rec.timestamp}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}

export default AttendancePage
