using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Threading;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace NotchKu
{
    public class TodoItem : INotifyPropertyChanged
    {
        private string _text = "";
        private bool _done;

        public string Text { get => _text; set { _text = value; OnPropertyChanged(nameof(Text)); } }
        public bool Done { get => _done; set { _done = value; OnPropertyChanged(nameof(Done)); } }

        public event PropertyChangedEventHandler PropertyChanged;
        private void OnPropertyChanged(string name) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }

    public partial class MainWindow : Window
    {
        private const int HOTKEY_ID = 9000;
        private const int WM_HOTKEY = 0x0312;

        [DllImport("user32.dll")]
        private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);
        [DllImport("user32.dll")]
        private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

        private const uint MOD_ALT = 0x0001;
        private const uint VK_N = 0x4E;

        private bool _expanded;
        private readonly DispatcherTimer _clockTimer = new DispatcherTimer();
        private readonly DispatcherTimer _pomodoroTimer = new DispatcherTimer();
        private readonly DispatcherTimer _musicTimer = new DispatcherTimer();
        private readonly HttpClient _http = new HttpClient { Timeout = TimeSpan.FromSeconds(10) };

        private readonly ObservableCollection<TodoItem> _todos = new ObservableCollection<TodoItem>();
        private string TodosPath => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "NotchKu", "todos.json");

        // Demo music
        private readonly string[] _songs = { "Lofi Morning — Demo", "Rainy Window — Demo", "Night Drive — Demo" };
        private readonly int[] _durations = { 187, 214, 163 }; // seconds
        private int _songIndex = 0;
        private int _musicPosition = 0;
        private bool _musicPlaying;

        // Pomodoro
        private int _pomodoroTotal = 25 * 60;
        private int _pomodoroRemaining = 25 * 60;
        private bool _pomodoroRunning;
        private bool _pomodoroFocus = true;

        // Calendar
        private DateTime _calMonth = new DateTime(DateTime.Now.Year, DateTime.Now.Month, 1);

        public MainWindow()
        {
            InitializeComponent();
            TodoList.ItemsSource = _todos;
            LoadTodos();
            SongList.ItemsSource = _songs;
            SongList.SelectedIndex = 0;

            _clockTimer.Interval = TimeSpan.FromSeconds(1);
            _clockTimer.Tick += (s, e) => UpdateClock();
            _clockTimer.Start();

            _pomodoroTimer.Interval = TimeSpan.FromSeconds(1);
            _pomodoroTimer.Tick += PomodoroTick;

            _musicTimer.Interval = TimeSpan.FromSeconds(1);
            _musicTimer.Tick += MusicTick;

            UpdateClock();
            RenderCalendar();
            UpdatePomodoroDisplay();
            _ = FetchWeatherAsync();
        }

        private void Window_Loaded(object sender, RoutedEventArgs e)
        {
            CenterTop();
            var helper = new WindowInteropHelper(this);
            var source = HwndSource.FromHwnd(helper.Handle);
            source?.AddHook(WndProc);
            RegisterHotKey(helper.Handle, HOTKEY_ID, MOD_ALT, VK_N);
        }

        protected override void OnClosed(EventArgs e)
        {
            var helper = new WindowInteropHelper(this);
            UnregisterHotKey(helper.Handle, HOTKEY_ID);
            SaveTodos();
            base.OnClosed(e);
        }

        private IntPtr WndProc(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam, ref bool handled)
        {
            if (msg == WM_HOTKEY && wParam.ToInt32() == HOTKEY_ID)
            {
                if (!IsVisible) { Show(); }
                ToggleExpand();
                handled = true;
            }
            return IntPtr.Zero;
        }

        private void CenterTop()
        {
            Left = (SystemParameters.PrimaryScreenWidth - Width) / 2;
            Top = 0;
        }

        private void UpdateClock()
        {
            var now = DateTime.Now;
            var text = now.ToString("HH:mm", CultureInfo.InvariantCulture);
            CollapsedClock.Text = text;
            ExpandedClock.Text = text;
            ExpandedDate.Text = now.ToString("ddd, d MMM yyyy", new CultureInfo("id-ID"));
        }

        // ---------- Expand / collapse ----------

        private void ToggleExpand()
        {
            if (_expanded) Collapse(); else Expand();
        }

        private void Expand()
        {
            _expanded = true;
            Width = 560;
            Height = 380;
            RootBorder.CornerRadius = new CornerRadius(18);
            CollapsedView.Visibility = Visibility.Collapsed;
            ExpandedView.Visibility = Visibility.Visible;
            CenterTop();
        }

        private void Collapse()
        {
            _expanded = false;
            Width = 220;
            Height = 32;
            RootBorder.CornerRadius = new CornerRadius(16);
            ExpandedView.Visibility = Visibility.Collapsed;
            CollapsedView.Visibility = Visibility.Visible;
            CenterTop();
        }

        private void CollapsedView_MouseLeftButtonUp(object sender, MouseButtonEventArgs e) => Expand();
        private void CollapseButton_Click(object sender, RoutedEventArgs e) => Collapse();
        private void HideButton_Click(object sender, RoutedEventArgs e) => Hide();

        // ---------- Weather (Open-Meteo) ----------

        private async Task FetchWeatherAsync()
        {
            try
            {
                const string url = "https://api.open-meteo.com/v1/forecast?latitude=-6.2&longitude=106.8&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m";
                var json = await _http.GetStringAsync(url);
                var cur = JObject.Parse(json)["current"];
                if (cur == null) return;
                var temp = cur["temperature_2m"]?.Value<double>() ?? 0;
                var code = cur["weather_code"]?.Value<int>() ?? 0;
                var humidity = cur["relative_humidity_2m"]?.Value<int>() ?? 0;
                var wind = cur["wind_speed_10m"]?.Value<double>() ?? 0;
                var condition = WeatherCodeLabel(code);
                var tempText = $"{temp:0}°C";
                CollapsedWeather.Text = tempText;
                WeatherTemp.Text = tempText;
                WeatherCondition.Text = condition;
                WeatherDetail.Text = $"Kelembapan {humidity}%  •  Angin {wind:0} km/jam";
            }
            catch
            {
                WeatherCondition.Text = "Cuaca tidak tersedia";
            }
        }

        private static string WeatherCodeLabel(int code)
        {
            switch (code)
            {
                case 0: return "Cerah";
                case 1: return "Cerah Berawan";
                case 2: return "Berawan";
                case 3: return "Mendung";
                case 45: case 48: return "Berkabut";
                case 51: case 53: case 55: return "Gerimis";
                case 61: case 63: case 65: return "Hujan";
                case 71: case 73: case 75: return "Salju";
                case 80: case 81: case 82: return "Hujan Deras";
                case 95: return "Badai Petir";
                default: return "Berawan";
            }
        }

        private void WeatherRefreshButton_Click(object sender, RoutedEventArgs e)
        {
            WeatherCondition.Text = "Memuat...";
            _ = FetchWeatherAsync();
        }

        // ---------- Music (demo) ----------

        private void SongList_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (SongList.SelectedIndex < 0) return;
            _songIndex = SongList.SelectedIndex;
            _musicPosition = 0;
            UpdateMusicDisplay();
        }

        private void PlayPauseButton_Click(object sender, RoutedEventArgs e)
        {
            _musicPlaying = !_musicPlaying;
            PlayPauseButton.Content = _musicPlaying ? "⏸" : "▶";
            if (_musicPlaying) _musicTimer.Start(); else _musicTimer.Stop();
            UpdateMusicDisplay();
        }

        private void PrevButton_Click(object sender, RoutedEventArgs e)
        {
            _songIndex = (_songIndex + _durations.Length - 1) % _durations.Length;
            SongList.SelectedIndex = _songIndex;
        }

        private void NextButton_Click(object sender, RoutedEventArgs e)
        {
            _songIndex = (_songIndex + 1) % _durations.Length;
            SongList.SelectedIndex = _songIndex;
        }

        private void MusicTick(object sender, EventArgs e)
        {
            _musicPosition++;
            if (_musicPosition >= _durations[_songIndex])
            {
                NextButton_Click(sender, null);
                return;
            }
            UpdateMusicDisplay();
        }

        private static string FormatTime(int seconds) => $"{seconds / 60}:{seconds % 60:00}";

        private void UpdateMusicDisplay()
        {
            var total = _durations[_songIndex];
            MusicProgress.Maximum = total;
            MusicProgress.Value = _musicPosition;
            MusicPosition.Text = FormatTime(_musicPosition);
            MusicDuration.Text = FormatTime(total);
            NowPlaying.Text = _musicPlaying ? $"Memutar: {_songs[_songIndex]} (demo)" : $"Jeda: {_songs[_songIndex]} (demo)";
        }

        private void MusicProgress_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            // Slider is display-only in demo mode.
        }

        // ---------- Calendar ----------

        private void RenderCalendar()
        {
            CalendarMonthLabel.Text = _calMonth.ToString("MMMM yyyy", new CultureInfo("id-ID"));
            CalendarGrid.Children.Clear();
            var dayNames = new[] { "Sn", "Sl", "Rb", "Km", "Jm", "Sb", "Mg" };
            foreach (var d in dayNames)
            {
                CalendarGrid.Children.Add(new TextBlock
                {
                    Text = d, Foreground = new SolidColorBrush(Color.FromRgb(0x99, 0x99, 0x99)),
                    FontSize = 11, HorizontalAlignment = HorizontalAlignment.Center, Margin = new Thickness(0, 0, 0, 4)
                });
            }
            var first = new DateTime(_calMonth.Year, _calMonth.Month, 1);
            // Monday-first layout: .NET DayOfWeek Sunday=0
            int offset = ((int)first.DayOfWeek + 6) % 7;
            int daysInMonth = DateTime.DaysInMonth(_calMonth.Year, _calMonth.Month);
            var today = DateTime.Today;
            for (int i = 0; i < 42; i++)
            {
                var cell = new Border { CornerRadius = new CornerRadius(6), Margin = new Thickness(1) };
                int day = i - offset + 1;
                if (day >= 1 && day <= daysInMonth)
                {
                    bool isToday = _calMonth.Year == today.Year && _calMonth.Month == today.Month && day == today.Day;
                    cell.Background = isToday ? new SolidColorBrush(Color.FromRgb(0xFF, 0x8C, 0x42)) : Brushes.Transparent;
                    cell.Child = new TextBlock
                    {
                        Text = day.ToString(),
                        Foreground = isToday ? Brushes.White : new SolidColorBrush(Color.FromRgb(0xDD, 0xDD, 0xDD)),
                        FontSize = 12, HorizontalAlignment = HorizontalAlignment.Center,
                        VerticalAlignment = VerticalAlignment.Center, Margin = new Thickness(0, 3, 0, 3)
                    };
                }
                CalendarGrid.Children.Add(cell);
            }
        }

        private void CalPrev_Click(object sender, RoutedEventArgs e)
        {
            _calMonth = _calMonth.AddMonths(-1);
            RenderCalendar();
        }

        private void CalNext_Click(object sender, RoutedEventArgs e)
        {
            _calMonth = _calMonth.AddMonths(1);
            RenderCalendar();
        }

        // ---------- Todo ----------

        private void LoadTodos()
        {
            try
            {
                if (!File.Exists(TodosPath)) return;
                var items = JsonConvert.DeserializeObject<List<TodoItem>>(File.ReadAllText(TodosPath));
                if (items == null) return;
                foreach (var item in items) _todos.Add(item);
            }
            catch { /* start empty */ }
        }

        private void SaveTodos()
        {
            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(TodosPath));
                File.WriteAllText(TodosPath, JsonConvert.SerializeObject(_todos.ToList()));
            }
            catch { /* ignore save errors */ }
        }

        private void AddTodo()
        {
            var text = NewTodoBox.Text.Trim();
            if (text.Length == 0) return;
            _todos.Add(new TodoItem { Text = text, Done = false });
            NewTodoBox.Text = "";
            SaveTodos();
        }

        private void AddTodoButton_Click(object sender, RoutedEventArgs e) => AddTodo();

        private void NewTodoBox_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter) AddTodo();
        }

        private void DeleteTodoButton_Click(object sender, RoutedEventArgs e)
        {
            if ((sender as Button)?.DataContext is TodoItem item)
            {
                _todos.Remove(item);
                SaveTodos();
            }
        }

        private void TodoCheck_Changed(object sender, RoutedEventArgs e) => SaveTodos();

        // ---------- Pomodoro ----------

        private void PomodoroTick(object sender, EventArgs e)
        {
            if (_pomodoroRemaining > 0) _pomodoroRemaining--;
            if (_pomodoroRemaining == 0)
            {
                _pomodoroRunning = false;
                _pomodoroTimer.Stop();
                PomodoroStartButton.Content = "Mulai";
                PomodoroStatus.Text = _pomodoroFocus ? "Fokus selesai. Waktunya istirahat." : "Istirahat selesai. Siap fokus lagi.";
            }
            UpdatePomodoroDisplay();
        }

        private void UpdatePomodoroDisplay()
        {
            PomodoroTime.Text = FormatTime(_pomodoroRemaining);
            PomodoroMode.Text = _pomodoroFocus ? "Fokus" : "Istirahat";
            PomodoroProgress.Value = _pomodoroTotal > 0
                ? 100.0 * (_pomodoroTotal - _pomodoroRemaining) / _pomodoroTotal
                : 0;
        }

        private void SetPomodoroMode(bool focus)
        {
            _pomodoroFocus = focus;
            _pomodoroTotal = focus ? 25 * 60 : 5 * 60;
            _pomodoroRemaining = _pomodoroTotal;
            _pomodoroRunning = false;
            _pomodoroTimer.Stop();
            PomodoroStartButton.Content = "Mulai";
            PomodoroStatus.Text = "";
            UpdatePomodoroDisplay();
        }

        private void PomodoroStartButton_Click(object sender, RoutedEventArgs e)
        {
            _pomodoroRunning = !_pomodoroRunning;
            PomodoroStartButton.Content = _pomodoroRunning ? "Jeda" : "Mulai";
            if (_pomodoroRunning) _pomodoroTimer.Start(); else _pomodoroTimer.Stop();
        }

        private void PomodoroResetButton_Click(object sender, RoutedEventArgs e)
        {
            _pomodoroRemaining = _pomodoroTotal;
            _pomodoroRunning = false;
            _pomodoroTimer.Stop();
            PomodoroStartButton.Content = "Mulai";
            PomodoroStatus.Text = "";
            UpdatePomodoroDisplay();
        }

        private void PomodoroFocusButton_Click(object sender, RoutedEventArgs e) => SetPomodoroMode(true);
        private void PomodoroBreakButton_Click(object sender, RoutedEventArgs e) => SetPomodoroMode(false);
    }
}
