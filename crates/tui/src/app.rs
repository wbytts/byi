use std::path::PathBuf;

use byi_skill::{SkillEntry, SkillManager};
use byi_storage::RemoteConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Home,
    Skills,
    Sync,
}

#[derive(Clone, Debug)]
pub enum Popup {
    Message { title: String, body: String },
    Input { title: String, value: String, action: InputAction },
    Confirm { title: String, body: String, action: ConfirmAction },
}

#[derive(Clone, Debug)]
pub enum InputAction {
    AddSkillPath,
    AddSkillGithub,
}

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    RemoveSkill(String),
}

#[allow(dead_code)]
pub struct App {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub current_tab: Tab,
    pub should_quit: bool,
    pub status_message: String,
    pub hello_message: String,

    pub skill_manager: SkillManager,
    pub skill_entries: Vec<SkillEntry>,
    pub skill_selected: usize,
    pub skill_scan_issues: Vec<byi_skill::SkillDoctorIssue>,
    pub skill_scroll: usize,
    pub show_skill_detail: bool,

    pub sync_config: Option<RemoteConfig>,
    pub popup: Option<Popup>,
    pub input_cursor: usize,
}

impl App {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf) -> Result<Self, String> {
        let skill_manager = SkillManager::new(&data_dir);
        let scan = skill_manager.scan_and_reconcile_skills()?;
        let entries = byi_skill::join_skill_entries(&scan.skills, &scan.installed_skills);
        let sync_config = load_config(&config_dir).ok().and_then(|c| c.remote);
        let hello = byi_core::hello_message();

