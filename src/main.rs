use std::{fs, io, ops::Not, str};

use cargo_toml::{Inheritable, Manifest};
use git2::Repository;
use log::*;
use semver::Version;
use some_to_err::ErrOr;

mod conventional_commit;
mod version_update_handler;

use version_update_handler::{ProcessResult, VersionUpdateHandler, VersionUpdateTooWeak};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("TODO")]
    RepositoryNotFound(git2::Error),
    #[error("TODO")]
    WorkDirNotFound,
    #[error("TODO")]
    Git(#[from] git2::Error),
    #[error("TODO")]
    LostCargoToml,
    #[error("TODO")]
    TomlFileNotUtf8(#[from] str::Utf8Error),
    #[error("TODO")]
    CargoTomlCorrupted(#[from] cargo_toml::Error),
    #[error("TODO")]
    WhileWriteUpdatedToml(#[from] toml::ser::Error),
    #[error("TODO")]
    WhileWriteUpdatedTomlFile(#[from] io::Error),
    #[error("TODO")]
    SemverCorrupted(#[from] semver::Error),
    #[error("TODO")]
    CommitMessageEmpty,
    #[error("TODO")]
    CommitNotConvential(String),
    #[error("TODO")]
    LostVersionAtCargoToml,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let repo = Repository::discover(".").map_err(Error::RepositoryNotFound)?;

    let commit = repo.head()?.peel_to_commit()?;

    let workdir = repo.workdir().ok_or(Error::WorkDirNotFound)?;

    let cargo_toml_path = workdir.join("Cargo.toml");
    cargo_toml_path
        .exists()
        .not()
        .then_some(Error::LostCargoToml)
        .err_or(())?;

    let cargo_toml_path_relative = cargo_toml_path
        .strip_prefix(workdir)
        .expect("Safe, this is relatie path");

    let parent_cargo_toml_blob = repo.find_blob(
        commit
            .parent(0)?
            .tree()?
            .get_path(cargo_toml_path_relative)?
            .id(),
    )?;

    let parent_cargo_toml_str = str::from_utf8(parent_cargo_toml_blob.content())?;
    let previous_commit_manifest_version = Version::parse(
        Manifest::from_str(parent_cargo_toml_str)?
            .package()
            .version(),
    )?;

    let mut manifest = Manifest::from_path(&cargo_toml_path)?;
    let current = Version::parse(manifest.package().version())?;

    let ctx = VersionUpdateHandler {
        current,
        previous: previous_commit_manifest_version,
        commit: conventional_commits_parser::parse_commit_msg(
            commit.message().ok_or(Error::CommitMessageEmpty)?,
        )
        .map_err(|err| Error::CommitNotConvential(format!("{err:?}")))?,
    };

    match ctx.get_next_version() {
        Ok(ProcessResult::Patch { new }) => {
            manifest.package.as_mut().ok_or(Error::LostVersionAtCargoToml)?.version = Inheritable::from(Some(new.to_string()));
            manifest.bin.clear();

            let manifest_new_content = toml::to_string_pretty(&manifest)?;

            let mut index = repo.index()?;
            let cargo_toml_entry = index.get_path(cargo_toml_path_relative, 0).ok_or(Error::LostCargoToml)?;
            index.add_frombuffer(&cargo_toml_entry, manifest_new_content.as_bytes())?;
            // TODO Add signature, if it was presented early
            commit.amend(Some("HEAD"), None, None, None, None, Some(&repo.find_tree(index.write_tree()?)?))?;

            // TODO Modify only version, not full file
            fs::write(&cargo_toml_path, manifest_new_content)?;

            index.add_path(cargo_toml_path_relative)?;
            index.write()?;

            println!("Patched");
        }
        Ok(ProcessResult::ManualChanged { previous, current }) => println!("Issue an INFO that the version has been changed manually and respects versioning rules: Previous: {previous}, Current: {current}"),
        Err(VersionUpdateTooWeak {
            expected_at_least,
            actual,
        }) => eprintln!("Issue a WARN that the version has been changed manually and does NOT comply with versioning rules: Actual: {actual}, Expected: >={expected_at_least}"),
    };

    Ok(())
}
