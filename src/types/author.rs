use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        // 如果两个都有邮箱则判断邮箱是否一致
        if self.email.is_some() && other.email.is_some() {
            return self.email == other.email;
        }
        self.name == other.name
    }
}

#[test]
fn test_author_eq() {
    assert_eq!(
        Author {
            name: "Cno".to_string(),
            email: None
        },
        Author {
            name: "Cno".to_string(),
            email: Some("dsyourshy@qq.com".to_string())
        }
    );
    assert_ne!(
        Author {
            name: "Cno".to_string(),
            email: Some("j3rry@qq.com".to_string())
        },
        Author {
            name: "Cno".to_string(),
            email: Some("dsyourshy@qq.com".to_string())
        }
    );
    assert_eq!(
        Author {
            name: "J3rry".to_string(),
            email: Some("dsyourshy@qq.com".to_string())
        },
        Author {
            name: "Cno".to_string(),
            email: Some("dsyourshy@qq.com".to_string())
        }
    );
    assert_ne!(
        Author {
            name: "Cno".to_string(),
            email: None
        },
        Author {
            name: "J3rry".to_string(),
            email: None
        }
    );
}