        Ok(Self {
            config_dir,
            data_dir,
            current_tab: Tab::Home,
            should_quit: false,
            status_message: String::new(),
            hello_message: hello,
            skill_manager,
            skill_entries: entries,
            skill_selected: 0,
            skill_scan_issues: scan.issues,
            skill_scroll: 0,
            show_skill_detail: true,
            sync_config,
            popup: None,
            input_cursor: 0,
        })
    }

    pub fn refresh_skills(&mut self) {
        match self.skill_manager.scan_and_reconcile_skills() {
            Ok(scan) => {
                self.skill_entries =
                    byi_skill::join_skill_entries(&scan.skills, &scan.installed_skills);
                self.skill_scan_issues = scan.issues;
                if self.skill_selected >= self.skill_entries.len() && !self.skill_entries.is_empty()
                {
                    self.skill_selected = self.skill_entries.len() - 1;
                }
            }
            Err(e) => self.set_status(format!("刷新 skill 失败: {e}")),
        }
    }

    pub fn refresh_sync(&mut self) {
        self.sync_config = load_config(&self.config_dir).ok().and_then(|c| c.remote);
    }

    pub fn handle_key(&mut self, code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> bool {
        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && code == crossterm::event::KeyCode::Char('c') {
            return true;
        }

        if let Some(popup) = self.popup.take() {
            return self.handle_popup_key(code, popup);
        }

        match code {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => return true,
            crossterm::event::KeyCode::Char('1') => self.current_tab = Tab::Home,
            crossterm::event::KeyCode::Char('2') => self.current_tab = Tab::Skills,
            crossterm::event::KeyCode::Char('3') => self.current_tab = Tab::Sync,
            crossterm::event::KeyCode::Tab => self.next_tab(),
            crossterm::event::KeyCode::BackTab => self.prev_tab(),
            _ => match self.current_tab {
                Tab::Home => self.handle_home_key(code),
                Tab::Skills => self.handle_skills_key(code),
                Tab::Sync => self.handle_sync_key(code),
            },
        }
        false
    }

    fn handle_popup_key(&mut self, code: crossterm::event::KeyCode, popup: Popup) -> bool {
        match popup {
            Popup::Message { .. } => {
                self.popup = None;
            }
            Popup::Input { title, mut value, action } => match code {
                crossterm::event::KeyCode::Esc => self.popup = None,
                crossterm::event::KeyCode::Enter => {
                    self.popup = None;
                    self.execute_input_action(action, &value);
                }
                crossterm::event::KeyCode::Char(c) => {
                    value.insert(self.input_cursor, c);
                    self.input_cursor += 1;
                    self.popup = Some(Popup::Input { title, value, action });
                }
                crossterm::event::KeyCode::Backspace => {
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                        value.remove(self.input_cursor);
                    }
                    self.popup = Some(Popup::Input { title, value, action });
                }
                crossterm::event::KeyCode::Left => {
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                    }
                    self.popup = Some(Popup::Input { title, value, action });
                }
                crossterm::event::KeyCode::Right => {
                    if self.input_cursor < value.len() {
                        self.input_cursor += 1;
                    }
                    self.popup = Some(Popup::Input { title, value, action });
                }
                _ => {
                    self.popup = Some(Popup::Input { title, value, action });
                }
            },
            Popup::Confirm { title: _, body: _, action } => match code {
                crossterm::event::KeyCode::Char('y')
                | crossterm::event::KeyCode::Char('Y')
                | crossterm::event::KeyCode::Enter => {
                    self.popup = None;
                    self.execute_confirm_action(action);
                }
                _ => {
                    self.popup = None;
                    self.set_status("操作已取消".to_string());
                }
            },
        }
        false
    }

    fn execute_input_action(&mut self, action: InputAction, value: &str) {
        match action {
            InputAction::AddSkillPath => {
                if value.trim().is_empty() {
                    self.set_status("路径不能为空".to_string());
                    return;
                }
                let cmd = byi_skill::SkillCommand::Add(byi_skill::SkillAddCommand {
                    path: Some(value.trim().to_string()),
                    github: None,
                    r#ref: None,
                    subdir: None,
                });
                match self.skill_manager.run_command(Some(cmd)) {
                    Ok(msg) => {
                        self.set_status(msg);
                        self.refresh_skills();
                    }
                    Err(e) => self.set_status(format!("添加失败: {e}")),
                }
            }
            InputAction::AddSkillGithub => {
                if value.trim().is_empty() {
                    self.set_status("GitHub repo 不能为空".to_string());
                    return;
                }
                let cmd = byi_skill::SkillCommand::Add(byi_skill::SkillAddCommand {
                    path: None,
                    github: Some(value.trim().to_string()),
                    r#ref: None,
                    subdir: None,
                });
                match self.skill_manager.run_command(Some(cmd)) {
                    Ok(msg) => {
                        self.set_status(msg);
                        self.refresh_skills();
                    }
                    Err(e) => self.set_status(format!("添加失败: {e}")),
                }
            }
        }
    }

    fn execute_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::RemoveSkill(id) => {
                let cmd = byi_skill::SkillCommand::Remove(byi_skill::SkillInstanceCommand {
                    instance_id: id,
                });
                match self.skill_manager.run_command(Some(cmd)) {
                    Ok(msg) => {
                        self.set_status(msg);
                        self.refresh_skills();
                    }
                    Err(e) => self.set_status(format!("删除失败: {e}")),
                }
            }
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Home => Tab::Skills,
            Tab::Skills => Tab::Sync,
            Tab::Sync => Tab::Home,
        };
    }

    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Home => Tab::Sync,
            Tab::Skills => Tab::Home,
            Tab::Sync => Tab::Skills,
        };
    }

    fn handle_home_key(&mut self, _code: crossterm::event::KeyCode) {}

    fn handle_skills_key(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                if self.skill_selected > 0 {
                    self.skill_selected -= 1;
                }
                if self.skill_selected < self.skill_scroll {
                    self.skill_scroll = self.skill_selected;
                }
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                if !self.skill_entries.is_empty() && self.skill_selected < self.skill_entries.len() - 1
                {
                    self.skill_selected += 1;
                }
            }
            crossterm::event::KeyCode::Char(' ') | crossterm::event::KeyCode::Enter => {
                self.show_skill_detail = !self.show_skill_detail;
            }
            crossterm::event::KeyCode::Char('a') => {
                self.popup = Some(Popup::Input {
                    title: "添加本地 Skill".to_string(),
                    value: String::new(),
                    action: InputAction::AddSkillPath,
                });
                self.input_cursor = 0;
            }
            crossterm::event::KeyCode::Char('g') => {
                self.popup = Some(Popup::Input {
                    title: "从 GitHub 添加 Skill (owner/repo)".to_string(),
                    value: String::new(),
                    action: InputAction::AddSkillGithub,
                });
                self.input_cursor = 0;
            }
            crossterm::event::KeyCode::Char('e') => {
                if let Some(entry) = self.skill_entries.get(self.skill_selected) {
                    let cmd = byi_skill::SkillCommand::Enable(byi_skill::SkillInstanceCommand {
                        instance_id: entry.installed.instance_id.clone(),
                    });
                    match self.skill_manager.run_command(Some(cmd)) {
                        Ok(msg) => {
                            self.set_status(msg);
                            self.refresh_skills();
                        }
                        Err(e) => self.set_status(format!("启用失败: {e}")),
                    }
                }
            }
            crossterm::event::KeyCode::Char('d') => {
                if let Some(entry) = self.skill_entries.get(self.skill_selected) {
                    let cmd = byi_skill::SkillCommand::Disable(byi_skill::SkillInstanceCommand {
                        instance_id: entry.installed.instance_id.clone(),
                    });
                    match self.skill_manager.run_command(Some(cmd)) {
                        Ok(msg) => {
                            self.set_status(msg);
                            self.refresh_skills();
                        }
                        Err(e) => self.set_status(format!("停用失败: {e}")),
                    }
                }
            }
            crossterm::event::KeyCode::Char('r') => {
                self.refresh_skills();
                self.set_status("Skill 列表已刷新".to_string());
            }
            crossterm::event::KeyCode::Char('R') => {
                let cmd = byi_skill::SkillCommand::Rescan(Default::default());
                match self.skill_manager.run_command(Some(cmd)) {
                    Ok(msg) => {
                        self.set_status(msg);
                        self.refresh_skills();
                    }
                    Err(e) => self.set_status(format!("重扫描失败: {e}")),
                }
            }
            crossterm::event::KeyCode::Char('D') => {
                let cmd = byi_skill::SkillCommand::Doctor(Default::default());
                match self.skill_manager.run_command(Some(cmd)) {
                    Ok(msg) => {
                        self.popup = Some(Popup::Message {
                            title: "Skill Doctor".to_string(),
                            body: msg,
                        });
                        self.refresh_skills();
                    }
                    Err(e) => self.set_status(format!("Doctor 失败: {e}")),
                }
            }
            crossterm::event::KeyCode::Char('x') => {
                if let Some(entry) = self.skill_entries.get(self.skill_selected) {
                    self.popup = Some(Popup::Confirm {
                        title: "确认删除".to_string(),
                        body: format!(
                            "确定要删除 skill '{}' (instance: {}) 吗？\n按 Y/Enter 确认，其他键取消",
                            entry.skill.name, entry.installed.instance_id
                        ),
                        action: ConfirmAction::RemoveSkill(
                            entry.installed.instance_id.clone(),
                        ),
                    });
                }
            }
            crossterm::event::KeyCode::Char('v') => {
                if let Some(entry) = self.skill_entries.get(self.skill_selected) {
                    let ref_id = entry.installed.instance_id.clone();
                    let cmd = byi_skill::SkillCommand::View(byi_skill::SkillViewCommand {
                        reference: ref_id,
                        format: Default::default(),
                    });
                    match self.skill_manager.run_command(Some(cmd)) {
                        Ok(msg) => {
                            self.popup = Some(Popup::Message {
                                title: format!("Skill: {}", entry.skill.name),
                                body: msg,
                            });
                        }
                        Err(e) => self.set_status(format!("查看失败: {e}")),
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_sync_key(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Char('t') => {
                if let Some(remote) = &self.sync_config {
                    let storage = byi_storage::storage_for(remote);
                    match storage.test() {
                        Ok(()) => self.set_status("Sync remote test passed.".to_string()),
                        Err(e) => self.set_status(format!("Sync test 失败: {e}")),
                    }
                } else {
                    self.set_status("未配置同步远端".to_string());
                }
            }
            crossterm::event::KeyCode::Char('p') => {
                self.run_sync_pull();
            }
            crossterm::event::KeyCode::Char('P') => {
                self.run_sync_push();
            }
            crossterm::event::KeyCode::Char('r') => {
                self.refresh_sync();
                self.set_status("同步配置已刷新".to_string());
            }
            _ => {}
        }
    }

    fn run_sync_pull(&mut self) {
        let remote = match &self.sync_config {
            Some(r) => r.clone(),
            None => {
                self.set_status("未配置同步远端".to_string());
                return;
            }
        };
        let storage = byi_storage::storage_for(&remote);
        match self.skill_manager.sync_pull_from_storage(storage.as_ref()) {
            Ok(()) => {
                self.set_status("Pulled remote data.".to_string());
                self.refresh_skills();
            }
            Err(e) => self.set_status(format!("Pull 失败: {e}")),
        }
    }

    fn run_sync_push(&mut self) {
        let remote = match &self.sync_config {
            Some(r) => r.clone(),
            None => {
                self.set_status("未配置同步远端".to_string());
                return;
            }
        };
        let storage = byi_storage::storage_for(&remote);
        match self.skill_manager.sync_push_to_storage(storage.as_ref()) {
            Ok(()) => self.set_status("Pushed local data.".to_string()),
            Err(e) => self.set_status(format!("Push 失败: {e}")),
        }
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = msg;
    }

    pub fn on_tick(&mut self) {}

    pub fn selected_skill(&self) -> Option<&SkillEntry> {
        self.skill_entries.get(self.skill_selected)
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct ByiConfig {
    remote: Option<RemoteConfig>,
}

fn load_config(config_dir: &PathBuf) -> Result<ByiConfig, String> {
    let path = config_dir.join("config.toml");
    if !path.exists() {
        return Ok(ByiConfig::default());
    }
    let contents = std::fs::read_to_string(&path)
        .map_err(|err| format!("读取配置失败 {}: {err}", path.display()))?;
    toml::from_str(&contents).map_err(|err| format!("解析配置失败 {}: {err}", path.display()))
}
