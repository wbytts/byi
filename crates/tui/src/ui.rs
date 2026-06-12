use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Sparkline, Table, Tabs, Wrap,
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

// ── Sparkle frames for Home animation ─────────────────────────────
const SPARKLES: &[&str] = &["✦", "✧", "⋆", "★", "✶", "✴", "✳", "✺"];

// ── Throbber frames for status spinner ────────────────────────────
const THROBBER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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

// ── Status bar with animated spinner ──────────────────────────────

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    if app.status_message.is_empty() {
        let spinner_frame = THROBBER[app.tick as usize % THROBBER.len()];
        let keys = vec![
            Span::styled(format!(" {spinner_frame} "), Style::default().fg(MAUVE)),
            Span::styled(" q ", Style::default().bg(RED).fg(BG).bold()),
            Span::styled(" 退出 ", Style::default().fg(SUBTEXT)),
            Span::styled(" 1/2/3 ", Style::default().bg(BLUE).fg(BG).bold()),
            Span::styled(" 切换标签 ", Style::default().fg(SUBTEXT)),
        ];
        f.render_widget(
            Paragraph::new(Line::from(keys)).style(Style::default().bg(BG)),
            area,
        );
    } else {
        let spinner_frame = THROBBER[app.tick as usize % THROBBER.len()];
        let fade_alpha = if app.status_ttl < 10 { app.status_ttl as u16 * 25 } else { 255 };
        let msg_color = if fade_alpha > 128 { LAVENDER } else { SUBTEXT };
        let line = Line::from(vec![
            Span::styled(format!(" {spinner_frame} "), Style::default().fg(MAUVE)),
            Span::styled(format!("{}", app.status_message), Style::default().fg(msg_color).bold()),
        ]);
        f.render_widget(
            Paragraph::new(line).style(Style::default().bg(BG)),
            area,
        );
    }
}

// ── Tabs with animated underline ──────────────────────────────────

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tab_index = app.current_tab as usize;
    let tab_colors = [LAVENDER, SAPPHIRE, TEAL];
    let tab_icons = ["⌂", "⚡", "⟳"];
    let tab_labels = ["Home", "Skills", "Sync"];

    let titles: Vec<Line> = (0..3)
        .map(|i| {
            if i == tab_index {
                Line::from(Span::styled(
                    format!(" {} {} ", tab_icons[i], tab_labels[i]),
                    Style::default().fg(tab_colors[i]).bold(),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} {} ", tab_icons[i], tab_labels[i]),
                    Style::default().fg(SUBTEXT),
                ))
            }
        })
        .collect();

    // Animated sparkle in logo
    let sparkle_idx = app.tick as usize % SPARKLES.len();
    let sparkle = SPARKLES[sparkle_idx];

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {sparkle} byi "),
                    Style::default().fg(MAUVE).bold(),
                ))
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(OVERLAY))
                .style(Style::default().bg(BG)),
        )
        .select(tab_index)
        .highlight_style(
            Style::default()
                .fg(BG)
                .bg(tab_colors[tab_index])
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(OVERLAY)));
    f.render_widget(tabs, area);
}

// ── Content router ────────────────────────────────────────────────

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Home => draw_home(f, app, area),
        Tab::Skills => draw_skills(f, app, area),
        Tab::Sync => draw_sync(f, app, area),
    }
}

