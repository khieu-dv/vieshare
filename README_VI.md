

# 🔌 FRPS Port Mapping Manager

**VieShare** là một ứng dụng desktop đa nền tảng giúp bạn quản lý cấu hình và kết nối tới **FRP (Fast Reverse Proxy)** một cách dễ dàng. Ứng dụng được xây dựng với [Tauri](https://tauri.app/) + [Next.js](https://nextjs.org/), cung cấp giao diện trực quan để tạo, theo dõi và xóa cổng port từ xa.

---

## 🔗 Tải xuống

Bạn có thể tải xuống phiên bản Windows mới nhất (.exe) tại đây:

➡️ [Tải xuống VieClone cho Windows](https://github.com/khieu-dv/vieshare/releases/download/1.1.0/vieshare_tauri_1.1.0_x64-setup.exe)
---

<p align="center">
  <img src="./public/s1.png" width="400" alt="VieVlog Dashboard" />
  <img src="./public/s2.png" width="400" alt="VieVlog Content Creation" />
</p>

<p align="center">
  <img src="./public/s3.png" width="400" alt="VieVlog Learning Interface" />
  <img src="./public/s4.png" width="400" alt="VieVlog Mobile View" />
</p>

---

## 🚀 Tính năng chính

* ⚡ Kết nối/ngắt kết nối đến FRP server chỉ với 1 cú click
* 🛠 Thêm và xóa port mapping động ngay trong app
* 🌐 Mở nhanh app đang chạy từ xa thông qua port
* 🧩 Tự động đặt tên proxy và chọn port phù hợp
* 📡 Xem trạng thái kết nối, số lượng port đang dùng theo thời gian thực
* 🔐 Bảo mật bằng xác thực token
* 💻 UI tối giản, dễ dùng, hoạt động mượt mà trên Tauri + Next.js

---

## 🧰 Yêu cầu hệ thống

Trước khi cài đặt, đảm bảo bạn đã có:

* [Node.js](https://nodejs.org/) (v18+)
* [Rust + Cargo](https://rustup.rs/)
* npm (đi kèm Node.js)

---

## 📦 Cài đặt & Setup

```bash
git clone https://github.com/khieu-dv/vieshare.git
cd vieshare
npm install
```

---

## 🧪 Chạy ở chế độ Development

```bash
npx tauri dev
```

Tác vụ này sẽ:

* Build frontend Next.js
* Khởi động cửa sổ Tauri
* Kiểm tra và cài đặt các công cụ phụ thuộc cần thiết

---

## 🏗 Build bản Production

```bash
npx tauri build
```

Sau khi hoàn tất, bạn sẽ thấy các file binary tại:

```
src-tauri/target/release/bundle/
```

---

## 📁 Cấu trúc thư mục dự án

```
frps-manager/
├── src-tauri/           # Backend Tauri (Rust)
├── src/                 # Next.js frontend
│   ├── app/             # App Router Pages
│   ├── lib/tauri/       # Tauri bindings
│   └── ui/              # Giao diện dùng lại (Card, Button,...)
├── public/              # Static assets
├── package.json
├── tauri.conf.json
└── README.md
```

---

## 💡 Mẹo sử dụng

* ✅ Dữ liệu cấu hình được lưu trong `AppData` hoặc thư mục tương đương, theo nền tảng.
* 🔁 Khi thay đổi cổng, chỉ cần `Add Mapping`, FRPS sẽ tự nhận diện và cập nhật.

---

## 🐞 Khắc phục sự cố phổ biến

| Vấn đề                       | Cách xử lý                                            |
| ---------------------------- | ----------------------------------------------------- |
| Không kết nối được FRP       | Kiểm tra địa chỉ/tên miền + token cấu hình            |
| Lỗi `Dependencies not ready` | Chờ hệ thống tự động bootstrap hoặc thử khởi động lại |
| Không hiện port đã thêm      | Bấm nút `Refresh` trong giao diện                     |

---

## 📦 Công nghệ sử dụng

* [FRP (Fast Reverse Proxy)](https://github.com/fatedier/frp)
* [Tauri](https://tauri.app/)
* [Next.js](https://nextjs.org/)
* [Lucide Icons](https://lucide.dev/)
* [Tailwind CSS](https://tailwindcss.com/)

