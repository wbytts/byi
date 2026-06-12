use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        canvas::{Canvas, Points},
        Block, BorderType, Borders, Cell, Clear, Gauge,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Sparkline, Table, Tabs,
        Wrap,
    },
    Frame,
};
use tui_big_text::{BigTextBuilder, PixelSize};

use crate::app::{App, Popup, Tab};

// ── Catppuccin Mocha palette ──────────────────────────────────────
const BG: Color = Color::Rgb(24, 24, 37);
const SURFACE: Color = Color::Rgb(30, 30, 46);
const OVERLAY: Color = Color::Rgb(49, 50, 68);
const TEXT: Color = Color::Rgb(205, 214, 244);
const SUBTEXT: Color = Color::Rgb(166, 173, 200);
const LAVENDER: Color = Color::Rgb(180, 190, 254);
const BLUE: Color = Color::Rgb(137, 180, 250);
const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
const TEAL: Color = Color::Rgb(148, 226, 213);
const GREEN: Color = Color::Rgb(166, 227, 161);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const PEACH: Color = Color::Rgb(250, 179, 135);
const RED: Color = Color::Rgb(243, 139, 168);
const PINK: Color = Color::Rgb(245, 194, 231);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const FLAMINGO: Color = Color::Rgb(245, 224, 220);

const SPARKLES: &[&str] = &["✦", "✧", "⋆", "★", "✶", "✴", "✳", "✺"];
const THROBBER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SKELETON_CHARS: &[&str] = &["░", "▒", "▓", "▒"];


/// Generate a "breathing" border color that oscillates between overlay and the given accent
fn glow_color(tick: u64, accent: Color, speed: u64) -> Color {
    let phase = (tick % (speed * 2)) as f64 / (speed * 2) as f64;
    let alpha = (phase * std::f64::consts::PI).sin() * 0.5 + 0.5; // 0..1
    let accent_rgb = match accent {
        Color::Rgb(_, g, b) => (g, b),
        _ => return accent,
    };
    let base = match OVERLAY { Color::Rgb(r, g, b) => (r, g, b), _ => (49, 50, 68) };
    let accent_r = match accent { Color::Rgb(r, _, _) => r, _ => 100 };
    let r = (base.0 as f64 + (accent_r as f64 - base.0 as f64) * alpha) as u8;
    let g = (base.1 as f64 + (accent_rgb.0 as f64 - base.1 as f64) * alpha) as u8;
    let b = (base.2 as f64 + (accent_rgb.1 as f64 - base.2 as f64) * alpha) as u8;
    Color::Rgb(r, g, b)
}

/// Unicode decorative divider line
fn decorative_divider(width: u16, tick: u64) -> Line<'static> {
    let sparkle = SPARKLES[tick as usize % SPARKLES.len()];
    let dash_count = (width as usize).saturating_sub(4) / 2;
    let left = "─".repeat(dash_count);
    let right = "─".repeat(dash_count);
    Line::from(vec![
        Span::styled(left, Style::default().fg(OVERLAY)),
        Span::styled(sparkle, Style::default().fg(MAUVE)),
        Span::styled(right, Style::default().fg(OVERLAY)),
    ])
}
// ── Main draw ─────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    if let Some(popup) = &app.popup {
        draw_popup(f, app, popup);
    }
}

