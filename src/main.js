// NotchKu frontend — vanilla JS, tanpa framework (ringan, target laptop 2 GB).
// Jalan di dua mode:
//  - Tauri: window.__TAURI__ ada, pakai invoke() ke Rust (storage + cuaca + resize).
//  - Browser biasa (npm run dev / demo): fallback localStorage + fetch langsung.

const isTauri = typeof window.__TAURI__ !== "undefined";
async function invoke(cmd, args = {}) {
  if (!isTauri) throw new Error("bukan Tauri");
  const { invoke } = window.__TAURI__.core;
  return invoke(cmd, args);
}

// ---------- Notch buka / tutup ----------
const notch = document.getElementById("notch");
const pill = document.getElementById("pill");
const panel = document.getElementById("panel");
let expanded = false;
let hoverTimer = null;

function renderNotch() {
  notch.classList.toggle("expanded", expanded);
  notch.classList.toggle("collapsed", !expanded);
  panel.hidden = !expanded;
  pill.style.display = expanded ? "none" : "flex";
}
async function setExpanded(open) {
  if (expanded === open) return;
  expanded = open;
  renderNotch();
  if (isTauri) {
    try { await invoke("set_notch_expanded", { open }); } catch { /* abaikan */ }
  }
}
pill.addEventListener("click", () => setExpanded(true));
document.getElementById("close-btn").addEventListener("click", () => setExpanded(false));
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") setExpanded(false);
  if (e.altKey && (e.key === "n" || e.key === "N")) { e.preventDefault(); setExpanded(!expanded); }
});
// Hover: buka setelah 150 ms, supaya tidak kebuka saat kursor cuma lewat.
pill.addEventListener("mouseenter", () => { hoverTimer = setTimeout(() => setExpanded(true), 150); });
pill.addEventListener("mouseleave", () => clearTimeout(hoverTimer));
notch.addEventListener("mouseleave", () => { if (expanded) setTimeout(() => setExpanded(false), 400); });

// ---------- Tab ----------
document.querySelectorAll("#tabs button").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll("#tabs button").forEach((b) => b.classList.remove("active"));
    document.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById("tab-" + btn.dataset.tab).classList.add("active");
  });
});

// ---------- Jam + baterai (pil tertutup) ----------
function tickClock() {
  const now = new Date();
  document.getElementById("pill-clock").textContent =
    String(now.getHours()).padStart(2, "0") + ":" + String(now.getMinutes()).padStart(2, "0");
}
tickClock(); setInterval(tickClock, 10000);
if (navigator.getBattery) {
  navigator.getBattery().then((b) => {
    const show = () => document.getElementById("pill-battery").textContent = Math.round(b.level * 100) + "%";
    show(); b.addEventListener("levelchange", show);
  });
}

// ---------- Todo (localStorage di browser, todos.json di Tauri) ----------
let todos = [];
async function loadTodos() {
  try {
    if (isTauri) todos = JSON.parse(await invoke("load_todos"));
    else todos = JSON.parse(localStorage.getItem("notchku-todos") || "[]");
  } catch { todos = []; }
  renderTodos();
}
async function saveTodos() {
  if (isTauri) { try { await invoke("save_todos", { todosJson: JSON.stringify(todos) }); } catch {} }
  else localStorage.setItem("notchku-todos", JSON.stringify(todos));
}
function renderTodos() {
  const ul = document.getElementById("todo-list");
  ul.innerHTML = "";
  todos.forEach((t, i) => {
    const li = document.createElement("li");
    if (t.done) li.classList.add("done");
    li.innerHTML = `<span>✅ ${t.text}</span><span><button data-act="toggle">✓</button> <button data-act="del">✕</button></span>`;
    li.querySelector('[data-act="toggle"]').onclick = () => { todos[i].done = !todos[i].done; saveTodos(); renderTodos(); };
    li.querySelector('[data-act="del"]').onclick = () => { todos.splice(i, 1); saveTodos(); renderTodos(); };
    ul.appendChild(li);
  });
  document.getElementById("todo-count").textContent = todos.filter((t) => !t.done).length;
  // Tugas berikutnya tampil di pil? MVP: tidak — jaga pil tetap ringkas.
}
document.getElementById("todo-form").addEventListener("submit", (e) => {
  e.preventDefault();
  const input = document.getElementById("todo-input");
  const text = input.value.trim();
  if (!text) return;
  todos.push({ text, done: false, created: Date.now() });
  input.value = "";
  saveTodos(); renderTodos();
});
if (!localStorage.getItem("notchku-todos") && !isTauri) {
  todos = [
    { text: "Coba hover notch di atas", done: false },
    { text: "Tes Pomodoro 25 menit", done: false },
    { text: "Cek cuaca Jakarta", done: true },
  ];
  saveTodos();
}
loadTodos();

