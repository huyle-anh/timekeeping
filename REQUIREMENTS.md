# 📋 PROJECT: TIME-KEEPING & PAYROLL SYSTEM

## 🚀 MỤC TIÊU HỆ THỐNG
Xây dựng ứng dụng quản lý chấm công, tính lương và tiền cọc với tiêu chuẩn **Reliability** cao nhất, đảm bảo không mất mát dữ liệu tài chính.

## 🛠️ STACK KỸ THUẬT (Xác định bởi Architect)
- **Backend**: Rust (Axum/Tokio) - Đảm bảo Safety & Performance.
- **Database**: SQLite (hoặc PostgreSQL) - Hỗ trợ ACID cho giao dịch tiền tệ.
- **Frontend**: [Điền Framework bạn chọn, ví dụ: React/Next.js].
- **Logging**: `tracing` crate với Correlation ID.

## 🎯 CÁC MODULE CHÍNH & LOGIC NGHIỆP VỤ

### 1. Module Chấm Công (Check-in/Out)
- **Logic**: Không đăng nhập để tối ưu tốc độ tại hiện trường.
- **Safety**: Chống spam request, ghi log IP/Device ID kèm `correlation_id`.

### 2. Module Nhân Viên & Phân Quyền
- **Roles**: `Admin`, `Manager`, `Staff`.
- **Constraint**: Chỉ `Admin` mới có quyền Export dữ liệu nhạy cảm.

### 3. Module Tiền Cọc (Crucial)
- **State Machine**: Bắt buộc triển khai theo `CONVENTIONS.md`.
- **States**: `Pending` (Chờ cọc) -> `Active` (Đang giữ cọc) -> `Released` (Đã hoàn trả) -> `Forfeited` (Bị khấu trừ).
- **Rule**: Tự động trích % lương hàng tháng cho đến khi đủ định mức.

### 4. Module Lương & Báo Cáo
- **Accuracy**: Dùng kiểu dữ liệu `Decimal`, tuyệt đối không dùng `Float` cho tiền tệ.
- **Export**: Hỗ trợ xuất Excel (xlsx).

### 5. Hệ Thống Backup & Recovery (Reliability)
- **Auto-backup**: Chạy cronjob mỗi ngày, xoay vòng 30 bản.
- **Recovery**: Script khôi phục một lệnh, phải được verify thủ công (Tier 4).

## 🚦 QUY TRÌNH THỰC HIỆN (Dành cho Aider)
1. **Bước 1**: Đọc `CONVENTIONS.md` và `REQUIREMENTS.md`.
2. **Bước 2 (Architect)**: Thiết kế Database Schema và State Machine cho Tiền cọc.
3. **Bước 3 (Senior Engineer)**: Triển khai Backend Core với Error Handling (không `unwrap`).
4. **Bước 4 (QA)**: Viết Integration Test cho luồng Chấm công -> Tính lương -> Trích cọc.