// ── Status bar ────────────────────────────────────────────────────

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let spinner = THROBBER[app.tick as usize % THROBBER.len()];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(10)])
        .split(area);

    // Left side: keys or status message
    if app.status_message.is_empty() {
        let keys = vec![
            Span::styled(format!(" {spinner} "), Style::default().fg(MAUVE)),
            Span::styled(" q ", Style::default().bg(RED).fg(BG).bold()),
            Span::styled(" 退出 ", Style::default().fg(SUBTEXT)),
            Span::styled(" 1/2/3 ", Style::default().bg(BLUE).fg(BG).bold()),
            Span::styled(" 切换 ", Style::default().fg(SUBTEXT)),
        ];
        f.render_widget(Paragraph::new(Line::from(keys)).style(Style::default().bg(BG)), chunks[0]);
    } else {
        let msg_color = if app.status_ttl < 10 { SUBTEXT } else { LAVENDER };
        let line = Line::from(vec![
            Span::styled(format!(" {spinner} "), Style::default().fg(MAUVE)),
            Span::styled(format!("{}", app.status_message), Style::default().fg(msg_color).bold()),
        ]);
        f.render_widget(Paragraph::new(line).style(Style::default().bg(BG)), chunks[0]);
    }

    // Right side: version badge
    let ver = Line::from(Span::styled(" v0.0.1 ", Style::default().fg(BG).bg(MAUVE).bold()));
    f.render_widget(
        Paragraph::new(ver).alignment(Alignment::Right).style(Style::default().bg(BG)),
        chunks[1],
    );
}

// ── Tabs ──────────────────────────────────────────────────────────

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tab_index = app.current_tab as usize;
    let tab_colors = [LAVENDER, SAPPHIRE, TEAL];
    let tab_icons = ["⌂", "⚡", "⟳"];
    let tab_labels = ["Home", "Skills", "Sync"];
    let sparkle = SPARKLES[app.tick as usize % SPARKLES.len()];

    let titles: Vec<Line> = (0..3)
        .map(|i| {
            let style = if i == tab_index {
                Style::default().fg(tab_colors[i]).bold()
            } else {
                Style::default().fg(SUBTEXT)
            };
            Line::from(Span::styled(format!(" {} {} ", tab_icons[i], tab_labels[i]), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(format!(" {sparkle} byi "), Style::default().fg(MAUVE).bold()))
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(glow_color(app.tick, tab_colors[tab_index], 50)))
                .style(Style::default().bg(BG)),
        )
        .select(tab_index)
        .highlight_style(
            Style::default().fg(BG).bg(tab_colors[tab_index]).add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(OVERLAY)));
    f.render_widget(tabs, area);
}

// ── Content ───────────────────────────────────────────────────────

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Home => draw_home(f, app, area),
        Tab::Skills => draw_skills(f, app, area),
        Tab::Sync => draw_sync(f, app, area),
    }
}

