import { useState } from 'react'

interface EditEmployeeModalProps {
  employee: { id: number; name: string; role: string; hourly_rate: string | null }
  name: string
  role: string
  payType: string
  hourlyRate: string
  monthlySalary: string
  onNameChange: (val: string) => void
  onRoleChange: (val: string) => void
  onPayTypeChange: (val: string) => void
  onHourlyRateChange: (val: string) => void
  onMonthlySalaryChange: (val: string) => void
  onSave: () => void | Promise<void>
  onClose: () => void
}

export default function EditEmployeeModal({
  employee,
  name,
  role,
  payType,
  hourlyRate,
  monthlySalary,
  onNameChange,
  onRoleChange,
  onPayTypeChange,
  onHourlyRateChange,
  onMonthlySalaryChange,
  onSave,
  onClose,
}: EditEmployeeModalProps) {
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  const handleSave = async () => {
    setSaving(true)
    setError('')
    try {
      await onSave()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể lưu')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 2000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: '#fff',
          borderRadius: '8px',
          padding: '24px',
          minWidth: '400px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.2)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ margin: '0 0 16px', fontSize: '18px' }}>
          Chỉnh sửa nhân viên #{employee.id}
        </h2>

        {error && (
          <div style={{ color: '#d32f2f', marginBottom: '12px', fontSize: '13px' }}>
            {error}
          </div>
        )}

        <div style={{ marginBottom: '12px' }}>
          <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}>
            Họ và tên
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
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
          <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}>
            Chức vụ
          </label>
          <select
            value={role}
            onChange={(e) => onRoleChange(e.target.value)}
            style={{
              width: '100%',
              padding: '8px',
              border: '1px solid #ccc',
              borderRadius: '4px',
              fontSize: '14px',
              boxSizing: 'border-box',
            }}
          >
            <option value="Staff">Nhân viên</option>
            <option value="Manager">Quản lý</option>
            <option value="Admin">Quản trị viên</option>
          </select>
        </div>

        <div style={{ marginBottom: '12px' }}>
          <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}>
            Loại lương
          </label>
          <select
            value={payType}
            onChange={(e) => onPayTypeChange(e.target.value)}
            style={{
              width: '100%',
              padding: '8px',
              border: '1px solid #ccc',
              borderRadius: '4px',
              fontSize: '14px',
              boxSizing: 'border-box',
            }}
          >
            <option value="Hourly">Theo giờ</option>
            <option value="Salary">Lương tháng</option>
          </select>
        </div>

        {payType === 'Hourly' ? (
          <div style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}>
              Lương/giờ
            </label>
            <input
              type="number"
              step="0.01"
              value={hourlyRate}
              onChange={(e) => onHourlyRateChange(e.target.value)}
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
        ) : (
          <div style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#666' }}>
              Lương tháng
            </label>
            <input
              type="number"
              step="0.01"
              value={monthlySalary}
              onChange={(e) => onMonthlySalaryChange(e.target.value)}
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
        )}

        <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              background: '#f5f5f5',
              border: '1px solid #ccc',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px',
            }}
          >
            Hủy
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            style={{
              padding: '8px 16px',
              background: saving ? '#90caf9' : '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: saving ? 'not-allowed' : 'pointer',
              fontSize: '14px',
            }}
          >
            {saving ? 'Đang lưu...' : 'Lưu'}
          </button>
        </div>
      </div>
    </div>
  )
}
