use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Tabs, Wrap,
    },
    Frame,
};
use tui_big_text::{BigTextBuilder, PixelSize};

use crate::app::{App, Popup, Tab};

// ── Dracula-inspired palette ──────────────────────────────────────
const BG: Color = Color::Rgb(24, 24, 37);       // crust
const SURFACE: Color = Color::Rgb(30, 30, 46);  // mantle
const OVERLAY: Color = Color::Rgb(49, 50, 68);  // surface0
const TEXT: Color = Color::Rgb(205, 214, 244);   // text
const SUBTEXT: Color = Color::Rgb(166, 173, 200);// subtext1
const LAVENDER: Color = Color::Rgb(180, 190, 254);// lavender
const BLUE: Color = Color::Rgb(137, 180, 250);   // blue
const SAPPHIRE: Color = Color::Rgb(116, 199, 236);// sapphire
const TEAL: Color = Color::Rgb(148, 226, 213);   // teal
const GREEN: Color = Color::Rgb(166, 227, 161);   // green
const YELLOW: Color = Color::Rgb(249, 226, 175);  // yellow
const PEACH: Color = Color::Rgb(250, 179, 135);   // peach
const RED: Color = Color::Rgb(243, 139, 168);     // red
const PINK: Color = Color::Rgb(245, 194, 231);    // pink
const MAUVE: Color = Color::Rgb(203, 166, 247);   // mauve

// ── Main draw ─────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    // Fill entire background
    f.render_widget(
        Block::default().style(Style::default().bg(BG)),
        f.area(),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // tabs
            Constraint::Min(0),     // content
            Constraint::Length(1),  // status
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
    let msg = if app.status_message.is_empty() {
        let keys = vec![
            Span::styled(" q ", Style::default().bg(RED).fg(BG).bold()),
            Span::styled(" 退出 ", Style::default().fg(SUBTEXT)),
            Span::styled(" 1/2/3 ", Style::default().bg(BLUE).fg(BG).bold()),
            Span::styled(" 切换标签 ", Style::default().fg(SUBTEXT)),
        ];
        let line = Line::from(keys);
        let para = Paragraph::new(line).style(Style::default().bg(BG));
        f.render_widget(para, area);
        return;
    } else {
        format!(" {}", app.status_message)
    };
    let para = Paragraph::new(msg)
        .style(Style::default().fg(LAVENDER).bg(BG))
        .add_modifier(Modifier::BOLD);
    f.render_widget(para, area);
}

// ── Tabs ──────────────────────────────────────────────────────────

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = vec![
        Line::from(Span::styled(
            " ⌂ Home ",
            Style::default().fg(TEXT),
        )),
        Line::from(Span::styled(
            " ⚡ Skills ",
            Style::default().fg(TEXT),
        )),
        Line::from(Span::styled(
            " ⟳ Sync ",
            Style::default().fg(TEXT),
        )),
    ];

    let tab_index = app.current_tab as usize;
    let highlight_colors = [LAVENDER, SAPPHIRE, TEAL];
    let highlight = highlight_colors[tab_index];

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(
                    " ✦ byi ",
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
                .bg(highlight)
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

// ── Home tab ──────────────────────────────────────────────────────

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let block = block_default("");

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into banner top and shortcuts bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // big text banner
            Constraint::Min(0),     // shortcuts
        ])
        .split(inner);

    // Big ASCII banner
    let big = BigTextBuilder::default()
        .pixel_size(PixelSize::HalfHeight)
        .lines(vec![
            Line::from(Span::styled("byi", Style::default().fg(MAUVE).bold())),
        ])
        .build();
    f.render_widget(big, chunks[0]);

    // Tagline + shortcuts
    let tagline = Line::from(Span::styled(
        "✦ AI Skill Manager ✦",
        Style::default().fg(LAVENDER).italic(),
    ));

    let shortcut_rows = vec![
        shortcut_row("1 / Tab", "Home", LAVENDER),
        shortcut_row("2", "Skills 管理", SAPPHIRE),
        shortcut_row("3", "Sync 同步", TEAL),
        shortcut_row("q / Ctrl+C", "退出", RED),
    ];

    let mut lines = vec![
        Line::from(""),
        tagline,
        Line::from(""),
        Line::from(Span::styled(
            "─── 快捷键 ───",
            Style::default().fg(OVERLAY),
        )),
        Line::from(""),
    ];
    for row in shortcut_rows {
        lines.push(row);
    }
    lines.push(Line::from(""));
    lines.push(Line::from(app.hello_message.clone()));

    let para = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(para, chunks[1]);
}

