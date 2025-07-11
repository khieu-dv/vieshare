
# 🔌 FRPS Port Mapping Manager

**VieShare** is a cross-platform desktop application that helps you easily manage configuration and connections to **FRP (Fast Reverse Proxy)**. Built with [Tauri](https://tauri.app/) + [Next.js](https://nextjs.org/), it provides an intuitive interface to create, monitor, and delete remote port mappings.

---

## 🔗 Download

You can download the latest Windows version (.exe) here:

➡️ [Download VieClone for Windows](https://github.com/khieu-dv/vieshare/releases/download/1.1.0/vieshare_tauri_1.1.0_x64-setup.exe)

---

## 🚀 Key Features

* ⚡ Connect/disconnect to FRP server with a single click
* 🛠 Add and remove port mappings dynamically within the app
* 🌐 Quickly open running apps remotely through mapped ports
* 🧩 Automatically name proxies and choose appropriate ports
* 📡 View real-time connection status and number of active ports
* 🔐 Secured with token-based authentication
* 💻 Minimal, user-friendly UI, built with Tauri + Next.js

---

## 🧰 System Requirements

Before installing, ensure you have:

* [Node.js](https://nodejs.org/) (v18+)
* [Rust + Cargo](https://rustup.rs/)
* npm (included with Node.js)

---

## 📦 Installation & Setup

```bash
git clone https://github.com/khieu-dv/vieshare.git
cd vieshare
npm install
```

---

## 🧪 Run in Development Mode

```bash
npx tauri dev
```

This task will:

* Build the Next.js frontend
* Launch the Tauri window
* Check and install required dependencies

---

## 🏗 Build for Production

```bash
npx tauri build
```

After completion, you’ll find the binary files in:

```
src-tauri/target/release/bundle/
```

---

## 📁 Project Directory Structure

```
frps-manager/
├── src-tauri/           # Tauri backend (Rust)
├── src/                 # Next.js frontend
│   ├── app/             # App Router Pages
│   ├── lib/tauri/       # Tauri bindings
│   └── ui/              # Reusable UI components (Card, Button, etc.)
├── public/              # Static assets
├── package.json
├── tauri.conf.json
└── README.md
```

---

## 💡 Usage Tips

* ✅ Configuration data is saved in `AppData` or its platform equivalent.
* 🔁 When changing ports, just click `Add Mapping` — FRPS will auto-detect and update.

---

## 🐞 Common Issue Troubleshooting

| Issue                          | Solution                                                 |
| ------------------------------ | -------------------------------------------------------- |
| Unable to connect to FRP       | Check server address/domain name and token configuration |
| `Dependencies not ready` error | Wait for system to bootstrap or try restarting the app   |
| Added port not showing         | Click the `Refresh` button in the UI                     |

---

## 📦 Technologies Used

* [FRP (Fast Reverse Proxy)](https://github.com/fatedier/frp)
* [Tauri](https://tauri.app/)
* [Next.js](https://nextjs.org/)
* [Lucide Icons](https://lucide.dev/)
* [Tailwind CSS](https://tailwindcss.com/)

