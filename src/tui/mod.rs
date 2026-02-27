use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, ProgressBar},
    Frame, Terminal,
};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct TUIState {
    pub total_sites: usize,
    pub completed: Arc<AtomicUsize>,
    pub found_results: Arc<AtomicUsize>,
    pub is_running: Arc<AtomicBool>,
    pub results: Arc<std::sync::Mutex<Vec<(String, String)>>>,
}

impl TUIState {
    pub fn new(total_sites: usize) -> Self {
        Self {
            total_sites,
            completed: Arc::new(AtomicUsize::new(0)),
            found_results: Arc::new(AtomicUsize::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
            results: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn increment_completed(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_result(&self, site: String, url: String) {
        self.found_results.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut results) = self.results.lock() {
            results.push((site, url));
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

pub fn run_tui(state: TUIState) -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(size);

            // Title
            let title = Paragraph::new("⚡ Watson OSINT Tool")
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Watson"));
            f.render_widget(title, chunks[0]);

            // Progress
            let completed = state.completed.load(Ordering::Relaxed);
            let total = state.total_sites;
            let progress = if total > 0 {
                (completed as f64 / total as f64 * 100.0) as u16
            } else {
                0
            };

            let progress_bar = ProgressBar::new(progress)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Progress"));
            f.render_widget(progress_bar, chunks[1]);

            // Stats
            let stats_text = format!(
                "Checked: {} / {} | Found: {}",
                completed,
                total,
                state.found_results.load(Ordering::Relaxed)
            );
            let stats = Paragraph::new(stats_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(stats, chunks[3]);

            // Results list
            let results = state.results.lock().unwrap();
            let items: Vec<ListItem> = results
                .iter()
                .rev()
                .take(15)
                .map(|(site, url)| {
                    ListItem::new(format!("[+] {}: {}", site, url))
                        .style(Style::default().fg(Color::Green))
                })
                .collect();

            let results_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Found Accounts"),
                )
                .style(Style::default().fg(Color::White));

            f.render_widget(results_list, chunks[2]);
        })?;

        if !state.is_running.load(Ordering::Relaxed) {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
