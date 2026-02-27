use crate::engine::ProgressUpdate;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct TUIState {
    pub total_sites: Arc<AtomicUsize>,
    pub completed: Arc<AtomicUsize>,
    pub found_results: Arc<AtomicUsize>,
    pub is_running: Arc<AtomicBool>,
    pub results: Arc<Mutex<Vec<(String, String)>>>,
    pub current_site: Arc<Mutex<String>>,
    pub start_time: Arc<Mutex<Instant>>,
}

impl TUIState {
    pub fn new(total_sites: usize) -> Self {
        Self {
            total_sites: Arc::new(AtomicUsize::new(total_sites)),
            completed: Arc::new(AtomicUsize::new(0)),
            found_results: Arc::new(AtomicUsize::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
            results: Arc::new(Mutex::new(Vec::new())),
            current_site: Arc::new(Mutex::new(String::new())),
            start_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn handle_progress(&self, update: ProgressUpdate) {
        match update {
            ProgressUpdate::Started { total, .. } => {
                self.total_sites.store(total, Ordering::Relaxed);
            }
            ProgressUpdate::SiteChecked { site, url, found } => {
                self.completed.fetch_add(1, Ordering::Relaxed);
                if found {
                    self.found_results.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut results) = self.results.lock() {
                        results.push((site.clone(), url.clone()));
                    }
                }
                if let Ok(mut current) = self.current_site.lock() {
                    *current = site;
                }
            }
            ProgressUpdate::Completed { .. } => {
                self.is_running.store(false, Ordering::Relaxed);
            }
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
                    Constraint::Length(4),
                    Constraint::Min(8),
                    Constraint::Length(5),
                ])
                .split(size);

            // Title
            let title = Paragraph::new("⚡ Watson OSINT Tool")
                .style(Style::default().fg(Color::Cyan).bold())
                .block(Block::default().borders(Borders::ALL).title(" Watson "));
            f.render_widget(title, chunks[0]);

            // Progress bar
            let completed = state.completed.load(Ordering::Relaxed);
            let total = state.total_sites.load(Ordering::Relaxed);
            let progress = if total > 0 {
                (completed as f64 / total as f64 * 100.0) as u16
            } else {
                0
            };

            let progress_bar = Gauge::default()
                .gauge_style(Style::default().fg(Color::Green))
                .label(format!("{} / {} sites checked", completed, total))
                .percent(progress);
            f.render_widget(progress_bar, chunks[1]);

            // Results list
            let results = state.results.lock().unwrap();
            let items: Vec<ListItem> = results
                .iter()
                .rev()
                .take(12)
                .map(|(site, url)| {
                    ListItem::new(format!("[✓] {}: {}", site, url))
                        .style(Style::default().fg(Color::Green))
                })
                .collect();

            let results_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Found Accounts "),
                )
                .style(Style::default().fg(Color::White));

            f.render_widget(results_list, chunks[2]);

            // Status bar
            let elapsed = state.start_time.lock().unwrap().elapsed();
            let elapsed_str = format_time(elapsed.as_secs());

            let current_site = state.current_site.lock().unwrap().clone();
            let status_text = if current_site.is_empty() {
                format!(
                    "Found: {} | Elapsed: {} | Status: {}",
                    state.found_results.load(Ordering::Relaxed),
                    elapsed_str,
                    if state.is_running.load(Ordering::Relaxed) {
                        "Searching..."
                    } else {
                        "Complete"
                    }
                )
            } else {
                format!(
                    "Found: {} | Elapsed: {} | Current: {}",
                    state.found_results.load(Ordering::Relaxed),
                    elapsed_str,
                    current_site
                )
            };

            let status = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Status "));
            f.render_widget(status, chunks[3]);
        })?;

        if !state.is_running.load(Ordering::Relaxed) {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Final render to show completion
    terminal.draw(|f| {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Min(8),
                Constraint::Length(5),
            ])
            .split(size);

        let title = Paragraph::new("⚡ Watson OSINT Tool - COMPLETE")
            .style(Style::default().fg(Color::Green).bold())
            .block(Block::default().borders(Borders::ALL).title(" Watson "));
        f.render_widget(title, chunks[0]);

        let completed = state.completed.load(Ordering::Relaxed);
        let found = state.found_results.load(Ordering::Relaxed);
        let elapsed = state.start_time.lock().unwrap().elapsed();
        let elapsed_str = format_time(elapsed.as_secs());

        let summary = Paragraph::new(format!(
            "Total checked: {} | Found: {} | Time: {}",
            completed, found, elapsed_str
        ))
        .style(Style::default().fg(Color::Green).bold())
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
        f.render_widget(summary, chunks[1]);

        let results = state.results.lock().unwrap();
        let items: Vec<ListItem> = results
            .iter()
            .map(|(site, url)| {
                ListItem::new(format!("[✓] {}: {}", site, url))
                    .style(Style::default().fg(Color::Green))
            })
            .collect();

        let results_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Found Accounts "),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(results_list, chunks[2]);
    })?;

    Ok(())
}

fn format_time(seconds: u64) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}