// ── Home tab — animated banner + stats gauges ─────────────────────

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(OVERLAY))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // big text banner
            Constraint::Length(2),  // tagline + sparkle bar
            Constraint::Length(3),  // stats gauges
            Constraint::Min(0),     // shortcuts
        ])
        .split(inner);

    // ── Animated big text banner ──────────────────────────────
    let cycle = (app.tick / 4) as usize % 7;
    let banner_colors = [MAUVE, LAVENDER, BLUE, SAPPHIRE, TEAL, PINK, FLAMINGO];
    let banner_color = banner_colors[cycle];

    let big = BigTextBuilder::default()
        .pixel_size(PixelSize::HalfHeight)
        .lines(vec![
            Line::from(Span::styled("byi", Style::default().fg(banner_color).bold())),
        ])
        .build();
    f.render_widget(big, chunks[0]);

    // ── Animated sparkle divider + tagline ─────────────────────
    let width = chunks[1].width as usize;
    let tagline = "✦ AI Skill Manager ✦";
    let sparkle_idx = app.tick as usize % SPARKLES.len();
    let sparkle = SPARKLES[sparkle_idx];
    let dash_len = (width.saturating_sub(tagline.len() + 4)) / 2;
    let divider = format!(
        "{sparkle}{d:─<dash$}{sparkle} {tagline} {sparkle}{d:─<dash$}{sparkle}",
        d = "",
        dash = dash_len,
        tagline = tagline,
        sparkle = sparkle,
    );
    let tagline_line = Line::from(Span::styled(
        divider,
        Style::default().fg(MAUVE).italic(),
    ));
    f.render_widget(
        Paragraph::new(tagline_line).style(Style::default().bg(BG)),
        chunks[1],
    );

    // ── Stats gauges row ──────────────────────────────────────
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

    // Enabled gauge
    let enabled_ratio = if total > 0 { enabled as f64 / total as f64 } else { 0.0 };
    let enabled_gauge = Gauge::default()
        .block(Block::default()
            .title(Span::styled(" 启用率 ", Style::default().fg(GREEN).bold()))
            .style(Style::default().bg(BG)))
        .gauge_style(Style::default().fg(GREEN).bg(SURFACE))
        .ratio(enabled_ratio)
        .label(Span::styled(
            format!("{enabled}/{total}"),
            Style::default().fg(TEXT).bold(),
        ));
    f.render_widget(enabled_gauge, gauge_chunks[0]);

    // GitHub gauge
    let github_ratio = if total > 0 { github_count as f64 / total as f64 } else { 0.0 };
    let github_gauge = Gauge::default()
        .block(Block::default()
            .title(Span::styled(" GitHub ", Style::default().fg(SAPPHIRE).bold()))
            .style(Style::default().bg(BG)))
        .gauge_style(Style::default().fg(SAPPHIRE).bg(SURFACE))
        .ratio(github_ratio)
        .label(Span::styled(
            format!("{github_count}/{total}"),
            Style::default().fg(TEXT).bold(),
        ));
    f.render_widget(github_gauge, gauge_chunks[1]);

    // Sparkline — fake activity visualization using tick
    let spark_data: Vec<u64> = (0..20)
        .map(|i| {
            let phase = app.tick as f64 + i as f64 * 0.7;
            ((phase.sin() * 3.0 + 5.0).max(0.0)) as u64
        })
        .collect();
    let sparkline = Sparkline::default()
        .block(Block::default()
            .title(Span::styled(" 活动 ", Style::default().fg(PEACH).bold()))
            .style(Style::default().bg(BG)))
        .data(&spark_data)
        .style(Style::default().fg(PEACH).bg(SURFACE));
    f.render_widget(sparkline, gauge_chunks[2]);

    // ── Shortcuts ─────────────────────────────────────────────
    let shortcut_rows = vec![
        shortcut_row("1 / Tab", "Home", LAVENDER),
        shortcut_row("2", "Skills 管理", SAPPHIRE),
        shortcut_row("3", "Sync 同步", TEAL),
        shortcut_row("q / Ctrl+C", "退出", RED),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "─── 快捷键 ───",
            Style::default().fg(OVERLAY),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
    ];
    for row in shortcut_rows {
        lines.push(row);
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(&app.hello_message)));

    let para = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(para, chunks[3]);
}

fn shortcut_row(key: &str, desc: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {key:^12} ", key = key),
            Style::default().fg(BG).bg(color).bold(),
        ),
        Span::styled(" → ", Style::default().fg(OVERLAY)),
        Span::styled(format!("{desc}"), Style::default().fg(TEXT)),
    ])
}

// ── Skills tab ────────────────────────────────────────────────────

