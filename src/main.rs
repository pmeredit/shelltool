use clap::{App, Arg, SubCommand};
use regex::Regex;
use std::{
    fmt::Debug,
    fs::{copy, hard_link, rename},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Regex(regex::Error),
    Convert(std::convert::Infallible),
    CustomError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Self::Regex(err)
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(err: std::convert::Infallible) -> Self {
        Self::Convert(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::CustomError(err.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let matches = App::new("shelltool")
        .version("1.0")
        .author("Patrick Meredith <pmeredit@gmail.com>")
        .about("a better version of copy and move")
        .subcommand(
            SubCommand::with_name("cp")
                .about("copy files and directories")
                .arg(Arg::with_name("src_pattern").required(true))
                .arg(Arg::with_name("dest_pattern").required(true)),
        )
        .subcommand(
            SubCommand::with_name("mv")
                .about("rename files and directories")
                .arg(Arg::with_name("src_pattern").required(true))
                .arg(Arg::with_name("dest_pattern").required(true)),
        )
        .subcommand(
            SubCommand::with_name("lns")
                .about("soft link files and directories")
                .arg(Arg::with_name("src_pattern").required(true))
                .arg(Arg::with_name("dest_pattern").required(true)),
        )
        .subcommand(
            SubCommand::with_name("lnh")
                .about("hard link files and directories")
                .arg(Arg::with_name("src_pattern").required(true))
                .arg(Arg::with_name("dest_pattern").required(true)),
        )
        .get_matches();

    let (func, src, dst): (
        fn(PathBuf, PathBuf) -> std::result::Result<(), Error>,
        PathBuf,
        PathBuf,
    ) = if let Some(matches) = matches.subcommand_matches("cp") {
        (
            copy_adapter,
            PathBuf::from_str(matches.value_of("src_pattern").unwrap())?,
            PathBuf::from_str(matches.value_of("dest_pattern").unwrap())?,
        )
    } else if let Some(matches) = matches.subcommand_matches("mv") {
        (
            rename_adapter,
            PathBuf::from_str(matches.value_of("src_pattern").unwrap())?,
            PathBuf::from_str(matches.value_of("dest_pattern").unwrap())?,
        )
    } else if let Some(matches) = matches.subcommand_matches("lns") {
        (
            symlink_adapter,
            PathBuf::from_str(matches.value_of("src_pattern").unwrap())?,
            PathBuf::from_str(matches.value_of("dest_pattern").unwrap())?,
        )
    } else {
        (
            hard_link_adapter,
            PathBuf::from_str(matches.value_of("src_pattern").unwrap())?,
            PathBuf::from_str(matches.value_of("dest_pattern").unwrap())?,
        )
    };
    two_arg_helper(copy_adapter, src, dst)
}

fn rename_adapter(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    rename(src, dst)?;
    Ok(())
}

fn copy_adapter(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    copy(src, dst)?;
    Ok(())
}

fn symlink_adapter(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    symlink(src, dst)?;
    Ok(())
}

fn hard_link_adapter(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    hard_link(src, dst)?;
    Ok(())
}

fn two_arg_helper(
    func: impl Fn(PathBuf, PathBuf) -> Result<()>,
    from: PathBuf,
    to: PathBuf,
) -> Result<()> {
    let (src_file_name, dst_file_name) = (
        from.file_name()
            .ok_or("no filename in source pattern".to_string())?
            .to_string_lossy(),
        to.file_name()
            .ok_or("no filename in destination pattern".to_string())?
            .to_string_lossy(),
    );
    let (from_prefix, from_pattern) = (from.parent(), Regex::new(src_file_name.as_ref())?);
    let to_prefix = to.parent();

    let dir = from_prefix.unwrap_or(Path::new("."));
    for file in dir.read_dir()? {
        if let Ok(file) = file {
            let path = file.path().to_string_lossy().into_owned();
            //let dst = from_pattern.replace_all(&path, dst_file_name);
            let dst = dst_file_name.clone();
            if dst != path {
                func(PathBuf::from_str(&path)?, PathBuf::from_str(&dst)?)?;
            }
        }
    }
    Ok(())
}
