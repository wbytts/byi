use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;

pub fn run(config_dir: PathBuf, data_dir: PathBuf) -> Result<(), String> {
    let mut terminal = setup_terminal()?;
    let result = app::App::new(config_dir, data_dir).and_then(|mut app| {
        let res = run_app(&mut terminal, &mut app);
        restore_terminal(&mut terminal)?;
        res
    });
    if result.is_err() {
        let _ = restore_terminal(&mut terminal);
    }
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, String> {
    enable_raw_mode().map_err(|e| format!("启用 raw mode 失败: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("进入 alternate screen 失败: {e}"))?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(|e| format!("创建终端失败: {e}"))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), String> {
    disable_raw_mode().map_err(|e| format!("禁用 raw mode 失败: {e}"))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| format!("离开 alternate screen 失败: {e}"))?;
    terminal
        .show_cursor()
        .map_err(|e| format!("显示光标失败: {e}"))
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> Result<(), String> {
    // Initial draw
    terminal
        .draw(|f| ui::draw(f, app))
        .map_err(|e| format!("绘制失败: {e}"))?;

    loop {
        // Block until input event (no busy-wait, no periodic redraw)
        if crossterm::event::poll(std::time::Duration::from_millis(500)).map_err(|e| format!("事件轮询失败: {e}"))? {
            match event::read().map_err(|e| format!("读取事件失败: {e}"))? {
                Event::Key(key) => {
                    if app.handle_key(key.code, key.modifiers) {
                        return Ok(());
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse);
                }
                _ => continue,
            }
            // Redraw only after handling an event
            terminal
                .draw(|f| ui::draw(f, app))
                .map_err(|e| format!("绘制失败: {e}"))?;
        }

        // Expire status message (no redraw needed — next event will pick it up)
        app.on_tick();
    }
}