// ── Home: big banner + canvas waves + gauges ──────────────────────

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(glow_color(app.tick, LAVENDER, 80)))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // banner
            Constraint::Length(6),  // canvas wave
            Constraint::Length(3),  // gauges
            Constraint::Min(0),     // shortcuts
        ])
        .split(inner);

    // ── Animated banner ──
    let cycle = (app.tick / 8) as usize % 7;
    let banner_colors = [MAUVE, LAVENDER, BLUE, SAPPHIRE, TEAL, PINK, FLAMINGO];
    let big = BigTextBuilder::default()
        .pixel_size(PixelSize::HalfHeight)
        .lines(vec![Line::from(Span::styled(
            "byi",
            Style::default().fg(banner_colors[cycle]).bold(),
        ))])
        .build();
    f.render_widget(big, chunks[0]);

    // ── Particle field animation ──
    draw_particle_field(f, app, chunks[1]);

    // ── Gauges row ──
    let total = app.skill_entries.len().max(1);
    let enabled = app.skill_entries.iter().filter(|e| e.installed.enabled).count();
    let github_count = app.skill_entries.iter().filter(|e| e.installed.source.starts_with("github:")).count();

    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[2]);

    let enabled_ratio = if total > 0 { enabled as f64 / total as f64 } else { 0.0 };
    f.render_widget(
        Gauge::default()
            .block(Block::default().title(Span::styled(" ● 启用率 ", Style::default().fg(GREEN).bold())).style(Style::default().bg(BG)))
            .gauge_style(Style::default().fg(GREEN).bg(SURFACE))
            .ratio(enabled_ratio)
            .label(Span::styled(format!("{enabled}/{total}"), Style::default().fg(TEXT).bold())),
        gauge_chunks[0],
    );

    let github_ratio = if total > 0 { github_count as f64 / total as f64 } else { 0.0 };
    f.render_widget(
        Gauge::default()
            .block(Block::default().title(Span::styled(" ⎇ GitHub ", Style::default().fg(SAPPHIRE).bold())).style(Style::default().bg(BG)))
            .gauge_style(Style::default().fg(SAPPHIRE).bg(SURFACE))
            .ratio(github_ratio)
            .label(Span::styled(format!("{github_count}/{total}"), Style::default().fg(TEXT).bold())),
        gauge_chunks[1],
    );

    // Animated sparkline
    let spark_data: Vec<u64> = (0..30)
        .map(|i| {
            let phase = app.tick as f64 * 0.08 + i as f64 * 0.4;
            ((phase.sin() * 3.0 + 5.0).max(0.0) * (1.0 + 0.3 * (phase * 0.3).cos())) as u64
        })
        .collect();
    f.render_widget(
        Sparkline::default()
            .block(Block::default().title(Span::styled(" ~ 活动 ", Style::default().fg(PEACH).bold())).style(Style::default().bg(BG)))
            .data(&spark_data)
            .style(Style::default().fg(PEACH).bg(SURFACE)),
        gauge_chunks[2],
    );

    // ── Shortcuts ──
    let lines = vec![
        Line::from(""),
        decorative_divider(chunks[3].width, app.tick),
        Line::from(vec![
            Span::styled("  ⌨ ", Style::default().fg(MAUVE)),
            Span::styled("快 捷 键", Style::default().fg(TEXT).bold()),
        ]).alignment(Alignment::Center),
        Line::from(""),
        shortcut_row("1 / Tab", "Home", LAVENDER),
        shortcut_row("2", "Skills 管理", SAPPHIRE),
        shortcut_row("3", "Sync 同步", TEAL),
        shortcut_row("q / Ctrl+C", "退出", RED),
    ];
    f.render_widget(
        Paragraph::new(Text::from(lines)).alignment(Alignment::Center).wrap(Wrap { trim: true }),
        chunks[3],
    );
}

/// Draw floating particle field on Home page using Canvas
fn draw_particle_field(f: &mut Frame, app: &App, area: Rect) {
    let particle_colors = [LAVENDER, BLUE, SAPPHIRE, TEAL, MAUVE, PINK, PEACH];

    // Group particles by color for batch rendering
    let mut groups: Vec<(Color, Vec<(f64, f64)>)> = particle_colors
        .iter()
        .map(|&c| (c, Vec::new()))
        .collect();

    for p in &app.particles {
        if p.life > 0.0 && (p.color_idx as usize) < groups.len() {
            groups[p.color_idx as usize].1.push((p.x, p.y));
        }
    }

    let canvas = Canvas::default()
        .marker(ratatui::symbols::Marker::Braille)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            for (color, points) in &groups {
                if !points.is_empty() {
                    ctx.draw(&Points {
                        coords: points,
                        color: *color,
                    });
                }
            }
            // Draw connecting lines between nearby particles (constellation effect)
            let particles = &app.particles;
            let max_dist = 15.0;
            for i in 0..particles.len() {
                for j in (i + 1)..particles.len() {
                    let dx = particles[i].x - particles[j].x;
                    let dy = particles[i].y - particles[j].y;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt();
                    if dist < max_dist {
                        let alpha = 1.0 - dist / max_dist;
                        let min_life = particles[i].life.min(particles[j].life);
                        if alpha > 0.3 && min_life > 0.1 {
                            ctx.draw(&ratatui::widgets::canvas::Line {
                                x1: particles[i].x,
                                y1: particles[i].y,
                                x2: particles[j].x,
                                y2: particles[j].y,
                                color: particle_colors[particles[i].color_idx as usize],
                            });
                        }
                    }
                }
            }
        });

    f.render_widget(
        canvas
            .block(Block::default().style(Style::default().bg(BG)))
            .background_color(BG),
        area,
    );
}

