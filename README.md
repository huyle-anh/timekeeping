# Timekeeping & Payroll System

Hệ thống quản lý chấm công và tính lương cho doanh nghiệp.  
Backend **Rust** (Axum + SQLite), Frontend **React + TypeScript** (Vite).

## Tính năng hiện có

- Quản lý nhân viên (CRUD, phân quyền Admin)
- Chấm công vào/ra không cần đăng nhập (tối ưu hiện trường)
- Xem lịch sử chấm công, lọc theo ngày hoặc tháng
- Thống kê tổng giờ làm theo tháng với progress bar
- Quản lý tiền cọc (state machine: Pending → Active → Released/Forfeited)
- Xác thực Admin bằng JWT
- Rate limiting chấm công theo nhân viên
- CORS chỉ cho phép origin được cấu hình

## Yêu cầu

- **Rust** 1.70+
- **Node.js** 18+ và npm

## Cài đặt và chạy

### 1. Backend

```bash
# Tạo file cấu hình từ template
cp .env.example .env
# Chỉnh sửa .env theo môi trường (bắt buộc đổi JWT_SECRET, ADMIN_PASSWORD)

# Build và chạy (development)
cargo run

# Build production
cargo build --release
./target/release/timekeeping
```

### 2. Frontend (development)

```bash
cd frontend
npm install
npm run dev
```

### 3. Frontend (production build)

```bash
cd frontend
npm ci
npm run build
# Output: frontend/dist/
```

### 4. Chạy binary release (từ GitHub Releases)

#### Bước chung (tất cả hệ điều hành)

```bash
# 1. Tải artifact từ GitHub Releases
#    - Vào https://github.com/<user>/<repo>/releases
#    - Chọn release mong muốn
#    - Tải file tương ứng với hệ điều hành của bạn

# 2. Giải nén (nếu file là .tar.gz hoặc .zip)
tar -xzf timekeeping-<os>-<arch>.tar.gz
# hoặc
unzip timekeeping-<os>-<arch>.zip

# 3. Tạo file .env từ template
cp .env.example .env
# Chỉnh sửa .env theo môi trường (bắt buộc đổi JWT_SECRET, ADMIN_PASSWORD)

# 4. Đảm bảo thư mục frontend-dist/ nằm cùng thư mục với binary
#    (hoặc set biến FRONTEND_DIR trỏ đến đúng đường dẫn)
```

#### Linux (x86_64)

```bash
# Cấp quyền thực thi
chmod +x timekeeping-linux-x86_64

# Chạy
./timekeeping-linux-x86_64
```

#### Windows (x86_64)

```powershell
# Mở Command Prompt hoặc PowerShell trong thư mục chứa file
# Chạy trực tiếp (file .exe)
timekeeping-windows-x86_64.exe

# Nếu Windows Defender chặn, chọn "Run anyway" hoặc thêm vào exception
```

#### macOS (Intel - x86_64)

```bash
# Cấp quyền thực thi
chmod +x timekeeping-macos-x86_64

# Nếu bị Gatekeeper chặn (cảnh báo "cannot be opened because the developer cannot be verified")
xattr -d com.apple.quarantine timekeeping-macos-x86_64

# Chạy
./timekeeping-macos-x86_64
```

#### macOS (Apple Silicon - ARM)

```bash
# Cấp quyền thực thi
chmod +x timekeeping-macos-aarch64

# Nếu bị Gatekeeper chặn
xattr -d com.apple.quarantine timekeeping-macos-aarch64

# Chạy
./timekeeping-macos-aarch64
```

> **Lưu ý**: Trên macOS, bạn có thể cần cho phép ứng dụng trong **System Settings → Privacy & Security** nếu lần đầu chạy.

## Biến môi trường

### Backend (`.env` hoặc shell)

| Biến | Mặc định | Mô tả |
|---|---|---|
| `DATABASE_URL` | `timekeeping.db` | Đường dẫn file SQLite |
| `DB_POOL_SIZE` | `10` | Số connection pool |
| `BIND_ADDR` | `0.0.0.0:3000` | Địa chỉ và port server |
| `JWT_SECRET` | *(yếu — phải đổi)* | Khóa ký JWT |
| `ADMIN_USERNAME` | `admin` | Tên đăng nhập quản trị |
| `ADMIN_PASSWORD` | `admin123` | Mật khẩu quản trị *(phải đổi)* |
| `CORS_ALLOWED_ORIGIN` | `http://localhost:5173` | Origin được phép truy cập |
| `RATE_LIMIT_PER_MINUTE` | `30` | Giới hạn chấm công/phút/nhân viên |

