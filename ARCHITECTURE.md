# Arsitektur NotchKu — MVP v0.1

NotchKu adalah aplikasi notch interaktif buatanku sendiri untuk Windows,
dibangun dari nol dengan Rust + Tauri v2 dan vanilla JS.

## 1. Model mental NotchKu

NotchKu bukan aplikasi window biasa. Ia adalah **satu window overlay**:

- Posisi: top-center, menempel di tepi atas layar.
- State tertutup: pil hitam kecil (~200×32 px). Isinya ringkas:
  jam di tengah, info kiri/kanan (cuaca, baterai, lagu, tugas).
- State terbuka: panel turun ke bawah (~560×380 px), berisi tab.
- Trigger buka: hover kursor di pil, klik, atau `Alt+N`.
- Trigger tutup: kursor pergi, `Esc`, `Alt+N` lagi, atau klik di luar.
- Selalu di atas (`always_on_top`), tapi mundur saat ada app
  fullscreen, dan tidak muncul di screenshot kecuali diizinkan.
- Satu notch per layar (multi-monitor: window per display).

Idenya: Dynamic Island di iOS, dibawa ke desktop — sebagai konsep
umum, bukan menyalin aplikasi mana pun.

## 2. Stack yang dipakai

| Komponen | Pilihan |
|---|---|
| Bahasa Rust | Backend + window manager |
| Tauri v2 | Rust + WebView (WebView2 di Windows, WebKitGTK di Linux) |
| Frontend | Vanilla JS (tanpa framework) supaya ringan dan gampang dibaca — cocok untuk target laptop 2 GB |
| Data lokal | `%APPDATA%\NotchKu` |

## 3. Window manager (inti tersulit)

Inilah bagian yang membedakan notch dengan aplikasi biasa:

1. **Frameless + transparent.** Window tanpa title bar, background
   transparan, konten digambar sebagai pil membulat (border-radius).
2. **Click-through selektif.** Saat tertutup, hanya area pil yang
   menerima klik. Area transparan di sekitarnya harus tembus ke
   window di belakang (di Tauri: `set_ignore_cursor_events(true)`
   kecuali di atas pil; di Win32: `WS_EX_TRANSPARENT` + hit-test).
3. **Resize animasi.** Buka/tutup = ubah ukuran window
   (mis. 200×32 → 560×380) dengan animasi di frontend (CSS
   transition) + `set_size` di Rust. Jangan buat dua window —
   satu window, dua ukuran.
4. **Hover detection.** Rust mendengarkan mouse enter/leave di
   area pil, atau frontend mengirim event `mouseenter` lewat IPC.
   Tambahkan delay ~150 ms sebelum membuka, supaya tidak kebuka
   saat kursor cuma lewat.
5. **Fullscreen / Show Desktop.** Dengarkan perubahan foreground
   window (Win32 `EVENT_SYSTEM_FOREGROUND`, atau polling
   `GetForegroundWindow`). Kalau app fullscreen, sembunyikan notch.
   Di MVP, window tidak boleh di-minimize oleh OS; kalau
   disembunyikan dari luar, Rust mengembalikannya sendiri.
6. **Multi-monitor.** Buat satu window per display, posisi
   dihitung dari `monitor.size` masing-masing.

Di `src-tauri/src/lib.rs` sudah ada kerangka: `setup_notch_window`,
`toggle_notch`, dan shortcut `Alt+N` via
`tauri-plugin-global-shortcut`.

## 4. Modul (frontend + Rust)

| Modul | Frontend | Rust / API eksternal |
|---|---|---|
| Musik | `src/main.js` — kontrol + progress | Windows: SMTC via `windows-rs` (`GlobalSystemMediaTransportControlsSessionManager`). Linux: MPRIS via D-Bus. Mac: MediaRemote. MVP: demo lokal. |
| Kalender | render bulan + event | Google Calendar / Outlook / iCal `.ics` — fetch + parse di Rust, cache lokal. |
| Todo | CRUD + due date teks ("besok jam 9") | Simpan `todos.json` via command `save_todos` / `load_todos`. Parser tanggal: crate `chrono` + aturan sederhana ID/EN. |
| Pomodoro | timer `setInterval`, bunyi selesai | State di Rust supaya tetap jalan walau WebView di-reload. MVP: state di JS + `localStorage`. |
| Cuaca | fetch langsung dari JS | Open-Meteo (tanpa key). Jangan hardcode key pihak ketiga. |
| Settings | form | `settings.json` di app-data dir, dibaca saat start. |

Penyimpanan: JSON polos di app-data dir, bukan SQLite, supaya
gampang di-backup dan di-debug. Enkripsi tidak wajib di MVP.

## 5. Update & distribusi

- Bundle NSIS `*_x64-setup.exe` (disarankan) + portable `.exe`.
- `tauri-plugin-updater`: cek `latest.json` di GitHub Releases
  saat start + tiap 6 jam. Update di-start langsung, atau tunggu
  idle lalu install + restart sendiri.
- Setiap update **harus ditandatangani** (signing key updater),
  aplikasi menolak file yang signaturenya tidak cocok.
- Tanpa code signing berbayar, SmartScreen akan muncul.
  Itu normal, bukan tanda malware.
- Mac/Linux: update manual (unduh ulang, pasang di atas yang lama,
  settings tetap).

## 6. Privasi

- Default: tidak ada jaringan kecuali cuaca + cek update.
- Setiap integrasi (kalender, AI, lirik) opt-in, dan key API
  milik pengguna disimpan lokal, tidak dikirim ke server kita.
- Tidak ada ID perangkat, tidak ada telemetri. Kalau mau hitung
  pengguna, pakai hitungan anonim harian (satu hit per hari,
  per negara, tanpa IP) — dan sediakan tombol matikan.

## 7. Performa — target laptop 2 GB

- Idle: < 80 MB RAM, < 1% CPU (WebView diam, timer 1 detik).
- Jangan polling SMTC/kalender tiap detik; pakai event + interval
  5–30 detik.
- Matikan animasi berat saat idle; visualizer musik hanya saat
  tab Musik terbuka.
- Uji di laptop 2 GB sebelum menambah modul: ukur idle RAM
  di Task Manager setelah 10 menit.

## 8. Peta jalan

- **v0.1 (ini):** shell notch + 5 modul inti + build Windows.
- **v0.2:** SMTC Windows asli, notifikasi Windows, Dock bawah.
- **v0.3:** kalender Google/iCal nyata, lirik LRCLIB, jadwal sholat.
- **v1.0:** AI Chat (Gemini/Claude/Ollama, key pengguna), Shelf,
  update otomatis + signing, Mac/Linux beta.

## 9. Prinsip desain

- NotchKu adalah karya orisinal: nama, logo, dan semua teks UI
  ditulis sendiri, bukan menyalin aplikasi lain.
- Semua aset (ikon, gambar, teks) dibuat sendiri atau dari
  sumber berlisensi bebas.
