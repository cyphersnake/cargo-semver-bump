use std::str::FromStr;

pub use conventional_commits_parser::Commit as ConventionalCommit;

#[derive(Debug, PartialEq, Eq, Clone, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ConventionalCommitType {
    Build,
    Chore,
    Ci,
    Docs,
    Feat,
    Fix,
    Perf,
    Refactor,
    Revert,
    Style,
    Test,
    #[strum(disabled)]
    Custom(String),
}

impl ConventionalCommitType {
    pub fn new(ty_: &str) -> Self {
        match Self::from_str(ty_) {
            Ok(type_) => type_,
            Err(strum::ParseError::VariantNotFound) => Self::Custom(ty_.to_owned()),
        }
    }
}
impl<'r> From<&ConventionalCommit<'r>> for ConventionalCommitType {
    fn from(value: &ConventionalCommit) -> Self {
        Self::new(value.ty)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ConventionalCommit, ConventionalCommitType};

    #[test]
    fn test_conventional_commit_type_from_str() {
        assert_eq!(
            ConventionalCommitType::from_str("build"),
            Ok(ConventionalCommitType::Build)
        );
        assert_eq!(
            ConventionalCommitType::from_str("chore"),
            Ok(ConventionalCommitType::Chore)
        );
        assert_eq!(
            ConventionalCommitType::from_str("ci"),
            Ok(ConventionalCommitType::Ci)
        );
        assert_eq!(
            ConventionalCommitType::from_str("docs"),
            Ok(ConventionalCommitType::Docs)
        );
        assert_eq!(
            ConventionalCommitType::from_str("feat"),
            Ok(ConventionalCommitType::Feat)
        );
        assert_eq!(
            ConventionalCommitType::from_str("fix"),
            Ok(ConventionalCommitType::Fix)
        );
        assert_eq!(
            ConventionalCommitType::from_str("perf"),
            Ok(ConventionalCommitType::Perf)
        );
        assert_eq!(
            ConventionalCommitType::from_str("refactor"),
            Ok(ConventionalCommitType::Refactor)
        );
        assert_eq!(
            ConventionalCommitType::from_str("revert"),
            Ok(ConventionalCommitType::Revert)
        );
        assert_eq!(
            ConventionalCommitType::from_str("style"),
            Ok(ConventionalCommitType::Style)
        );
        assert_eq!(
            ConventionalCommitType::from_str("test"),
            Ok(ConventionalCommitType::Test)
        );
    }

    #[test]
    fn test_conventional_commit_type_new() {
        assert_eq!(
            ConventionalCommitType::new("build"),
            ConventionalCommitType::Build
        );
        assert_eq!(
            ConventionalCommitType::new("chore"),
            ConventionalCommitType::Chore
        );
        assert_eq!(
            ConventionalCommitType::new("non_existent"),
            ConventionalCommitType::Custom("non_existent".to_owned())
        );
    }

    #[test]
    fn test_conventional_commit_type_from_conventional_commit() {
        let commit = ConventionalCommit {
            ty: "build",
            body: None,
            desc: "",
            footer: vec![],
            is_breaking_change: false,
            scope: None,
        };
        assert_eq!(
            ConventionalCommitType::from(&commit),
            ConventionalCommitType::Build
        );

        let commit = ConventionalCommit {
            ty: "non_existent",
            body: None,
            desc: "",
            footer: vec![],
            is_breaking_change: false,
            scope: None,
        };
        assert_eq!(
            ConventionalCommitType::from(&commit),
            ConventionalCommitType::Custom("non_existent".to_owned())
        );
    }
}
