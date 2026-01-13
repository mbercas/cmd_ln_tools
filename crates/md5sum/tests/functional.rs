use assert_cmd::cargo::*;
use assert_fs::prelude::FileWriteStr;
use predicates::prelude::*;

mod md5sum_functional_tests {

    use super::*;

    #[test]
    fn read_text_input_files() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
        let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;

        file1.write_str("123456")?;
        file2.write_str("234567")?;

        let md5sum1 = "e10adc3949ba59abbe56e057f20f883e";
        let md5sum2 = "508df4cb2f4d8f80519256258cfb975f";

        let mut cmd = cargo_bin_cmd!("md5sum");

        let mut output = format!(
            "{}  {:?}\n{}  {:?}",
            md5sum1,
            file1.path(),
            md5sum2,
            file2.path()
        );
        output = output.replace("\"", "");

        cmd.arg(file1.path()).arg(file2.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(output));

        Ok(())
    }

    #[test]
    fn read_text_input_files_with_zero_flags() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
        let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;

        file1.write_str("123456")?;
        file2.write_str("234567")?;

        let md5sum1 = "e10adc3949ba59abbe56e057f20f883e";
        let md5sum2 = "508df4cb2f4d8f80519256258cfb975f";

        let mut cmd = cargo_bin_cmd!("md5sum");

        let mut output = format!(
            "{}  {:?}\0{}  {:?}",
            md5sum1,
            file1.path(),
            md5sum2,
            file2.path()
        );
        output = output.replace("\"", "");

        cmd.arg("--zero").arg(file1.path()).arg(file2.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(output));

        let mut cmd = cargo_bin_cmd!("md5sum");

        let mut output = format!(
            "{} *{:?}\0{} *{:?}",
            md5sum1,
            file1.path(),
            md5sum2,
            file2.path()
        );
        output = output.replace("\"", "");

        cmd.arg("--zero")
            .arg("-b")
            .arg(file1.path())
            .arg(file2.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(output));

        Ok(())
    }

    #[test]
    fn read_text_input_files_bsd_output() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
        let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;

        file1.write_str("123456")?;
        file2.write_str("234567")?;

        let md5sum1 = "e10adc3949ba59abbe56e057f20f883e";
        let md5sum2 = "508df4cb2f4d8f80519256258cfb975f";

        let mut cmd = cargo_bin_cmd!("md5sum");

        let mut output = format!(
            "MD5 ({:?}) = {}\nMD5 ({:?}) = {}",
            file1.path(),
            md5sum1,
            file2.path(),
            md5sum2
        );
        output = output.replace("\"", "");

        cmd.arg("--tag").arg(file1.path()).arg(file2.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(output));

        Ok(())
    }

    #[test]
    fn read_text_input_files_bsd_output_with_zero_flag() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = assert_fs::NamedTempFile::new("first_file.txt")?;
        let file2 = assert_fs::NamedTempFile::new("second_file.txt")?;

        file1.write_str("123456")?;
        file2.write_str("234567")?;

        let md5sum1 = "e10adc3949ba59abbe56e057f20f883e";
        let md5sum2 = "508df4cb2f4d8f80519256258cfb975f";

        let mut cmd = cargo_bin_cmd!("md5sum");

        let mut output = format!(
            "MD5 ({:?}) = {}\0MD5 ({:?}) = {}",
            file1.path(),
            md5sum1,
            file2.path(),
            md5sum2
        );
        output = output.replace("\"", "");

        cmd.arg("--tag")
            .arg("-z")
            .arg(file1.path())
            .arg(file2.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(output));

        Ok(())
    }
}
