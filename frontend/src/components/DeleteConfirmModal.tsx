import { useState } from 'react'

interface DeleteConfirmModalProps {
  employee: { id: number; name: string }
  onConfirm: () => void
  onCancel: () => void
}

export default function DeleteConfirmModal({
  employee,
  onConfirm,
  onCancel,
}: DeleteConfirmModalProps) {
  const [deleting, setDeleting] = useState(false)
  const [error, setError] = useState('')

  const handleDelete = async () => {
    setDeleting(true)
    setError('')
    try {
      await onConfirm()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Không thể xóa')
    } finally {
      setDeleting(false)
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
      onClick={onCancel}
    >
      <div
        style={{
          background: '#fff',
          borderRadius: '8px',
          padding: '24px',
          minWidth: '350px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.2)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ margin: '0 0 12px', fontSize: '18px', color: '#d32f2f' }}>
          Xác nhận xóa
        </h2>

        <p style={{ margin: '0 0 16px', fontSize: '14px', color: '#333' }}>
          Bạn có chắc chắn muốn xóa <strong>{employee.name}</strong> (ID: {employee.id})?
          Hành động này không thể hoàn tác.
        </p>

        {error && (
          <div style={{ color: '#d32f2f', marginBottom: '12px', fontSize: '13px' }}>
            {error}
          </div>
        )}

        <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
          <button
            onClick={onCancel}
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
            onClick={handleDelete}
            disabled={deleting}
            style={{
              padding: '8px 16px',
              background: deleting ? '#ef9a9a' : '#d32f2f',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: deleting ? 'not-allowed' : 'pointer',
              fontSize: '14px',
            }}
          >
            {deleting ? 'Đang xóa...' : 'Xóa'}
          </button>
        </div>
      </div>
    </div>
  )
}