fn shortcut_row(key: &str, desc: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {key:^12} ", key = key), Style::default().fg(color).bold()),
        Span::styled("→", Style::default().fg(OVERLAY)),
        Span::styled(format!("  {desc}"), Style::default().fg(TEXT)),
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
    let title = format!(
        " ⚡ Skills ({}) ",
        app.skill_entries.len()
    );

    let header = Row::new(vec![
        Cell::from(Span::styled("#", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("名称", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("状态", Style::default().fg(MAUVE).bold())),
        Cell::from(Span::styled("来源", Style::default().fg(MAUVE).bold())),
    ])
    .style(Style::default().bg(SURFACE))
    .height(1)
    .bottom_margin(0);

    let rows: Vec<Row> = app
        .skill_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.skill_selected;
            let is_even = i % 2 == 0;

            let status = if entry.installed.enabled {
                Span::styled("● 启用", Style::default().fg(GREEN))
            } else {
                Span::styled("○ 停用", Style::default().fg(RED))
            };

            let source = if entry.installed.source.starts_with("github:") {
                entry.installed.source.replacen("github:", "", 1)
            } else {
                entry.installed.source.clone()
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
                Cell::from((i + 1).to_string()),
                Cell::from(Span::styled(
                    entry.skill.name.clone(),
                    Style::default().fg(if is_selected { BG } else { LAVENDER }),
                )),
                Cell::from(Line::from(vec![Span::styled(status.content.clone(), Style::default().fg(if is_selected { BG } else if entry.installed.enabled { GREEN } else { RED }))])),
                Cell::from(source),
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

    // Scrollbar
    if !app.skill_entries.is_empty() {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(LAVENDER))
            .track_style(Style::default().fg(OVERLAY))
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut state = ScrollbarState::new(app.skill_entries.len())
            .position(app.skill_selected);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }

    // Help footer at bottom of table area
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
                Span::styled(" a", Style::default().fg(GREEN).bold()),
                Span::styled(" 添加本地 ", Style::default().fg(SUBTEXT)),
                Span::styled(" g", Style::default().fg(SAPPHIRE).bold()),
                Span::styled(" GitHub ", Style::default().fg(SUBTEXT)),
                Span::styled(" e", Style::default().fg(TEAL).bold()),
                Span::styled(" 启用 ", Style::default().fg(SUBTEXT)),
                Span::styled(" d", Style::default().fg(PEACH).bold()),
                Span::styled(" 停用 ", Style::default().fg(SUBTEXT)),
                Span::styled(" x", Style::default().fg(RED).bold()),
                Span::styled(" 删除 ", Style::default().fg(SUBTEXT)),
            ]),
            Line::from(vec![
                Span::styled(" v", Style::default().fg(YELLOW).bold()),
                Span::styled(" 查看 ", Style::default().fg(SUBTEXT)),
                Span::styled(" r", Style::default().fg(LAVENDER).bold()),
                Span::styled(" 刷新 ", Style::default().fg(SUBTEXT)),
                Span::styled(" R", Style::default().fg(LAVENDER).bold()),
                Span::styled(" 重扫描 ", Style::default().fg(SUBTEXT)),
                Span::styled(" D", Style::default().fg(PINK).bold()),
                Span::styled(" Doctor ", Style::default().fg(SUBTEXT)),
                Span::styled(" Space", Style::default().fg(MAUVE).bold()),
                Span::styled(" 详情", Style::default().fg(SUBTEXT)),
            ]),
        ]))
        .style(Style::default().bg(BG));
        f.render_widget(help, help_area);
    }
}

