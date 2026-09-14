use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

pub(super) const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
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
    keyvalues_serde::from_str(&raw).ok()
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

#[cfg(test)]
mod tests {
    use super::*;


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
        let result: LoginUsers = keyvalues_serde::from_str(raw).unwrap();
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
        let result: LoginUsers = keyvalues_serde::from_str(raw).unwrap();
        assert_eq!(result.users.len(), 2);
        assert!(result.users.contains_key("111"));
        assert!(result.users.contains_key("222"));
    }

    #[test]
    fn parse_vdf_missing_persona_name() {
        let raw = r#"
"users"
{
    "99"
    {
        "AccountName"   "alice"
    }
}
"#;
        let result: LoginUsers = keyvalues_serde::from_str(raw).unwrap();
        let user = &result.users["99"];
        assert_eq!(user.account_name.as_deref(), Some("alice"));
        assert!(user.persona_name.is_none());
    }

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
