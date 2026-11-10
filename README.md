# NotchKu — Interactive Notch for Windows (v0.1)

NotchKu adalah aplikasi notch interaktif buatanku sendiri, dibangun dari
nol: sebuah pil kecil di tengah atas layar yang terbuka jadi panel musik,
kalender, todo, Pomodoro, dan cuaca.

## ⬇️ Download (Windows)

> Satu installer kecil (≤6 MB), langsung jalan — WebView2 ter-download otomatis saat instal, tidak perlu instal apapun lagi secara manual (butuh internet saat instal).

**[Download NotchKu_0.1.2_x64-setup.exe](https://github.com/anzarmf/notchku/releases/latest/download/NotchKu_0.1.2_x64-setup.exe)**

Jalankan installer-nya, lalu tekan `Alt+N` untuk membuka notch.
SmartScreen mungkin muncul karena installer belum ditandatangani
(code signing) — klik "More info" → "Run anyway". Semua rilis ada di
[Releases](https://github.com/anzarmf/notchku/releases).

## Apa yang sudah jalan di MVP ini

Notch hitam di tengah atas layar, tertutup sebagai pil kecil, terbuka
saat hover / klik / `Alt+N`:

- **Musik** — kontrol putar/jeda/prev/next, progress bar, volume.
  Di Windows nyata: baca media yang sedang diputar lewat Windows
  System Media Transport Controls (SMTC) via `windows-rs`. Di MVP
  ini masih pemutar demo lokal (file `src/main.js`, fungsi
  `initMusicDemo`), supaya UI bisa dicoba tanpa SMTC.
- **Kalender** — bulan ini + rapat berikutnya (data contoh di
  `src/main.js`, ganti dengan Google Calendar / iCal di `lib.rs`).
- **Todo** — tambah, centang, hapus. Tersimpan lokal.
- **Pomodoro** — 25/5, timer + stopwatch, hitung mundur tetap
  terlihat di notch tertutup.
- **Cuaca** — Open-Meteo, tanpa API key
  (`https://api.open-meteo.com/v1/forecast?latitude=-6.2&longitude=106.8&current=temperature_2m,weather_code`).
  Lokasi default Jakarta, bisa diubah di Settings.
- **Settings** — ukuran/posisi notch, transparansi, modul yang tampil.

Yang **belum** ada di MVP ini: Dock, AI Chat,
lirik lagu, jadwal sholat, Wi-Fi/Bluetooth, Shelf/drop file, Buddy 3D,
notifikasi Windows, update otomatis.

## Arsitektur

```
+------------------+  Tauri IPC (invoke)  +-------------------+
|  Frontend (Web)  | --------------------> |  Rust (src-tauri) |
|  HTML/CSS/JS     | <-------------------- |  main.rs / lib.rs |
|  src/index.html  |   events / state     |  - window manager |
|  src/main.js     |                      |  - storage JSON   |
|  src/styles.css  |                      |  - global shortcut|
+------------------+                      |  - update checker |
        |                                 +-------------------+
        | WebView2 (Win) / WebKitGTK (Linux)
        v
   OS window: top-center, always-on-top, frameless,
   transparent, click-through di luar pil saat tertutup
```

Detail lengkap: `ARCHITECTURE.md`.

## Cara build (di PC Windows, bukan di server Linux ini)

Server ini Linux tanpa Rust dan tanpa WebView2, jadi kode di sini
**tidak dikompilasi di server**. Build di PC Windows 10/11 64-bit:

```powershell
# 1. Install Rust (rustup) + Node.js 20 LTS
#    https://rustup.rs  |  https://nodejs.org
# 2. Install WebView2 Runtime (biasanya sudah ada di Windows 11)
#    https://developer.microsoft.com/en-us/microsoft-edge/webview2/
# 3. Copy folder ini, lalu:
cd notchku
npm install
npm run tauri dev      # coba jalan
npm run tauri build    # hasil: src-tauri/target/release/bundle/nsis/
#    -> NotchKu_0.1.2_x64-setup.exe  (NSIS, update sendiri via tauri-plugin-updater)
```

Butuh juga: Visual Studio Build Tools (C++ workload) untuk
kompilasi Rust di Windows.

Di Linux (opsional, untuk uji UI saja):

```bash
npm install
npm run dev   # buka http://localhost:1420 — UI notch saja, tanpa window Rust
```

## Data & privasi

Semua tersimpan lokal:

- Windows: `%APPDATA%\NotchKu\` (`settings.json`, `todos.json`)
- Linux: `~/.config/NotchKu/`
- Jaringan hanya: cuaca (Open-Meteo) + cek update (GitHub Releases).
  Tidak ada akun, tidak ada tracking, tidak ada telemetri.

## File penting

| File | Isi |
|---|---|
| `src-tauri/Cargo.toml` | Dependensi Rust |
| `src-tauri/tauri.conf.json` | Konfigurasi window, bundle, updater |
| `src-tauri/src/main.rs` | Entry point |
| `src-tauri/src/lib.rs` | Window notch, shortcut Alt+N, commands storage |
| `src-tauri/capabilities/default.json` | Izin Tauri v2 |
| `src/index.html` | UI notch |
| `src/main.js` | Logika tab, todo, pomodoro, cuaca, musik demo |
| `src/styles.css` | Gaya notch (pil hitam, glass, animasi buka/tutup) |
| `ARCHITECTURE.md` | Desain lengkap + peta jalan ke v1.0 |

## Batasan yang jujur

- Ini MVP v0.1. Di sini ada 5 modul inti; modul lain menyusul di versi berikutnya.
- Musik SMTC Windows, notifikasi Windows, dan Dock butuh kode
  Win32 tambahan — sudah dipetakan di `ARCHITECTURE.md`, belum
  diimplementasikan.
- Belum ada code signing, jadi SmartScreen akan muncul seperti
  aplikasi tanpa sertifikat berbayar. Jangan bypass, klik More info → Run anyway hanya
  untuk build sendiri.
- Belum diuji di hardware nyata (dibangun di server Linux).
  Uji pertama harus di PC Windows: hover, Alt+N, multi-monitor,
  fullscreen, dan idle RAM.
