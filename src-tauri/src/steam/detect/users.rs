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

#[cfg(test)]
mod tests {
    use super::*;

    // quoted_tokens

    #[test]
    fn quoted_tokens_empty_line() {
        assert!(quoted_tokens("").is_empty());
    }

    #[test]
    fn quoted_tokens_single_value() {
        assert_eq!(
            quoted_tokens("\t\"76561198000000001\""),
            vec!["76561198000000001"]
        );
    }

    #[test]
    fn quoted_tokens_key_value_pair() {
        assert_eq!(
            quoted_tokens("\t\t\"AccountName\"\t\t\"bob\""),
            vec!["AccountName", "bob"]
        );
    }

    #[test]
    fn quoted_tokens_ignores_unquoted_content() {
        assert_eq!(quoted_tokens("{}"), Vec::<String>::new());
    }

    // parse_login_users_vdf

    #[test]
    fn parse_vdf_empty_input_returns_empty_map() {
        let result = parse_login_users_vdf("").unwrap();
        assert!(result.users.is_empty());
    }

    #[test]
    fn parse_vdf_single_user() {
        let raw = r#"
"users"
{
    "123456"
    {
        "AccountName"   "bob"
        "PersonaName"   "Bob"
    }
}
"#;
        let result = parse_login_users_vdf(raw).unwrap();
        assert_eq!(result.users.len(), 1);
        let user = &result.users["123456"];
        assert_eq!(user.account_name.as_deref(), Some("bob"));
        assert_eq!(user.persona_name.as_deref(), Some("Bob"));
    }

    #[test]
    fn parse_vdf_multiple_users() {
        let raw = r#"
"users"
{
    "111"
    {
        "AccountName"   "alice"
        "PersonaName"   "Alice"
    }
    "222"
    {
        "AccountName"   "carol"
        "PersonaName"   "Carol"
    }
}
"#;
        let result = parse_login_users_vdf(raw).unwrap();
        assert_eq!(result.users.len(), 2);
        assert!(result.users.contains_key("111"));
        assert!(result.users.contains_key("222"));
    }

    #[test]
    fn parse_vdf_missing_persona_name() {
        let raw = "\"99\"\n\"AccountName\"\t\"alice\"\n";
        let result = parse_login_users_vdf(raw).unwrap();
        let user = &result.users["99"];
        assert_eq!(user.account_name.as_deref(), Some("alice"));
        assert!(user.persona_name.is_none());
    }

    // LoginUser::display_name

    #[test]
    fn display_name_prefers_persona_name() {
        let user = LoginUser {
            account_name: Some("alice_login".to_string()),
            persona_name: Some("Alice".to_string()),
        };
        assert_eq!(user.display_name().as_deref(), Some("Alice"));
    }

    #[test]
    fn display_name_falls_back_to_account_name() {
        let user = LoginUser {
            account_name: Some("alice_login".to_string()),
            persona_name: None,
        };
        assert_eq!(user.display_name().as_deref(), Some("alice_login"));
    }

    #[test]
    fn display_name_skips_blank_persona_name() {
        let user = LoginUser {
            account_name: Some("alice_login".to_string()),
            persona_name: Some("   ".to_string()),
        };
        assert_eq!(user.display_name().as_deref(), Some("alice_login"));
    }

    #[test]
    fn display_name_none_when_both_absent() {
        let user = LoginUser {
            account_name: None,
            persona_name: None,
        };
        assert!(user.display_name().is_none());
    }

    // login_user_for_userdata_id

    #[test]
    fn lookup_by_direct_userdata_id() {
        let mut users = std::collections::HashMap::new();
        users.insert(
            "123456".to_string(),
            LoginUser {
                account_name: Some("bob".to_string()),
                persona_name: None,
            },
        );
        let lu = LoginUsers { users };
        assert!(login_user_for_userdata_id(&lu, "123456").is_some());
    }

    #[test]
    fn lookup_by_steam64_id_converts_to_userdata_id() {
        // userdata ID 123456 corresponds to SteamID64 76561197960389184
        let userdata_id: u64 = 123456;
        let steam64 = userdata_id + STEAM_ID64_BASE;
        let mut users = std::collections::HashMap::new();
        users.insert(
            steam64.to_string(),
            LoginUser {
                account_name: Some("bob".to_string()),
                persona_name: None,
            },
        );
        let lu = LoginUsers { users };
        assert!(login_user_for_userdata_id(&lu, &userdata_id.to_string()).is_some());
    }

    #[test]
    fn lookup_returns_none_for_unknown_id() {
        let lu = LoginUsers {
            users: std::collections::HashMap::new(),
        };
        assert!(login_user_for_userdata_id(&lu, "999999").is_none());
    }
}
