use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

pub(super) const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

#[derive(Debug, Deserialize)]
pub(super) struct LoginUsers {
    pub(super) users: HashMap<String, LoginUser>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct LoginUser {
    pub(super) account_name: Option<String>,
    pub(super) persona_name: Option<String>,
}

impl LoginUser {
    pub(super) fn display_name(&self) -> Option<String> {
        self.persona_name
            .as_ref()
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                self.account_name
                    .as_ref()
                    .filter(|name| !name.trim().is_empty())
            })
            .cloned()
    }
}

pub(super) fn read_login_users(install_path: &Path) -> Option<LoginUsers> {
    let path = install_path.join("config").join("loginusers.vdf");
    let raw = fs::read_to_string(path).ok()?;
    parse_login_users_vdf(&raw)
}

fn parse_login_users_vdf(raw: &str) -> Option<LoginUsers> {
    let mut users = HashMap::new();
    let mut current_id: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut persona_name: Option<String> = None;

    for line in raw.lines() {
        let tokens = quoted_tokens(line);
        match tokens.as_slice() {
            [id] if id.chars().all(|c| c.is_ascii_digit()) => {
                if let Some(previous) = current_id.take() {
                    users.insert(
                        previous,
                        LoginUser {
                            account_name: account_name.take(),
                            persona_name: persona_name.take(),
                        },
                    );
                }
                current_id = Some(id.clone());
            }
            [key, value] if key == "AccountName" => {
                account_name = Some(value.clone());
            }
            [key, value] if key == "PersonaName" => {
                persona_name = Some(value.clone());
            }
            _ => {}
        }
    }

    if let Some(previous) = current_id {
        users.insert(
            previous,
            LoginUser {
                account_name,
                persona_name,
            },
        );
    }

    Some(LoginUsers { users })
}

pub(super) fn login_user_for_userdata_id<'a>(
    login_users: &'a LoginUsers,
    userdata_id: &str,
) -> Option<&'a LoginUser> {
    login_users.users.get(userdata_id).or_else(|| {
        let account_id = userdata_id.parse::<u64>().ok()?;
        let steam_id64 = account_id.checked_add(STEAM_ID64_BASE)?;
        login_users.users.get(&steam_id64.to_string())
    })
}

fn quoted_tokens(line: &str) -> Vec<String> {
    line.split('"')
        .enumerate()
        .filter(|&(index, _part)| index % 2 == 1)
        .map(|(_index, part)| part.to_string())
        .collect()
}
