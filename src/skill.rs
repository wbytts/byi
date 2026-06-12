use byi_skill::SkillManager;

use crate::app::App;
use crate::cli::{
    SkillAddCommand, SkillCommand, SkillEditCommand, SkillFormatCommand, SkillInstanceCommand,
    SkillInstancesCommand, SkillListCommand, SkillViewCommand,
};

impl App {
    pub(crate) fn run_skill(&self, command: Option<SkillCommand>) -> Result<String, String> {
        self.skill_manager().run_command(convert_command(command))
    }

    pub(crate) fn sync_pull_skill_data(&self) -> Result<(), String> {
        let remote = self.require_remote()?;
        let storage = byi_storage::storage_for(&remote);
        self.skill_manager()
            .sync_pull_from_storage(storage.as_ref())
    }

    pub(crate) fn sync_push_skill_data(&self) -> Result<(), String> {
        let remote = self.require_remote()?;
        let storage = byi_storage::storage_for(&remote);
        self.skill_manager().sync_push_to_storage(storage.as_ref())
    }

    fn skill_manager(&self) -> SkillManager {
        SkillManager::new(self.data_dir.clone())
    }
}

fn convert_command(command: Option<SkillCommand>) -> Option<byi_skill::SkillCommand> {
    command.map(|command| match command {
        SkillCommand::Add(command) => byi_skill::SkillCommand::Add(convert_add_command(command)),
        SkillCommand::List(command) => byi_skill::SkillCommand::List(convert_list_command(command)),
        SkillCommand::View(command) => byi_skill::SkillCommand::View(convert_view_command(command)),
        SkillCommand::Edit(command) => byi_skill::SkillCommand::Edit(convert_edit_command(command)),
        SkillCommand::Remove(command) => {
            byi_skill::SkillCommand::Remove(convert_instance_command(command))
        }
        SkillCommand::Enable(command) => {
            byi_skill::SkillCommand::Enable(convert_instance_command(command))
        }
        SkillCommand::Disable(command) => {
            byi_skill::SkillCommand::Disable(convert_instance_command(command))
        }
        SkillCommand::Instances(command) => {
            byi_skill::SkillCommand::Instances(convert_instances_command(command))
        }
        SkillCommand::Doctor(format) => {
            byi_skill::SkillCommand::Doctor(convert_format_command(format))
        }
        SkillCommand::Rescan(format) => {
            byi_skill::SkillCommand::Rescan(convert_format_command(format))
        }
    })
}

fn convert_add_command(command: SkillAddCommand) -> byi_skill::SkillAddCommand {
    byi_skill::SkillAddCommand {
        path: command.path,
        github: command.github,
        r#ref: command.r#ref,
        subdir: command.subdir,
    }
}

fn convert_list_command(command: SkillListCommand) -> byi_skill::SkillListCommand {
    byi_skill::SkillListCommand {
        format: convert_format_command(command.format),
        enabled: command.enabled,
        disabled: command.disabled,
    }
}

fn convert_view_command(command: SkillViewCommand) -> byi_skill::SkillViewCommand {
    byi_skill::SkillViewCommand {
        reference: command.reference,
        format: convert_format_command(command.format),
    }
}

fn convert_edit_command(command: SkillEditCommand) -> byi_skill::SkillEditCommand {
    byi_skill::SkillEditCommand {
        reference: command.reference,
    }
}

fn convert_instance_command(command: SkillInstanceCommand) -> byi_skill::SkillInstanceCommand {
    byi_skill::SkillInstanceCommand {
        instance_id: command.instance_id,
    }
}

fn convert_instances_command(command: SkillInstancesCommand) -> byi_skill::SkillInstancesCommand {
    byi_skill::SkillInstancesCommand {
        format: convert_format_command(command.format),
    }
}

fn convert_format_command(command: SkillFormatCommand) -> byi_skill::SkillFormatCommand {
    byi_skill::SkillFormatCommand {
        json: command.json,
        long: command.long,
    }
}