### Frontend (`frontend/.env`)

| Biến | Mặc định | Mô tả |
|---|---|---|
| `VITE_API_BASE` | `http://127.0.0.1:3001` | URL backend API |

> **Bảo mật**: Luôn đổi `JWT_SECRET` và `ADMIN_PASSWORD` trước khi deploy production.

## API Endpoints

### Public (không cần xác thực)

| Method | Path | Mô tả |
|---|---|---|
| `GET` | `/health` | Health check |
| `POST` | `/auth/login` | Đăng nhập, nhận JWT token |
| `GET` | `/employees` | Danh sách nhân viên |
| `GET` | `/employees/:id` | Chi tiết nhân viên |
| `POST` | `/attendance/check-in` | Chấm công vào |
| `POST` | `/attendance/check-out` | Chấm công ra |
| `GET` | `/attendance` | Lịch sử chấm công (`?date=YYYY-MM-DD` hoặc `?month=YYYY-MM`) |
| `GET` | `/employees/:id/attendance` | Chấm công theo nhân viên (`?date=YYYY-MM-DD`) |

### Protected (yêu cầu `Authorization: Bearer <token>`)

| Method | Path | Mô tả |
|---|---|---|
| `POST` | `/employees` | Tạo nhân viên mới |
| `PUT` | `/employees/:id` | Cập nhật nhân viên |
| `DELETE` | `/employees/:id` | Xóa nhân viên |

### Ví dụ

```bash
# Đăng nhập
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
# → {"token": "eyJ..."}

# Tạo nhân viên (cần token)
curl -X POST http://localhost:3000/employees \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJ..." \
  -d '{"name":"Nguyễn Văn A","role":"Staff","hourly_rate":"50000"}'

# Chấm công vào (không cần token)
curl -X POST http://localhost:3000/attendance/check-in \
  -H "Content-Type: application/json" \
  -d '{"employee_id":1}'

# Lịch sử theo tháng
curl "http://localhost:3000/attendance?month=2026-04"
```

## Cấu trúc dự án

```
.
├── src/                    # Backend Rust
│   ├── main.rs             # Entry point, router, CORS, middleware
│   ├── lib.rs              # AppState
│   ├── config.rs           # Cấu hình từ env vars
│   ├── auth.rs             # JWT, login handler, admin middleware
│   ├── errors.rs           # Error types
│   ├── deposit.rs          # Deposit state machine
│   ├── handlers/mod.rs     # HTTP handlers
│   └── db/
│       ├── mod.rs          # Structs + repository functions
│       ├── schema.rs       # SQL migration
│       └── deposit_repo.rs # Deposit repository
├── frontend/               # React + TypeScript (Vite)
│   ├── src/
│   │   ├── App.tsx         # Main app
│   │   └── components/
│   │       ├── LoginPopup.tsx
│   │       ├── EditEmployeeModal.tsx
│   │       └── DeleteConfirmModal.tsx
│   └── package.json
├── src-tauri/              # Tauri desktop wrapper (tùy chọn)
├── .env.example            # Template biến môi trường backend
├── .gitignore
└── Cargo.toml
```

## Stack kỹ thuật

| Layer | Công nghệ |
|---|---|
| Runtime | Tokio (async) |
| Web framework | Axum 0.6 |
| Database | SQLite (rusqlite + r2d2 pool) |
| Auth | jsonwebtoken 9 (JWT HS256) |
| Số thập phân | rust_decimal (không dùng float cho tiền) |
| Logging | tracing (structured) |
| Frontend | React 18 + TypeScript + Vite |
| Desktop | Tauri 1.x (tùy chọn) |

## Test

```bash
cargo test
cargo test -- --nocapture
```

## Troubleshooting

### Port đã được sử dụng

```bash
lsof -i :3000
kill -9 <PID>
# Hoặc dùng port khác
BIND_ADDR=0.0.0.0:3001 cargo run
```

### Frontend không kết nối được backend

Kiểm tra `VITE_API_BASE` trong `frontend/.env` khớp với địa chỉ backend.

### Lỗi build Tauri (Linux)

```bash
sudo apt install libwebkit2gtk-4.0-dev build-essential libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

## License

MIT