fn shortcut_row(key: &str, desc: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {key:^12} ", key = key), Style::default().fg(BG).bg(color).bold()),
        Span::styled(" → ", Style::default().fg(OVERLAY)),
        Span::styled(desc.to_string(), Style::default().fg(TEXT)),
    ])
}

// ── Skills tab ────────────────────────────────────────────────────

fn draw_skills(f: &mut Frame, app: &App, area: Rect) {
    if app.skill_entries.is_empty() {
        draw_skill_empty(f, app, area);
        return;
    }

    let constraints = if app.show_skill_detail {
        vec![Constraint::Percentage(55), Constraint::Percentage(45)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    draw_skill_table(f, app, chunks[0]);
    if app.show_skill_detail && chunks.len() > 1 {
        draw_skill_detail(f, app, chunks[1]);
    }
}

/// Animated skeleton / empty state for no skills
fn draw_skill_empty(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" ⚡ Skills ", Style::default().fg(SAPPHIRE).bold()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(glow_color(app.tick, SAPPHIRE, 60)))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let frame = (app.tick / 4) as usize % 4;
    let shimmer = [OVERLAY, SURFACE, SUBTEXT, SURFACE][frame];
    let mid = inner.height / 2;
    let w = inner.width as usize;

    // Build centered card with shimmer boxes
    let box_w = 40.min(w.saturating_sub(4));
    let bar = "─".repeat(box_w);
    let pad = " ".repeat((w.saturating_sub(box_w)) / 2);
    let shimmer_short = SKELETON_CHARS[frame].repeat(box_w.saturating_sub(4));

    let lines: Vec<Line> = (0..inner.height)
        .map(|row| {
            if row == mid.saturating_sub(4) {
                // Top border of card
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled(format!("╭{bar}╮"), Style::default().fg(shimmer)),
                ])
            } else if row == mid.saturating_sub(3) {
                // Empty icon area + shimmer
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled(format!("│ {shimmer_short} │"), Style::default().fg(shimmer)),
                ])
            } else if row == mid.saturating_sub(2) {
                // Main message
                let msg = "  还没有任何 Skill";
                let inner_pad = " ".repeat((box_w.saturating_sub(msg.len())) / 2);
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled("│ ", Style::default().fg(shimmer)),
                    Span::styled(format!("{inner_pad}{msg}"), Style::default().fg(TEXT).bold()),
                    Span::styled(" │", Style::default().fg(shimmer)),
                ])
            } else if row == mid.saturating_sub(1) {
                // Hint
                let hint = " 按 a 添加 / g 从 GitHub ";
                let inner_pad = " ".repeat((box_w.saturating_sub(hint.len())) / 2);
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled("│ ", Style::default().fg(shimmer)),
                    Span::styled(format!("{inner_pad}{hint}"), Style::default().fg(SUBTEXT)),
                    Span::styled(" │", Style::default().fg(shimmer)),
                ])
            } else if row == mid {
                // Bottom border of card
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled(format!("╰{bar}╯"), Style::default().fg(shimmer)),
                ])
            } else if (row as i32 - mid as i32).unsigned_abs() <= 8 {
                // Near the card: faint shimmer dots
                Line::from(vec![
                    Span::styled(pad.clone(), Style::default()),
                    Span::styled(format!("│{shimmer_short}│"), Style::default().fg(SURFACE)),
                ])
            } else {
                Line::from("")
            }
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn draw_skill_table(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" ⚡ Skills ({}) ", app.skill_entries.len());

    let header = Row::new(vec![
        Cell::from(Span::styled("#", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("名称", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("状态", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("来源", Style::default().fg(MAUVE).bold())),
    ])
    .style(Style::default().bg(SURFACE))
    .height(1);

    let rows: Vec<Row> = app
        .skill_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.skill_selected;
            let is_even = i % 2 == 0;
            let status_color = if entry.installed.enabled { GREEN } else { RED };
            let status_icon = if entry.installed.enabled { "●" } else { "○" };
            let status_text = if entry.installed.enabled { "启用" } else { "停用" };

            let (source_text, source_style) = if entry.installed.source.starts_with("github:") {
                let repo = entry.installed.source.replacen("github:", "", 1);
                (format!(" ⎇ {repo} "), Style::default().fg(BG).bg(SAPPHIRE))
            } else {
                (format!(" {} ", entry.installed.source), Style::default().fg(SUBTEXT))
            };

            let row_bg = if is_selected { BLUE } else if is_even { SURFACE } else { BG };
            let row_fg = if is_selected { BG } else { TEXT };

            Row::new(vec![
                Cell::from(Span::styled(format!("{}", i + 1), Style::default().fg(if is_selected { BG } else { SUBTEXT }))),
                Cell::from(Span::styled(entry.skill.name.clone(), Style::default().fg(if is_selected { BG } else { LAVENDER }).bold())),
                Cell::from(Line::from(vec![
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::styled(format!(" {status_text}"), Style::default().fg(if is_selected { BG } else { status_color })),
                ])),
                Cell::from(Span::styled(source_text, source_style)),
            ])
            .style(Style::default()
                .fg(row_fg).bg(row_bg)
                .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }))
            .height(1)
        })
        .collect();

    let table = Table::new(rows, vec![
        Constraint::Length(4),
        Constraint::Percentage(40),
        Constraint::Length(10),
        Constraint::Percentage(40),
    ])
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(title, Style::default().fg(SAPPHIRE).bold()))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(glow_color(app.tick, SAPPHIRE, 70)))
            .style(Style::default().bg(BG)),
    )
    .row_highlight_style(Style::default().bg(BLUE).fg(BG));

    f.render_widget(table, area);

    // Animated scrollbar
    if !app.skill_entries.is_empty() {
        let thumb_idx = app.tick as usize % 5;
        let thumb_colors = [LAVENDER, MAUVE, BLUE, SAPPHIRE, PINK];
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(thumb_colors[thumb_idx]))
            .track_style(Style::default().fg(OVERLAY))
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut state = ScrollbarState::new(app.skill_entries.len()).position(app.skill_selected);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin { horizontal: 0, vertical: 1 }),
            &mut state,
        );
    }

    // Help footer with key badges
    let help_h = 2;
    if area.height > help_h + 4 {
        let help_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - help_h - 1,
            width: area.width - 2,
            height: help_h,
        };
        let help = Paragraph::new(Text::from(vec![
            Line::from(vec![
                key_badge("a", GREEN), Span::styled(" 添加 ", Style::default().fg(SUBTEXT)),
                key_badge("g", SAPPHIRE), Span::styled(" GitHub ", Style::default().fg(SUBTEXT)),
                key_badge("e", TEAL), Span::styled(" 启用 ", Style::default().fg(SUBTEXT)),
                key_badge("d", PEACH), Span::styled(" 停用 ", Style::default().fg(SUBTEXT)),
                key_badge("x", RED), Span::styled(" 删除 ", Style::default().fg(SUBTEXT)),
            ]),
            Line::from(vec![
                key_badge("v", YELLOW), Span::styled(" 查看 ", Style::default().fg(SUBTEXT)),
                key_badge("r", LAVENDER), Span::styled(" 刷新 ", Style::default().fg(SUBTEXT)),
                key_badge("R", LAVENDER), Span::styled(" 重扫描 ", Style::default().fg(SUBTEXT)),
                key_badge("D", PINK), Span::styled(" Doctor ", Style::default().fg(SUBTEXT)),
                key_badge("Spc", MAUVE), Span::styled(" 详情", Style::default().fg(SUBTEXT)),
            ]),
        ]))
        .style(Style::default().bg(BG));
        f.render_widget(help, help_area);
    }
}

