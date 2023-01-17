use anek::dtypes;
use assert_cmd::prelude::*;
use assert_fs::fixture::TempDir;
use assert_fs::prelude::*;
use rstest::rstest;
use std::process::Command;

pub type Error = Box<dyn std::error::Error>;

/// Check new directory is made successfully
#[rstest]
fn make_new_anek_config() -> Result<(), Error> {
    let wd = TempDir::new()?;
    let mut child = Command::cargo_bin("anek")
        .expect("No test binary")
        .arg("-q")
        .arg("new")
        .arg(wd.path())
        .spawn()
        .expect("Cannot run binary");
    child.wait()?;

    let anekdir = dtypes::AnekDirectory::from(&wd.path().to_path_buf());
    for dirtype in dtypes::anekdirtype_iter() {
        assert!(anekdir.get_directory(&dirtype).exists());
    }
    Ok(())
}

/// Check new directory is made successfully with variables
#[rstest(
    variables,
    case(vec!["var1", "var2", "var3"]),
    case(vec!["var_name_1", "var_name_2"]),
    case(vec!["var-name-1"]),
)]
fn make_new_anek_config_with_variables(variables: Vec<&str>) -> Result<(), Error> {
    let wd = TempDir::new()?;
    let mut child = Command::cargo_bin("anek")
        .expect("No test binary")
        .arg("-q")
        .arg("new")
        .arg("--variables")
        .arg(variables.join(","))
        .arg(wd.path())
        .spawn()
        .expect("Cannot run binary");
    child.wait()?;

    let anekdir = dtypes::AnekDirectory::from(&wd.path().to_path_buf());
    for dirtype in dtypes::anekdirtype_iter() {
        assert!(anekdir.get_directory(&dirtype).exists());
    }
    for var in variables {
        assert!(anekdir
            .get_file(&dtypes::AnekDirectoryType::Variables, &var)
            .exists());
    }
    Ok(())
}
