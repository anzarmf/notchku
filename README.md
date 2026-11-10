# NotchKu — Interactive Notch for Windows (v0.2)

NotchKu adalah aplikasi notch interaktif buatanku sendiri, dibangun dari
nol: sebuah pil kecil di tengah atas layar yang terbuka jadi panel musik,
kalender, todo, Pomodoro, dan cuaca.

**v0.2.0 sepenuhnya native (WPF / .NET Framework 4.8) — tanpa WebView,
tanpa browser, tanpa runtime tambahan.** .NET Framework 4.8 sudah bawaan
Windows 10/11, jadi user tidak perlu instal apapun. Build aktif ada di
[`notchku-wpf/`](notchku-wpf/). Kode Tauri/WebView lama dan percobaan
Slint/Rust masih ada di repo ini sebagai arsip, tapi tidak dipakai lagi:
Slint menghasilkan EXE ~12 MB (di atas batas 6 MB), sedangkan EXE WPF
di bawah 1 MB.

## ⬇️ Download (Windows)

> Satu installer kecil (≤6 MB), langsung jalan — tidak perlu instal apapun lagi.

**[Download NotchKu_0.2.0_x64-setup.exe](https://github.com/anzarmf/notchku/releases/latest/download/NotchKu_0.2.0_x64-setup.exe)**

Jalankan installer-nya, lalu tekan `Alt+N` untuk membuka notch.
SmartScreen mungkin muncul karena installer belum ditandatangani
(code signing) — klik "More info" → "Run anyway". Semua rilis ada di
[Releases](https://github.com/anzarmf/notchku/releases).

## Apa yang sudah jalan di MVP ini

Notch hitam di tengah atas layar, tertutup sebagai pil kecil, terbuka
saat hover / klik / `Alt+N`:

- **Musik** — kontrol putar/jeda/prev/next, progress bar.
  Di v0.2.0 masih pemutar demo lokal (3 lagu contoh), supaya UI bisa
  dicoba tanpa integrasi media. Integrasi SMTC Windows menyusul.
- **Kalender** — grid bulan ini, hari ini ditandai, navigasi bulan.
- **Todo** — tambah, centang, hapus. Tersimpan lokal di
  `%APPDATA%\NotchKu\todos.json`.
- **Pomodoro** — Fokus 25 menit / Istirahat 5 menit, mulai, jeda, reset.
- **Cuaca** — Open-Meteo, tanpa API key (Jakarta, bisa diubah di kode).
  Menampilkan suhu, kondisi, kelembapan, dan kecepatan angin.
- **Hotkey global** — `Alt+N` membuka/menutup notch dari mana saja.

Yang **belum** ada di MVP ini: Dock, AI Chat,
lirik lagu, jadwal sholat, Wi-Fi/Bluetooth, Shelf/drop file, Buddy 3D,
notifikasi Windows, update otomatis.

## Arsitektur (v0.2.0)

```
+-------------------------------+
|  WPF (.NET Framework 4.8)    |
|  MainWindow.xaml — pil + panel|
|  MainWindow.xaml.cs — logika  |
|  - jam, cuaca (Open-Meteo)    |
|  - todo JSON, kalender,       |
|    pomodoro, musik demo       |
|  - hotkey Alt+N (RegisterHotKey)|
+-------------------------------+
   Window: top-center, always-on-top, frameless,
   transparan, 220x32 saat tertutup, 560x380 saat terbuka
```

Detail lengkap: `ARCHITECTURE.md` (masih mendeskripsikan versi Tauri lama;
arsitektur WPF di atas adalah yang aktif).

## Cara build (di PC Windows, atau via GitHub Actions)

Build otomatis berjalan di GitHub Actions setiap push ke `notchku-wpf/`
dan menerbitkan Release v0.2.0. Build manual di PC Windows 10/11 64-bit:

```powershell
# 1. Install .NET SDK 8 (https://dotnet.microsoft.com/download)
# 2. Copy folder ini, lalu:
cd notchku
dotnet build notchku-wpf/NotchKu.csproj -c Release
# hasil: notchku-wpf\bin\Release\net48\NotchKu.exe
# installer: makensis notchku-wpf/installer.nsi
```

Tidak perlu Rust, Node.js, atau WebView2.

## Data & privasi

Semua tersimpan lokal:

- Windows: `%APPDATA%\NotchKu\` (`settings.json`, `todos.json`)
- Linux: `~/.config/NotchKu/`
- Jaringan hanya: cuaca (Open-Meteo) + cek update (GitHub Releases).
  Tidak ada akun, tidak ada tracking, tidak ada telemetri.

## File penting

| File | Isi |
|---|---|
| `notchku-wpf/NotchKu.csproj` | Proyek WPF (.NET Framework 4.8) |
| `notchku-wpf/MainWindow.xaml` | UI notch (pil + panel 5 tab) |
| `notchku-wpf/MainWindow.xaml.cs` | Logika: jam, cuaca, todo, kalender, pomodoro, musik demo, hotkey Alt+N |
| `notchku-wpf/installer.nsi` | Installer NSIS (≤6 MB) |
| `.github/workflows/release.yml` | Build + rilis otomatis di GitHub Actions |
| `ARCHITECTURE.md` | Desain lengkap + peta jalan ke v1.0 (versi Tauri lama) |

## Batasan yang jujur

- Ini MVP v0.2. Di sini ada 5 modul inti; modul lain menyusul di versi berikutnya.
- Musik SMTC Windows, notifikasi Windows, dan Dock butuh integrasi
  Win32 tambahan — belum diimplementasikan di v0.2.0.
- Belum ada code signing, jadi SmartScreen akan muncul seperti
  aplikasi tanpa sertifikat berbayar. Jangan bypass, klik More info → Run anyway hanya
  untuk build sendiri.
- Belum diuji di hardware nyata (dibangun di server Linux).
  Uji pertama harus di PC Windows: hover, Alt+N, multi-monitor,
  fullscreen, dan idle RAM.