fn key_badge(key: &str, color: Color) -> Span<'static> {
    Span::styled(format!(" {key} "), Style::default().fg(BG).bg(color).bold())
}

fn draw_skill_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" ◈ Detail ", Style::default().fg(TEAL).bold()))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(glow_color(app.tick, TEAL, 90)))
        .style(Style::default().bg(BG));

    let text = if let Some(entry) = app.selected_skill() {
        let status_color = if entry.installed.enabled { GREEN } else { RED };
        let status_icon = if entry.installed.enabled { "●" } else { "○" };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&entry.skill.name, Style::default().fg(LAVENDER).bold().add_modifier(Modifier::ITALIC)),
            ]),
            // Health bar line
            Line::from(vec![
                Span::styled("  ", Style::default()),
                if entry.installed.enabled {
                    // Animated enabled bar
                    let pulse = (app.tick as f64 * 0.05).sin() * 0.5 + 0.5;
                    let filled = (pulse * 12.0) as usize + 4;
                    let bar_full = "█".repeat(filled.min(20));
                    let bar_empty = "░".repeat(20_usize.saturating_sub(filled.min(20)));
                    Span::styled(format!("{bar_full}{bar_empty}"), Style::default().fg(GREEN))
                } else {
                    Span::styled("░░░░░░░░░░░░░░░░░░░░", Style::default().fg(RED))
                },
                Span::styled(format!(" {}", if entry.installed.enabled { "● ACTIVE" } else { "○ INACTIVE" }), Style::default().fg(status_color).bold()),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&entry.skill.id, Style::default().fg(SUBTEXT)),
            ]),
            Line::from(""),
            section_header("实例", SAPPHIRE),
            field_row("ID", &entry.installed.instance_id, SUBTEXT),
            field_row("目录", &entry.installed.dir_name, SUBTEXT),
            field_row("路径", &entry.installed.install_path, SUBTEXT),
            Line::from(vec![
                Span::styled("  来源 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
                if entry.installed.source.starts_with("github:") {
                    Span::styled(
                        format!(" ⎇ {} ", entry.installed.source.replacen("github:", "", 1)),
                        Style::default().fg(BG).bg(SAPPHIRE),
                    )
                } else {
                    Span::styled(format!(" {} ", entry.installed.source), Style::default().fg(TEXT))
                },
            ]),
            Line::from(vec![
                Span::styled("  状态 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
                Span::styled(status_icon, Style::default().fg(status_color)),
                Span::styled(
                    format!(" {}", if entry.installed.enabled { "已启用" } else { "已停用" }),
                    Style::default().fg(status_color).bold(),
                ),
            ]),
            field_row("创建", &entry.installed.created_at, SUBTEXT),
            field_row("更新", &entry.installed.updated_at, SUBTEXT),
        ];

        if !entry.skill.description.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header("描述", PEACH));
            lines.push(Line::from(format!("    {}", entry.skill.description)));
        }

        if !entry.skill.domains.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  领域 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
            ]));
            let domains: Vec<Span> = entry.skill.domains.iter().flat_map(|d| {
                vec![Span::styled(format!(" {d} "), Style::default().fg(BG).bg(MAUVE)), Span::raw(" ")]
            }).collect();
            lines.push(Line::from(domains));
        }

        if !entry.skill.modules.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  模块 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
            ]));
            let modules: Vec<Span> = entry.skill.modules.iter().flat_map(|m| {
                vec![Span::styled(format!(" {m} "), Style::default().fg(BG).bg(TEAL)), Span::raw(" ")]
            }).collect();
            lines.push(Line::from(modules));
        }

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("  未选择 skill", Style::default().fg(SUBTEXT).italic())),
        ])
    };

    f.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: true }), area);
}