fn draw_skills(f: &mut Frame, app: &App, area: Rect) {
    let constraints = if app.show_skill_detail && !app.skill_entries.is_empty() {
        vec![Constraint::Percentage(55), Constraint::Percentage(45)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    draw_skill_table(f, app, chunks[0]);
    if app.show_skill_detail && !app.skill_entries.is_empty() && chunks.len() > 1 {
        draw_skill_detail(f, app, chunks[1]);
    }
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

            let status_icon = if entry.installed.enabled { "●" } else { "○" };
            let status_text = if entry.installed.enabled { "启用" } else { "停用" };
            let status_color = if entry.installed.enabled { GREEN } else { RED };

            // Source badge styling
            let (source_text, source_style) = if entry.installed.source.starts_with("github:") {
                let repo = entry.installed.source.replacen("github:", "", 1);
                (repo, Style::default().fg(BG).bg(SAPPHIRE))
            } else {
                (entry.installed.source.clone(), Style::default().fg(SUBTEXT))
            };

            let row_bg = if is_selected {
                BLUE
            } else if is_even {
                SURFACE
            } else {
                BG
            };
            let row_fg = if is_selected { BG } else { TEXT };

            Row::new(vec![
                Cell::from(Span::styled(
                    format!("{}", i + 1),
                    Style::default().fg(if is_selected { BG } else { SUBTEXT }),
                )),
                Cell::from(Span::styled(
                    entry.skill.name.clone(),
                    Style::default().fg(if is_selected { BG } else { LAVENDER }).bold(),
                )),
                Cell::from(Line::from(vec![
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::styled(format!(" {status_text}"), Style::default().fg(if is_selected { BG } else { status_color })),
                ])),
                Cell::from(Span::styled(format!(" {source_text} "), source_style)),
            ])
            .style(
                Style::default()
                    .fg(row_fg)
                    .bg(row_bg)
                    .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            )
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
            .border_style(Style::default().fg(OVERLAY))
            .style(Style::default().bg(BG)),
    )
    .row_highlight_style(Style::default().bg(BLUE).fg(BG));

    f.render_widget(table, area);

    // Animated scrollbar
    if !app.skill_entries.is_empty() {
        let thumb_color_idx = app.tick as usize % 5;
        let thumb_colors = [LAVENDER, MAUVE, BLUE, SAPPHIRE, PINK];
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(thumb_colors[thumb_color_idx]))
            .track_style(Style::default().fg(OVERLAY))
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut state = ScrollbarState::new(app.skill_entries.len())
            .position(app.skill_selected);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin { horizontal: 0, vertical: 1 }),
            &mut state,
        );
    }

    // Help footer with keyboard badge style
    let help_height = 2;
    if area.height > help_height + 4 {
        let help_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - help_height - 1,
            width: area.width - 2,
            height: help_height,
        };
        let help = Paragraph::new(Text::from(vec![
            Line::from(vec![
                key_badge("a", GREEN),
                Span::styled(" 添加 ", Style::default().fg(SUBTEXT)),
                key_badge("g", SAPPHIRE),
                Span::styled(" GitHub ", Style::default().fg(SUBTEXT)),
                key_badge("e", TEAL),
                Span::styled(" 启用 ", Style::default().fg(SUBTEXT)),
                key_badge("d", PEACH),
                Span::styled(" 停用 ", Style::default().fg(SUBTEXT)),
                key_badge("x", RED),
                Span::styled(" 删除 ", Style::default().fg(SUBTEXT)),
            ]),
            Line::from(vec![
                key_badge("v", YELLOW),
                Span::styled(" 查看 ", Style::default().fg(SUBTEXT)),
                key_badge("r", LAVENDER),
                Span::styled(" 刷新 ", Style::default().fg(SUBTEXT)),
                key_badge("R", LAVENDER),
                Span::styled(" 重扫描 ", Style::default().fg(SUBTEXT)),
                key_badge("D", PINK),
                Span::styled(" Doctor ", Style::default().fg(SUBTEXT)),
                key_badge("Spc", MAUVE),
                Span::styled(" 详情", Style::default().fg(SUBTEXT)),
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
        .border_style(Style::default().fg(OVERLAY))
        .style(Style::default().bg(BG));

    let text = if let Some(entry) = app.selected_skill() {
        let status_color = if entry.installed.enabled { GREEN } else { RED };
        let status_icon = if entry.installed.enabled { "●" } else { "○" };
        let status_text = if entry.installed.enabled { "已启用" } else { "已停用" };

        let mut lines = vec![
            Line::from(""),
            // Name header
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&entry.skill.name, Style::default().fg(LAVENDER).bold().add_modifier(Modifier::ITALIC)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&entry.skill.id, Style::default().fg(SUBTEXT)),
            ]),
            Line::from(""),
            // Instance section
            section_header("实例", SAPPHIRE),
            field_row("ID", &entry.installed.instance_id, SUBTEXT),
            field_row("目录", &entry.installed.dir_name, SUBTEXT),
            field_row("路径", &entry.installed.install_path, SUBTEXT),
            // Source badge
            Line::from(vec![
                Span::styled("  来源 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
                if entry.installed.source.starts_with("github:") {
                    Span::styled(
                        format!(" {} ", entry.installed.source.replacen("github:", "⎇ ", 1)),
                        Style::default().fg(BG).bg(SAPPHIRE),
                    )
                } else {
                    Span::styled(
                        format!(" {} ", entry.installed.source),
                        Style::default().fg(TEXT),
                    )
                },
            ]),
            // Status
            Line::from(vec![
                Span::styled("  状态 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
                Span::styled(status_icon, Style::default().fg(status_color)),
                Span::styled(format!(" {status_text}"), Style::default().fg(status_color).bold()),
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
                vec![
                    Span::styled(format!(" {d} "), Style::default().fg(BG).bg(MAUVE)),
                    Span::raw(" "),
                ]
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
                vec![
                    Span::styled(format!(" {m} "), Style::default().fg(BG).bg(TEAL)),
                    Span::raw(" "),
                ]
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

    let para = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn section_header(label: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ── ", Style::default().fg(OVERLAY)),
        Span::styled(label.to_string(), Style::default().fg(color).bold()),
        Span::styled(" ──", Style::default().fg(OVERLAY)),
    ])
}

fn field_row<'a>(label: &str, value: &'a str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {label} "), Style::default().fg(YELLOW)),
        Span::styled("│ ", Style::default().fg(OVERLAY)),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

// ── Sync tab ──────────────────────────────────────────────────────

fn draw_sync(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" ⟳ Sync ", Style::default().fg(TEAL).bold()))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(OVERLAY))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = if let Some(remote) = &app.sync_config {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "─── 同步配置 ───",
                Style::default().fg(MAUVE).bold(),
            ))
            .alignment(Alignment::Center),
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
        lines.push(
            Line::from(Span::styled(
                "─── 快捷键 ───",
                Style::default().fg(MAUVE),
            ))
            .alignment(Alignment::Center),
        );
        lines.push(Line::from(""));
        lines.push(shortcut_row("t", "测试连通性", TEAL));
        lines.push(shortcut_row("p", "从远端拉取 (pull)", SAPPHIRE));
        lines.push(shortcut_row("P", "推送到远端 (push)", LAVENDER));
        lines.push(shortcut_row("r", "刷新配置", GREEN));

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "未配置同步远端",
                Style::default().fg(SUBTEXT).italic(),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(
                "请使用命令行配置:",
                Style::default().fg(TEXT),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  byi sync init ", Style::default().fg(SAPPHIRE).bold()),
                Span::styled("--provider github --repo owner/repo", Style::default().fg(SUBTEXT)),
            ])
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  byi sync init ", Style::default().fg(TEAL).bold()),
                Span::styled("--provider webdav --preset jianguoyun", Style::default().fg(SUBTEXT)),
            ])
            .alignment(Alignment::Center),
        ])
    };

    let para = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(para, inner);
}