fn draw_skill_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " ◈ Detail ",
            Style::default().fg(TEAL).bold(),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(OVERLAY))
        .style(Style::default().bg(BG));

    let text = if let Some(entry) = app.selected_skill() {
        let status_color = if entry.installed.enabled { GREEN } else { RED };
        let status_text = if entry.installed.enabled { "● 已启用" } else { "○ 已停用" };

        let mut lines = vec![
            Line::from(""),
            field_row("名称", &entry.skill.name, LAVENDER),
            field_row("ID", &entry.skill.id, SUBTEXT),
            Line::from(vec![
                Span::styled("  ── ", Style::default().fg(OVERLAY)),
                Span::styled("实例", Style::default().fg(SAPPHIRE).bold()),
                Span::styled(" ──", Style::default().fg(OVERLAY)),
            ]),
            field_row("ID", &entry.installed.instance_id, SUBTEXT),
            field_row("目录", &entry.installed.dir_name, SUBTEXT),
            field_row("路径", &entry.installed.install_path, SUBTEXT),
            field_row("来源", &entry.installed.source, SUBTEXT),
            Line::from(vec![
                Span::styled("  状态 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
                Span::styled(status_text, Style::default().fg(status_color).bold()),
            ]),
            field_row("创建", &entry.installed.created_at, SUBTEXT),
            field_row("更新", &entry.installed.updated_at, SUBTEXT),
        ];

        if !entry.skill.description.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  ── ", Style::default().fg(OVERLAY)),
                Span::styled("描述", Style::default().fg(PEACH).bold()),
                Span::styled(" ──", Style::default().fg(OVERLAY)),
            ]));
            lines.push(Line::from(format!("    {}", entry.skill.description)));
        }

        if !entry.skill.domains.is_empty() {
            let domains: Vec<Span> = entry
                .skill
                .domains
                .iter()
                .flat_map(|d| {
                    vec![
                        Span::styled(format!(" {d} "), Style::default().fg(MAUVE).bg(SURFACE)),
                        Span::raw(" "),
                    ]
                })
                .collect();
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  领域 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
            ]));
            lines.push(Line::from(domains));
        }

        if !entry.skill.modules.is_empty() {
            let modules: Vec<Span> = entry
                .skill
                .modules
                .iter()
                .flat_map(|m| {
                    vec![
                        Span::styled(format!(" {m} "), Style::default().fg(TEAL).bg(SURFACE)),
                        Span::raw(" "),
                    ]
                })
                .collect();
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  模块 ", Style::default().fg(YELLOW)),
                Span::styled("│ ", Style::default().fg(OVERLAY)),
            ]));
            lines.push(Line::from(modules));
        }

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  未选择 skill",
                Style::default().fg(SUBTEXT).italic(),
            )),
        ])
    };

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
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
        .title(Span::styled(
            " ⟳ Sync ",
            Style::default().fg(TEAL).bold(),
        ))
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
                lines.push(sync_field(
                    "用户名",
                    &username,
                    SUBTEXT,
                ));
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

// ── Popups ────────────────────────────────────────────────────────

fn draw_popup(f: &mut Frame, _app: &App, popup: &Popup) {
    let area = centered_rect(60, 40, f.area());

    // Dim background
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(BG)),
        area,
    );

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

            let footer = Line::from(vec![
                Span::styled(" Enter ", Style::default().bg(LAVENDER).fg(BG).bold()),
                Span::styled(" 关闭 ", Style::default().fg(SUBTEXT)),
            ])
            .alignment(Alignment::Center);

            let mut body_lines: Vec<Line> = body.lines().map(|l| Line::from(l.to_string())).collect();
            body_lines.push(Line::from(""));
            body_lines.push(footer);

            let para = Paragraph::new(Text::from(body_lines))
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(para, area);
        }

        Popup::Input { title, value, .. } => {
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

            // Input field
            let input_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // spacer
                    Constraint::Length(1), // input
                    Constraint::Length(1), // spacer
                    Constraint::Length(1), // hint
                ])
                .split(inner);

            // Input box with value
            let input_line = if value.is_empty() {
                vec![
                    Span::styled(" ", Style::default()),
                    Span::styled("▎", Style::default().fg(SAPPHIRE).slow_blink()),
                    Span::styled(" 输入路径...", Style::default().fg(OVERLAY)),
                ]
            } else {
                vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(value.clone(), Style::default().fg(TEXT)),
                    Span::styled("▎", Style::default().fg(SAPPHIRE)),
                ]
            };
            let input = Paragraph::new(Line::from(input_line))
                .style(Style::default().bg(BG));
            f.render_widget(input, input_chunks[1]);

            // Footer hint
            let hint = Line::from(vec![
                Span::styled(" Enter ", Style::default().bg(GREEN).fg(BG).bold()),
                Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                Span::styled(" Esc ", Style::default().bg(RED).fg(BG).bold()),
                Span::styled(" 取消", Style::default().fg(SUBTEXT)),
            ])
            .alignment(Alignment::Center);
            f.render_widget(Paragraph::new(hint), input_chunks[3]);
        }

        Popup::Confirm { title, body, .. } => {
            let block = Block::default()
                .title(Span::styled(
                    format!(" ⚠ {title} "),
                    Style::default().fg(PEACH).bold(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PEACH))
                .style(Style::default().bg(SURFACE));

            let mut lines: Vec<Line> = body.lines().map(|l| Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(TEXT),
            ))).collect();
            lines.push(Line::from(""));
            lines.push(
                Line::from(vec![
                    Span::styled(" Y/Enter ", Style::default().bg(RED).fg(BG).bold()),
                    Span::styled(" 确认  ", Style::default().fg(SUBTEXT)),
                    Span::styled(" 其他键 ", Style::default().bg(OVERLAY).fg(TEXT)),
                    Span::styled(" 取消", Style::default().fg(SUBTEXT)),
                ])
                .alignment(Alignment::Center),
            );

            let para = Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(para, area);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn block_default(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(LAVENDER).bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(OVERLAY))
        .style(Style::default().bg(BG))
}

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