fn field_row<'a>(label: &str, value: &'a str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {label} "), Style::default().fg(YELLOW)),
        Span::styled("│ ", Style::default().fg(OVERLAY)),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

fn section_header(label: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ╭", Style::default().fg(OVERLAY)),
        Span::styled("─".repeat(2), Style::default().fg(OVERLAY)),
        Span::styled(label.to_string(), Style::default().fg(color).bold()),
        Span::styled("─".repeat(8), Style::default().fg(OVERLAY)),
        Span::styled("╮", Style::default().fg(OVERLAY)),
    ])
}

fn draw_sync(f: &mut Frame, app: &App, area: Rect) {
    let pulse = (app.tick as f64 * 0.04).sin() * 0.5 + 0.5;
    let conn_color = if pulse > 0.5 { TEAL } else { SAPPHIRE };
    let conn_icon = if app.sync_config.is_some() { "◉" } else { "○" };
    let block = Block::default()
        .title(Span::styled(
            format!(" {conn_icon} Sync "),
            Style::default().fg(conn_color).bold(),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(glow_color(app.tick, TEAL, 75)))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = if let Some(remote) = &app.sync_config {
        let conn_status = if app.sync_config.is_some() {
            let dot_phase = (app.tick / 3) as usize % 4;
            let dots = ".".repeat(dot_phase);
            format!("已连接{dots}")
        } else {
            "未连接".to_string()
        };
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ● ", Style::default().fg(GREEN)),
                Span::styled(conn_status, Style::default().fg(GREEN)),
            ]).alignment(Alignment::Center),
            Line::from(Span::styled("─── 同步配置 ───", Style::default().fg(MAUVE).bold())).alignment(Alignment::Center),
        ];

        match remote {
            byi_storage::RemoteConfig::GitHub(config) => {
                lines.push(Line::from(""));
                lines.push(sync_field("类型", "GitHub", LAVENDER));
                lines.push(sync_field("仓库", &config.repo, TEXT));
                lines.push(sync_field("分支", &config.branch, TEXT));
                lines.push(sync_field("基础路径", &config.base_path, SUBTEXT));
                lines.push(sync_field("认证", &config.auth, SUBTEXT));
            }
            byi_storage::RemoteConfig::WebDav(config) => {
                lines.push(Line::from(""));
                lines.push(sync_field("类型", "WebDAV", LAVENDER));
                lines.push(sync_field("端点", &config.endpoint_url, TEXT));
                let preset = match config.preset {
                    byi_webdav::WebDavPreset::Jianguoyun => "坚果云",
                    byi_webdav::WebDavPreset::Custom => "自定义",
                };
                lines.push(sync_field("预设", preset, SAPPHIRE));
                lines.push(sync_field("基础路径", &config.base_path, SUBTEXT));
                let username = config.username.clone().unwrap_or_default();
                lines.push(sync_field("用户名", &username, SUBTEXT));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("─── 快捷键 ───", Style::default().fg(MAUVE))).alignment(Alignment::Center));
        lines.push(Line::from(""));
        lines.push(shortcut_row("t", "测试连通性", TEAL));
        lines.push(shortcut_row("p", "从远端拉取 (pull)", SAPPHIRE));
        lines.push(shortcut_row("P", "推送到远端 (push)", LAVENDER));
        lines.push(shortcut_row("r", "刷新配置", GREEN));

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("未配置同步远端", Style::default().fg(SUBTEXT).italic())).alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled("请使用命令行配置:", Style::default().fg(TEXT))).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  byi sync init ", Style::default().fg(SAPPHIRE).bold()),
                Span::styled("--provider github --repo owner/repo", Style::default().fg(SUBTEXT)),
            ]).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  byi sync init ", Style::default().fg(TEAL).bold()),
                Span::styled("--provider webdav --preset jianguoyun", Style::default().fg(SUBTEXT)),
            ]).alignment(Alignment::Center),
        ])
    };

    f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), inner);
}