// ---------- Pomodoro ----------
let pomoSecs = 25 * 60, pomoRunning = false, pomoTimer = null, pomoFocus = true;
const pomoEl = document.getElementById("pomo-time");
function renderPomo() {
  pomoEl.textContent = String(Math.floor(pomoSecs / 60)).padStart(2, "0") + ":" + String(pomoSecs % 60).padStart(2, "0");
  document.getElementById("pomo-label").textContent = pomoFocus ? "Fokus — 25 menit" : "Istirahat — 5 menit";
}
document.getElementById("pomo-start").addEventListener("click", (e) => {
  pomoRunning = !pomoRunning;
  e.target.textContent = pomoRunning ? "⏸ Jeda" : "▶ Mulai";
  if (pomoRunning) {
    pomoTimer = setInterval(() => {
      pomoSecs--;
      if (pomoSecs <= 0) { clearInterval(pomoTimer); pomoRunning = false; e.target.textContent = "▶ Mulai"; alert(pomoFocus ? "Fokus selesai. Istirahat!" : "Istirahat selesai."); }
      renderPomo();
    }, 1000);
  } else clearInterval(pomoTimer);
});
document.getElementById("pomo-reset").addEventListener("click", () => { clearInterval(pomoTimer); pomoRunning = false; pomoSecs = (pomoFocus ? 25 : 5) * 60; document.getElementById("pomo-start").textContent = "▶ Mulai"; renderPomo(); });
document.getElementById("pomo-mode").addEventListener("click", () => { pomoFocus = !pomoFocus; pomoSecs = (pomoFocus ? 25 : 5) * 60; renderPomo(); });
renderPomo();

// ---------- Musik demo ----------
const demoSongs = [
  { title: "Golden Hour Drive", artist: "Neon Avenue · Night Lines", dur: 204 },
  { title: "Lagu Demo Dua", artist: "NotchKu Band", dur: 187 },
  { title: "Lagu Demo Tiga", artist: "NotchKu Band", dur: 221 },
];
let songIdx = 0, songSecs = 0, playing = false, songTimer = null;
function renderSong() {
  const s = demoSongs[songIdx];
  document.getElementById("song-title").textContent = s.title;
  document.getElementById("song-artist").textContent = s.artist;
  document.getElementById("song-dur").textContent = Math.floor(s.dur / 60) + ":" + String(s.dur % 60).padStart(2, "0");
  document.getElementById("song-time").textContent = Math.floor(songSecs / 60) + ":" + String(songSecs % 60).padStart(2, "0");
  document.getElementById("song-progress").style.width = (songSecs / s.dur * 100) + "%";
}
document.getElementById("play-btn").addEventListener("click", (e) => {
  playing = !playing;
  e.target.textContent = playing ? "⏸" : "▶";
  if (playing) songTimer = setInterval(() => { songSecs++; if (songSecs >= demoSongs[songIdx].dur) { songIdx = (songIdx + 1) % demoSongs.length; songSecs = 0; } renderSong(); }, 1000);
  else clearInterval(songTimer);
});
document.getElementById("next-btn").addEventListener("click", () => { songIdx = (songIdx + 1) % demoSongs.length; songSecs = 0; renderSong(); });
document.getElementById("prev-btn").addEventListener("click", () => { songIdx = (songIdx - 1 + demoSongs.length) % demoSongs.length; songSecs = 0; renderSong(); });
renderSong();

// ---------- Kalender (bulan ini) ----------
(function renderCalendar() {
  const now = new Date();
  const y = now.getFullYear(), m = now.getMonth();
  document.getElementById("cal-month").textContent =
    now.toLocaleDateString("id-ID", { month: "long", year: "numeric" });
  const grid = document.getElementById("cal-grid");
  const days = new Date(y, m + 1, 0).getDate();
  for (let d = 1; d <= days; d++) {
    const div = document.createElement("div");
    div.textContent = d;
    if (d === now.getDate()) div.classList.add("today");
    grid.appendChild(div);
  }
  document.getElementById("cal-events").innerHTML =
    "<li><span>📅 Rapat tim — besok 09:00</span><span>Join</span></li>" +
    "<li><span>📅 Review ebook — Jumat 14:00</span><span>—</span></li>";
})();

// ---------- Cuaca (Open-Meteo, Jakarta default) ----------
const WX = { "-6.2,106.8": "Jakarta" };
async function loadWeather() {
  const el = document.getElementById("wx-desc");
  try {
    let data;
    if (isTauri) data = JSON.parse(await invoke("fetch_weather", { latitude: -6.2, longitude: 106.8 }));
    else data = await (await fetch("https://api.open-meteo.com/v1/forecast?latitude=-6.2&longitude=106.8&current=temperature_2m,weather_code,relative_humidity_2m")).json();
    const t = Math.round(data.current.temperature_2m);
    const codes = { 0: "Cerah", 1: "Cerah berawan", 2: "Berawan", 3: "Mendung", 61: "Hujan ringan", 95: "Hujan badai" };
    const desc = codes[data.current.weather_code] || ("Kode cuaca " + data.current.weather_code);
    document.getElementById("wx-temp").textContent = t + "°C";
    document.getElementById("wx-desc").textContent = desc;
    document.getElementById("pill-weather").textContent = t + "°";
    document.getElementById("wx-meta").textContent =
      "Kelembapan " + data.current.relative_humidity_2m + "% · waktu data: " + data.current.time;
  } catch {
    el.textContent = "Cuaca tidak bisa dimuat (offline?)";
  }
}
document.getElementById("wx-refresh").addEventListener("click", loadWeather);
loadWeather();

renderNotch();