fn sync_field(label: &str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {label} "), Style::default().fg(YELLOW)),
        Span::styled("│ ", Style::default().fg(OVERLAY)),
        Span::styled(value.to_string(), Style::default().fg(color)),
    ])
}

// ── Popups with shadow and animated cursor ────────────────────────

fn draw_popup(f: &mut Frame, _app: &App, popup: &Popup) {
    let area = centered_rect(60, 40, f.area());

    // Shadow effect — render dark block offset by (1,1)
    let shadow_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width,
        height: area.height,
    };
    f.render_widget(Clear, shadow_area);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(10, 10, 15))),
        shadow_area,
    );

    // Clear popup area
    f.render_widget(Clear, area);

    match popup {
        Popup::Message { title, body } => {
            let block = Block::default()
                .title(Span::styled(
                    format!(" ◈ {title} "),
                    Style::default().fg(MAUVE).bold(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(LAVENDER))
                .style(Style::default().bg(SURFACE));

            let mut body_lines: Vec<Line> =
                body.lines().map(|l| Line::from(Span::styled(l, Style::default().fg(TEXT)))).collect();
            body_lines.push(Line::from(""));
            body_lines.push(
                Line::from(vec![
                    key_badge("Enter", LAVENDER),
                    Span::styled(" 关闭", Style::default().fg(SUBTEXT)),
                ])
                .alignment(Alignment::Center),
            );

            f.render_widget(
                Paragraph::new(Text::from(body_lines)).block(block).wrap(Wrap { trim: true }),
                area,
            );
        }

        Popup::Input { title, value, action: _ } => {
            let block = Block::default()
                .title(Span::styled(
                    format!(" → {title} "),
                    Style::default().fg(SAPPHIRE).bold(),
                ))
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

            // Animated cursor blink
            let cursor_visible = (_app.tick / 2) % 2 == 0;
            let input_line = if value.is_empty() {
                vec![
                    Span::styled(" ", Style::default()),
                    if cursor_visible {
                        Span::styled("▎", Style::default().fg(SAPPHIRE))
                    } else {
                        Span::styled(" ", Style::default())
                    },
                    Span::styled(" 输入路径...", Style::default().fg(OVERLAY)),
                ]
            } else {
                vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(value.clone(), Style::default().fg(TEXT).bold()),
                    if cursor_visible {
                        Span::styled("▎", Style::default().fg(SAPPHIRE))
                    } else {
                        Span::styled(" ", Style::default().fg(SAPPHIRE))
                    },
                ]
            };
            // Input field background
            let input_bg = Block::default().style(Style::default().bg(BG));
            let input_inner = input_bg.inner(input_chunks[1]);
            f.render_widget(input_bg, input_chunks[1]);
            f.render_widget(
                Paragraph::new(Line::from(input_line)),
                input_inner,
            );

            // Footer hint
            let hint = Line::from(vec![
                key_badge("Enter", GREEN),
                Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                key_badge("Esc", RED),
                Span::styled(" 取消", Style::default().fg(SUBTEXT)),
            ])
            .alignment(Alignment::Center);
            f.render_widget(
                Paragraph::new(hint).style(Style::default().bg(SURFACE)),
                input_chunks[2],
            );
        }

        Popup::Confirm { title, body, action: _ } => {
            let block = Block::default()
                .title(Span::styled(
                    format!(" ⚠ {title} "),
                    Style::default().fg(PEACH).bold(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PEACH))
                .style(Style::default().bg(SURFACE));

            let mut lines: Vec<Line> = body
                .lines()
                .map(|l| Line::from(Span::styled(l, Style::default().fg(TEXT))))
                .collect();
            lines.push(Line::from(""));
            lines.push(
                Line::from(vec![
                    key_badge("Y/Enter", RED),
                    Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                    key_badge("其他键", OVERLAY),
                    Span::styled(" 取消", Style::default().fg(SUBTEXT)),
                ])
                .alignment(Alignment::Center),
            );

            f.render_widget(
                Paragraph::new(Text::from(lines)).block(block).wrap(Wrap { trim: true }),
                area,
            );
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
