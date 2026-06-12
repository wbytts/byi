use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, Tabs, Wrap,
    },
    Frame,
};

use crate::app::{App, Popup, Tab};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    if let Some(popup) = &app.popup {
        draw_popup(f, app, popup);
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.status_message.is_empty() {
        " q:退出 1/2/3:切换标签 ".to_string()
    } else {
        format!(" {} ", app.status_message)
    };
    let status = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = vec!["Home", "Skills", "Sync"]
        .into_iter()
        .map(|t| Line::from(format!("  {}  ", t)))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().title(" byi ").borders(Borders::ALL))
        .select(app.current_tab as usize)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("");
    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Home => draw_home(f, app, area),
        Tab::Skills => draw_skills(f, app, area),
        Tab::Sync => draw_sync(f, app, area),
    }
}

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Home ")
        .borders(Borders::ALL);
    let text = Text::from(vec![
        Line::from(""),
        Line::from(app.hello_message.clone()).alignment(Alignment::Center),
        Line::from(""),
        Line::from("═══ 快捷键 ═══").alignment(Alignment::Center),
        Line::from(""),
        Line::from("  1 / Tab    →  Home").alignment(Alignment::Center),
        Line::from("  2          →  Skills 管理").alignment(Alignment::Center),
        Line::from("  3          →  Sync 同步").alignment(Alignment::Center),
        Line::from("  q / Ctrl+C →  退出").alignment(Alignment::Center),
    ]);
    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_skills(f: &mut Frame, app: &App, area: Rect) {
    let constraints = if app.show_skill_detail && !app.skill_entries.is_empty() {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
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
    let header = Row::new(vec!["#", "名称", "状态", "来源"])
        .style(Style::default().fg(Color::Yellow))
        .height(1);

    let rows: Vec<Row> = app
        .skill_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.skill_selected;
            let status = if entry.installed.enabled {
                Span::styled("✓ 启用", Style::default().fg(Color::Green))
            } else {
                Span::styled("✗ 停用", Style::default().fg(Color::Red))
            };
            let source = if entry.installed.source.starts_with("github:") {
                entry.installed.source.replacen("github:", "", 1)
            } else {
                entry.installed.source.clone()
            };
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from((i + 1).to_string()),
                Cell::from(entry.skill.name.clone()),
                Cell::from(Line::from(vec![status])),
                Cell::from(source),
            ])
            .style(style)
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
            .title(format!(
                " Skills ({}) [a:添加本地 g:GitHub e:启用 d:停用 x:删除 v:查看 r:刷新 R:重扫描 D:Doctor] ",
                app.skill_entries.len()
            ))
            .borders(Borders::ALL),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, area);

    if !app.skill_entries.is_empty() {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut state = ScrollbarState::new(app.skill_entries.len()).position(app.skill_selected);
        f.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }
}

fn draw_skill_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL);

    let text = if let Some(entry) = app.selected_skill() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("名称: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.skill.name.clone()),
            ]),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.skill.id.clone()),
            ]),
            Line::from(vec![
                Span::styled("实例: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.instance_id.clone()),
            ]),
            Line::from(vec![
                Span::styled("目录: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.dir_name.clone()),
            ]),
            Line::from(vec![
                Span::styled("路径: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.install_path.clone()),
            ]),
            Line::from(vec![
                Span::styled("来源: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.source.clone()),
            ]),
            Line::from(vec![
                Span::styled("状态: ", Style::default().fg(Color::Yellow)),
                if entry.installed.enabled {
                    Span::styled("已启用", Style::default().fg(Color::Green))
                } else {
                    Span::styled("已停用", Style::default().fg(Color::Red))
                },
            ]),
            Line::from(vec![
                Span::styled("创建: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.created_at.clone()),
            ]),
            Line::from(vec![
                Span::styled("更新: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.installed.updated_at.clone()),
            ]),
        ];
        if !entry.skill.description.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("描述: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.skill.description.clone()),
            ]));
        }
        if !entry.skill.domains.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("领域: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.skill.domains.join(", ")),
            ]));
        }
        if !entry.skill.modules.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("模块: ", Style::default().fg(Color::Yellow)),
                Span::from(entry.skill.modules.join(", ")),
            ]));
        }
        Text::from(lines)
    } else {
        Text::from("未选择 skill")
    };

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_sync(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Sync ")
        .borders(Borders::ALL);

    let text = if let Some(remote) = &app.sync_config {
        let mut lines = vec![Line::from(""), Line::from("同步配置").alignment(Alignment::Center)];
        match remote {
            byi_storage::RemoteConfig::GitHub(config) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("类型: ", Style::default().fg(Color::Yellow)),
                    Span::from("GitHub"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("仓库: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.repo.clone()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("分支: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.branch.clone()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("基础路径: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.base_path.clone()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("认证: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.auth.clone()),
                ]));
            }
            byi_storage::RemoteConfig::WebDav(config) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("类型: ", Style::default().fg(Color::Yellow)),
                    Span::from("WebDAV"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("端点: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.endpoint_url.clone()),
                ]));
                let preset = match config.preset {
                    byi_webdav::WebDavPreset::Jianguoyun => "坚果云",
                    byi_webdav::WebDavPreset::Custom => "自定义",
                };
                lines.push(Line::from(vec![
                    Span::styled("预设: ", Style::default().fg(Color::Yellow)),
                    Span::from(preset),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("基础路径: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.base_path.clone()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("用户名: ", Style::default().fg(Color::Yellow)),
                    Span::from(config.username.clone().unwrap_or_default()),
                ]));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from("═══ 快捷键 ═══").alignment(Alignment::Center));
        lines.push(Line::from(""));
        lines.push(Line::from("  t → 测试连通性").alignment(Alignment::Center));
        lines.push(Line::from("  p → 从远端拉取 (pull)").alignment(Alignment::Center));
        lines.push(Line::from("  P → 推送到远端 (push)").alignment(Alignment::Center));
        lines.push(Line::from("  r → 刷新配置").alignment(Alignment::Center));
        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from("未配置同步远端").alignment(Alignment::Center),
            Line::from(""),
            Line::from("请使用命令行配置:").alignment(Alignment::Center),
            Line::from(""),
            Line::from("  byi sync config").alignment(Alignment::Center),
            Line::from("  byi sync init --provider github --repo owner/repo").alignment(Alignment::Center),
            Line::from("  byi sync init --provider webdav --preset jianguoyun --username ...").alignment(Alignment::Center),
        ])
    };

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_popup(f: &mut Frame, _app: &App, popup: &Popup) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    match popup {
        Popup::Message { title, body } => {
            let block = Block::default().title(title.clone()).borders(Borders::ALL);
            let para = Paragraph::new(body.clone())
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(para, area);
        }
        Popup::Input { title, value, .. } => {
            let block = Block::default().title(title.clone()).borders(Borders::ALL);
            let text = Text::from(vec![
                Line::from(""),
                Line::from(value.clone()),
                Line::from(""),
                Line::from("Enter:确认  Esc:取消").alignment(Alignment::Center),
            ]);
            let para = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(para, area);
        }
        Popup::Confirm { title, body, .. } => {
            let block = Block::default().title(title.clone()).borders(Borders::ALL);
            let text = Text::from(vec![
                Line::from(body.clone()),
                Line::from(""),
                Line::from("Y/Enter:确认  其他键:取消").alignment(Alignment::Center),
            ]);
            let para = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(para, area);
        }
    }
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
