use crate::Group;
use std::collections::HashMap;
use std::iter;
use std::process::Command;

const SACCTMG_NAME: &str = "sacctmgr";
const IMMEDIATE: &str = "--immediate";
const SUB_COMMAND_SHOW: &str = "show";
const SUB_COMMAND_ADD: &str = "add";
const SUB_COMMAND_MODIFY: &str = "modify";
const SUB_COMMAND_DELETE: &str = "delete";

const SET: &str = "set";
const ASSOCIATION: &str = "assoc";
const USER: &str = "User";
const ACCOUNT: &str = "Account";
const DEFAULT_QOS: &str = "DefaultQOS";
const QOS: &str = "QOS";
const SLURM_PRASEABLE_ARG: &str = "--parsable";

enum SlurmSubCommand {
    Add { group: Group },
    Delete,
    Modify(HashMap<&'static str, Vec<String>>),
    Show { parseable: bool },
}

fn from_username(value: SlurmSubCommand, username: String) -> Vec<String> {
    match value {
        SlurmSubCommand::Add { group } => {
            vec![
                SUB_COMMAND_ADD.into(),
                USER.into(),
                username,
                format!("{}={}", ACCOUNT, group.to_string()),
            ]
        }
        SlurmSubCommand::Delete => vec![SUB_COMMAND_DELETE.into(), USER.into(), username],
        SlurmSubCommand::Modify(map) => {
            let mut to_set: Vec<String> = map
                .into_iter()
                .map(|(key, values)| format!("{}={}", key, values.join(",")))
                .collect();
            to_set.sort();
            vec![SUB_COMMAND_MODIFY.into(), USER.into(), username, SET.into()]
                .into_iter()
                .chain(to_set)
                .collect()
        }
        SlurmSubCommand::Show { parseable } => {
            let mut command = if parseable {
                vec![SLURM_PRASEABLE_ARG.to_owned()]
            } else {
                Vec::new()
            };
            command.extend_from_slice(&[
                SUB_COMMAND_SHOW.into(),
                ASSOCIATION.into(),
                format!("format={}%30,{},{},{}%80", USER, ACCOUNT, DEFAULT_QOS, QOS),
            ]);
            command
        }
    }
}

/// Builder to construct slurm commands for execution as local process commands or as strings for
/// ssh remote execution.
/// One or more commands are added before the call of [`remote_commands`] for remote or [`local_commanded`] for local
pub struct CommandBuilder {
    sub_commands: Vec<SlurmSubCommand>,
    username: String,
    immediate: bool,
    sacctmgr_path: String,
}

impl CommandBuilder {
    pub fn new_delete(username: String) -> Self {
        Self::new_inner(username, vec![SlurmSubCommand::Delete])
    }

    pub fn new_show(parseable: bool) -> Self {
        Self::new_inner(
            Default::default(),
            vec![SlurmSubCommand::Show { parseable }],
        )
    }

    #[cfg(test)]
    pub fn new_modify(username: String, modifier: HashMap<&'static str, Vec<String>>) -> Self {
        Self::new_inner(username, vec![SlurmSubCommand::Modify(modifier)])
    }

    /// Adds a command to modify the default quality of service for user aka parameter `username`
    pub fn new_modify_qos_default_qos(
        username: String,
        default_qos: String,
        qos: Vec<String>,
    ) -> Self {
        let command = Self::create_modify_command(default_qos, qos);
        Self::new_inner(username, vec![command])
    }

    pub fn new_add(username: String, group: Group, default_qos: String, qos: Vec<String>) -> Self {
        // Note: The order of execution is important here!
        // Slurm expects the user to have QOS, before it can set the default QOS
        let mod_command = Self::create_modify_command(default_qos, qos);
        Self::new_inner(username, vec![SlurmSubCommand::Add { group }, mod_command])
    }

    pub fn immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }

    pub fn sacctmgr_path(mut self, sacctmgr_path: String) -> Self {
        self.sacctmgr_path = sacctmgr_path;
        self
    }

    pub fn remote_commands(self) -> Vec<String> {
        let args = Self::construct_args(self.username, self.immediate, self.sub_commands);
        args.into_iter()
            .map(|args| {
                let mut command = Vec::with_capacity(args.len() + 1);
                command.push(self.sacctmgr_path.to_owned());
                command.extend(args);
                command.join(" ")
            })
            .collect()
    }

    pub fn local_commands(self) -> Vec<Command> {
        let args = Self::construct_args(self.username, self.immediate, self.sub_commands);
        args.into_iter()
            .map(|args| {
                let mut command = Command::new(&self.sacctmgr_path);
                command.args(args);
                command
            })
            .collect()
    }

    fn new_inner(username: String, sub_commands: Vec<SlurmSubCommand>) -> Self {
        Self {
            sub_commands,
            username,
            immediate: false,
            sacctmgr_path: SACCTMG_NAME.to_owned(),
        }
    }

    fn create_modify_command(default_qos: String, qos: Vec<String>) -> SlurmSubCommand {
        let map = HashMap::from_iter([(DEFAULT_QOS, vec![default_qos]), (QOS, qos)]);
        SlurmSubCommand::Modify(map)
    }

    fn construct_args(
        username: String,
        immediate: bool,
        sub_commands: Vec<SlurmSubCommand>,
    ) -> Vec<Vec<String>> {
        sub_commands
            .into_iter()
            .map(|command| {
                let args = from_username(command, username.to_owned());
                if immediate {
                    args.into_iter()
                        .chain(iter::once(IMMEDIATE.to_owned()))
                        .collect()
                } else {
                    args
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn produce_add_username_with_account_and_qos() {
        let input = CommandBuilder::new_add(
            "somebody".to_owned(),
            Group::Staff,
            "student".to_owned(),
            vec!["student".into(), "worker".into()],
        );

        let actual = input.remote_commands();
        insta::assert_yaml_snapshot!(actual);
    }
    #[test]
    fn produce_immediate_add_username_with_account_and_qos() {
        let input = CommandBuilder::new_add(
            "somebody".to_owned(),
            Group::Staff,
            "student".to_owned(),
            vec!["student".into(), "worker".into()],
        )
        .immediate(true);

        let actual = input.remote_commands();
        insta::assert_yaml_snapshot!(actual);
    }

    #[test]
    fn produce_delete_user_with_separate_path() {
        let input = CommandBuilder::new_delete("somebody".to_owned())
            .sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_commands();
        insta::assert_yaml_snapshot!(actual);
    }

    #[test]
    fn produce_delete_user_with_local_command() {
        let input = CommandBuilder::new_delete("somebody".to_owned())
            .sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.local_commands();
        insta::assert_debug_snapshot!(actual);
    }

    #[test]
    fn list_user() {
        let input = CommandBuilder::new_show(false).sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_commands();
        insta::assert_debug_snapshot!(actual);
    }

    #[test]
    fn list_user_parserable() {
        let input = CommandBuilder::new_show(true).sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_commands();
        insta::assert_debug_snapshot!(actual);
    }

    #[test]
    fn modify_user() {
        let map: HashMap<&'static str, _> = HashMap::from_iter([
            (QOS, vec!["basic".to_string(), "interactive".to_string()]),
            (DEFAULT_QOS, vec!["basic".to_string()]),
        ]);
        let input = CommandBuilder::new_modify("somebody".to_owned(), map);
        let actual = input.remote_commands();
        insta::assert_debug_snapshot!(actual);
    }
}