fn sync_field(label: &str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {label} "), Style::default().fg(YELLOW)),
        Span::styled("│ ", Style::default().fg(OVERLAY)),
        Span::styled(value.to_string(), Style::default().fg(color)),
    ])
}

// ── Popups ────────────────────────────────────────────────────────

fn draw_popup(f: &mut Frame, app: &App, popup: &Popup) {
    let area = centered_rect(60, 40, f.area());

    // Shadow
    let shadow = Rect { x: area.x + 1, y: area.y + 1, width: area.width, height: area.height };
    f.render_widget(Clear, shadow);
    f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(10, 10, 15))), shadow);
    f.render_widget(Clear, area);

    match popup {
        Popup::Message { title, body } => {
            let block = Block::default()
                .title(Span::styled(format!(" ◈ {title} "), Style::default().fg(MAUVE).bold()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(LAVENDER))
                .style(Style::default().bg(SURFACE));

            let mut body_lines: Vec<Line> = body.lines()
                .map(|l| Line::from(Span::styled(l, Style::default().fg(TEXT))))
                .collect();
            body_lines.push(Line::from(""));
            body_lines.push(Line::from(vec![
                key_badge("Enter", LAVENDER),
                Span::styled(" 关闭", Style::default().fg(SUBTEXT)),
            ]).alignment(Alignment::Center));

            f.render_widget(Paragraph::new(Text::from(body_lines)).block(block).wrap(Wrap { trim: true }), area);
        }

        Popup::Input { title, value, action: _ } => {
            let block = Block::default()
                .title(Span::styled(format!(" → {title} "), Style::default().fg(SAPPHIRE).bold()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(SAPPHIRE))
                .style(Style::default().bg(SURFACE));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let input_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            let cursor_on = (app.tick / 4) % 2 == 0;
            let input_line = if value.is_empty() {
                vec![
                    Span::styled(" ", Style::default()),
                    if cursor_on { Span::styled("▎", Style::default().fg(SAPPHIRE)) } else { Span::styled(" ", Style::default()) },
                    Span::styled(" 输入路径...", Style::default().fg(OVERLAY)),
                ]
            } else {
                vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(value.clone(), Style::default().fg(TEXT).bold()),
                    if cursor_on { Span::styled("▎", Style::default().fg(SAPPHIRE)) } else { Span::styled(" ", Style::default().fg(SAPPHIRE)) },
                ]
            };
            let input_bg = Block::default().style(Style::default().bg(BG));
            let input_inner = input_bg.inner(input_chunks[1]);
            f.render_widget(input_bg, input_chunks[1]);
            f.render_widget(Paragraph::new(Line::from(input_line)), input_inner);

            let hint = Line::from(vec![
                key_badge("Enter", GREEN), Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                key_badge("Esc", RED), Span::styled(" 取消", Style::default().fg(SUBTEXT)),
            ]).alignment(Alignment::Center);
            f.render_widget(Paragraph::new(hint).style(Style::default().bg(SURFACE)), input_chunks[2]);
        }

        Popup::Confirm { title, body, action: _ } => {
            let block = Block::default()
                .title(Span::styled(format!(" ⚠ {title} "), Style::default().fg(PEACH).bold()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PEACH))
                .style(Style::default().bg(SURFACE));

            let mut lines: Vec<Line> = body.lines()
                .map(|l| Line::from(Span::styled(l, Style::default().fg(TEXT))))
                .collect();
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                key_badge("Y/Enter", RED), Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                key_badge("其他键", OVERLAY), Span::styled(" 取消", Style::default().fg(SUBTEXT)),
            ]).alignment(Alignment::Center));

            f.render_widget(Paragraph::new(Text::from(lines)).block(block).wrap(Wrap { trim: true }), area);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
