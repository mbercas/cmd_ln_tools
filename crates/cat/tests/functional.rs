use assert_cmd::cargo::*;
use assert_fs::prelude::FileWriteStr;
use predicates::prelude::*;

#[test]
fn dump_file_contents() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("first_file.txt")?;
    file.write_str("Line 1\nLine 2\nLine3")?;

    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Line 1"));

    Ok(())
}

#[test]
fn add_line_numbers() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("first_file.txt")?;
    file.write_str("Line 1\nLine 2\nLine3")?;

    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg("-n").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1 Line 1\n2 Line 2"));

    Ok(())
}

#[test]
fn concatenate_file_contents() -> Result<(), Box<dyn std::error::Error>> {
    let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
    let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;
    file1.write_str("Line 1\nLine 2")?;
    file2.write_str("Line 3\nLine 4")?;

    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg(file1.path()).arg(file2.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Line 1\nLine 2\nLine 3\nLine 4"));
    // .stdout(predicate::str::contains("1 Line 1\n2 Line 2\nLine 3\nLine 4"));

    Ok(())
}

#[test]
fn concatenate_file_contents_and_number() -> Result<(), Box<dyn std::error::Error>> {
    let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
    let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;
    file1.write_str("Line 1\nLine 2")?;
    file2.write_str("Line 3\nLine 4")?;

    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg("-n").arg(file1.path()).arg(file2.path());
    cmd.assert().success().stdout(predicate::str::contains(
        "1 Line 1\n2 Line 2\n3 Line 3\n4 Line 4",
    ));

    Ok(())
}

#[test]
fn add_eol_characterr() -> Result<(), Box<dyn std::error::Error>> {
    let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
    let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;
    file1.write_str("Line 1\nLine 2\n\n")?;
    file2.write_str("Line 3\nLine 4")?;

    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg("-nE").arg(file1.path()).arg(file2.path());
    cmd.assert().success().stdout(predicate::str::contains(
        "1 Line 1$\n2 Line 2$\n3 $\n4 $\n5 Line 3$\n6 Line 4$",
    ));

    let mut cmd = cargo_bin_cmd!("cat");
    // -b overrides -n
    cmd.arg("-nbE").arg(file1.path()).arg(file2.path());
    cmd.assert().success().stdout(predicate::str::contains(
        "1 Line 1$\n2 Line 2$\n$\n$\n3 Line 3$\n4 Line 4$",
    ));

    let mut cmd = cargo_bin_cmd!("cat");
    // -bsE
    cmd.arg("-nbsE").arg(file1.path()).arg(file2.path());
    cmd.assert().success().stdout(predicate::str::contains(
        "1 Line 1$\n2 Line 2$\n$\n3 Line 3$\n4 Line 4$",
    ));

    Ok(())
}

#[test]
fn read_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("cat");

    cmd.arg("-n").arg("-");
    cmd.write_stdin("Line 1\nLine 2");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1 Line 1\n2 Line 2"));

    Ok(())
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("cat");

    let bad_file = "kk.txt";
    cmd.arg(bad_file);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));

    Ok(())
}
