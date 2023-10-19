#![allow(dead_code)]
use crate::Group;
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

pub enum Modifier {
    Qos,
    DefaultQOS,
    Account,
}

impl Modifier {
    pub fn to_static_ref(&self) -> &'static str {
        match self {
            Self::Qos => QOS,
            Self::DefaultQOS => DEFAULT_QOS,
            Self::Account => ACCOUNT,
        }
    }
}

enum SlurmSubCommand {
    Add {
        group: Group,
    },
    Delete,
    Modify {
        prefix: &'static str,
        value: Vec<String>,
    },
    Show {
        parseable: bool,
    },
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
        SlurmSubCommand::Modify { prefix, value } => {
            vec![
                SUB_COMMAND_MODIFY.into(),
                USER.into(),
                username,
                SET.into(),
                format!("{}={}", prefix, value.join(",")),
            ]
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
// User%30,Account,DefaultQOS,QOS%80
pub struct CommandBuilder {
    sub_commands: Vec<SlurmSubCommand>,
    username: String,
    immediate: bool,
    sacctmgr_path: String,
}

impl CommandBuilder {
    fn new_inner(username: String, sub_commands: Vec<SlurmSubCommand>) -> Self {
        Self {
            sub_commands,
            username,
            immediate: false,
            sacctmgr_path: SACCTMG_NAME.to_owned(),
        }
    }

    pub fn new_delete(username: String) -> Self {
        Self::new_inner(username, vec![SlurmSubCommand::Delete])
    }

    pub fn new_show(parseable: bool) -> Self {
        Self::new_inner(
            Default::default(),
            vec![SlurmSubCommand::Show { parseable }],
        )
    }

    pub fn new_modify(username: String, modifier: Modifier, value: Vec<String>) -> Self {
        Self::new_inner(
            username,
            vec![SlurmSubCommand::Modify {
                prefix: modifier.to_static_ref(),
                value,
            }],
        )
    }

    pub fn new_add(username: String, group: Group, default_qos: String, qos: Vec<String>) -> Self {
        // Note: The order of execution is important here!
        // Slurm expects the user to have QOS, before it can set the default QOS
        Self::new_inner(
            username,
            vec![
                SlurmSubCommand::Add { group },
                SlurmSubCommand::Modify {
                    prefix: QOS,
                    value: qos,
                },
                SlurmSubCommand::Modify {
                    prefix: DEFAULT_QOS,
                    value: vec![default_qos],
                },
            ],
        )
    }

    pub fn immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }
    pub fn sacctmgr_path(mut self, sacctmgr_path: String) -> Self {
        self.sacctmgr_path = sacctmgr_path;
        self
    }

    pub fn remote_command(self) -> Vec<String> {
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
    pub fn local_command(self) -> Vec<Command> {
        let args = Self::construct_args(self.username, self.immediate, self.sub_commands);
        args.into_iter()
            .map(|args| {
                let mut command = Command::new(&self.sacctmgr_path);
                command.args(args);
                command
            })
            .collect()
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

        let actual = input.remote_command();
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

        let actual = input.remote_command();
        insta::assert_yaml_snapshot!(actual);
    }

    #[test]
    fn produce_delete_user_with_seperate_path() {
        let input = CommandBuilder::new_delete("somebody".to_owned())
            .sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_command();
        insta::assert_yaml_snapshot!(actual);
    }
    #[test]
    fn produce_delete_user_with_local_command() {
        let input = CommandBuilder::new_delete("somebody".to_owned())
            .sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.local_command();
        insta::assert_debug_snapshot!(actual);
    }
    #[test]
    fn list_user() {
        let input = CommandBuilder::new_show(false).sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_command();
        insta::assert_debug_snapshot!(actual);
    }
    #[test]
    fn list_user_parserable() {
        let input = CommandBuilder::new_show(true).sacctmgr_path("some_path/sacctmgr".to_owned());
        let actual = input.remote_command();
        insta::assert_debug_snapshot!(actual);
    }
    #[test]
    fn modify_user() {
        let input = CommandBuilder::new_modify(
            "somebody".to_owned(),
            Modifier::Qos,
            vec!["student".into(), "staff".into()],
        );
        let actual = input.remote_command();
        insta::assert_debug_snapshot!(actual);
    }
}
