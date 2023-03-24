use log::*;
use semver::Version;

use crate::conventional_commit::{ConventionalCommit, ConventionalCommitType};

#[derive(Debug, PartialEq)]
pub struct VersionUpdateTooWeak {
    pub expected_at_least: Version,
    pub actual: Version,
}

#[derive(Debug, PartialEq)]
pub enum ProcessResult {
    Patch { new: Version },
    ManualChanged { previous: Version, current: Version },
}

#[derive(Debug)]
pub struct VersionUpdateHandler<'r> {
    pub previous: Version,
    pub current: Version,
    pub commit: ConventionalCommit<'r>,
}

impl<'r> VersionUpdateHandler<'r> {
    pub fn get_next_version(self) -> Result<ProcessResult, VersionUpdateTooWeak> {
        let mut candidate = self.previous.clone();

        let type_ = ConventionalCommitType::from(&self.commit);
        trace!("Type of commit: {type_:?}");

        let new_candidate = match type_ {
            _ if self.commit.is_breaking_change && candidate.major != 0 => {
                trace!("Breaking Change - Update Major");
                Version::new(candidate.major + 1, 0, 0)
            }
            ConventionalCommitType::Fix => {
                trace!("Fix without breaking change, update batch");
                candidate.patch += 1;
                candidate
            }
            ConventionalCommitType::Feat => {
                candidate.minor += 1;
                trace!("New feature, update minor version to {}", candidate.minor);
                candidate.patch = 0;
                candidate
            }
            type_ => {
                trace!("Type commit {type_:?}, no update version needed");
                candidate
            }
        };

        let is_manual_changed = self.current != self.previous;
        if is_manual_changed {
            trace!("Version was manual changed");
            if self.current < new_candidate {
                error!("New version ({}) updated not enough", self.current);
                Err(VersionUpdateTooWeak {
                    expected_at_least: new_candidate,
                    actual: self.current,
                })
            } else {
                trace!("Version {} updated enough", self.current);
                Ok(ProcessResult::ManualChanged {
                    previous: self.previous,
                    current: self.current,
                })
            }
        } else {
            let patch = ProcessResult::Patch { new: new_candidate };
            trace!("{patch:?} needed");
            Ok(patch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    use crate::conventional_commit::ConventionalCommit;

    fn create_commit(commit_type: &str, breaking_change: bool) -> ConventionalCommit {
        ConventionalCommit {
            ty: commit_type,
            scope: None,
            body: None,
            desc: "",
            footer: vec![],
            is_breaking_change: breaking_change,
        }
    }

    #[test]
    fn test_major_version_update() {
        assert_eq!(
            VersionUpdateHandler {
                previous: Version::new(1, 2, 3),
                current: Version::new(1, 2, 3),
                commit: create_commit("feat", true),
            }
            .get_next_version()
            .unwrap(),
            ProcessResult::Patch {
                new: Version::new(2, 0, 0)
            }
        );
    }

    #[test]
    fn test_minor_version_update() {
        assert_eq!(
            VersionUpdateHandler {
                previous: Version::new(1, 2, 3),
                current: Version::new(1, 2, 3),
                commit: create_commit("feat", false),
            }
            .get_next_version()
            .unwrap(),
            ProcessResult::Patch {
                new: Version::new(1, 3, 0)
            }
        );
    }

    #[test]
    fn test_patch_version_update() {
        assert_eq!(
            VersionUpdateHandler {
                previous: Version::new(1, 2, 3),
                current: Version::new(1, 2, 3),
                commit: create_commit("fix", false),
            }
            .get_next_version()
            .unwrap(),
            ProcessResult::Patch {
                new: Version::new(1, 2, 4)
            }
        );
    }

    #[test]
    fn test_manual_version_change() {
        const PREVIOUS: Version = Version::new(1, 2, 3);
        const CURRENT: Version = Version::new(1, 4, 0);

        assert_eq!(
            VersionUpdateHandler {
                previous: PREVIOUS,
                current: CURRENT,
                commit: create_commit("feat", false),
            }
            .get_next_version()
            .unwrap(),
            ProcessResult::ManualChanged {
                previous: PREVIOUS,
                current: CURRENT
            }
        );
    }

    #[test]
    fn test_version_update_too_weak() {
        let previous = Version::new(1, 2, 3);
        let current = Version::new(1, 2, 4);

        assert_eq!(
            VersionUpdateHandler {
                previous,
                current: current.clone(),
                commit: create_commit("feat", false),
            }
            .get_next_version()
            .unwrap_err(),
            VersionUpdateTooWeak {
                expected_at_least: Version::new(1, 3, 0),
                actual: current
            }
        );
    }
}
