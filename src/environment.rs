use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use super::cli::CliOptions;
use anyhow::anyhow;
use glob_match::glob_match;

pub fn find_all_tool_from_env(options: &mut CliOptions) -> Result<(), anyhow::Error> {
    if options.linker_tool == None {
        options.linker_tool = Some(
            find_tool_from_env(if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                "ld"
            } else {
                ""
            })?
            .to_string_lossy()
            .to_string(),
        );
    }

    if options.archiver_tool == None {
        options.archiver_tool = Some(
            find_tool_from_env(if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                "ar"
            } else {
                ""
            })?
            .to_string_lossy()
            .to_string(),
        );
    }

    if options.generator_tool == None {
        options.generator_tool = Some(
            find_tool_from_env(if cfg!(target_os = "macos") {
                "libtool"
            } else if cfg!(target_os = "linux") {
                "objcopy"
            } else {
                ""
            })?
            .to_string_lossy()
            .to_string(),
        );
    }

    if options.symbol_provider_tool == None {
        options.symbol_provider_tool = Some(
            find_tool_from_env(if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                "nm"
            } else {
                ""
            })?
            .to_string_lossy()
            .to_string(),
        );
    }

    Ok(())
}

pub fn find_tool_from_env<P>(name: P) -> Result<PathBuf, anyhow::Error>
where
    P: AsRef<Path>,
{
    let env = env::var_os("PATH").ok_or(anyhow!("Cannot read PATH environment variable"))?;
    let paths = env::split_paths(&env);

    for path in paths {
        let file_name = path.join(&name);
        if file_name.is_file() {
            return Ok(file_name);
        }
    }

    Err(anyhow!(
        "Failed to find {}",
        name.as_ref().to_string_lossy()
    ))
}

pub fn print_with_prefix<P, V>(writer: &mut impl Write, prefix: P, value: V)
where
    P: AsRef<str>,
    V: AsRef<str>,
{
    let prefix = format!("{}: ", prefix.as_ref());
    let indent = std::iter::repeat(' ')
        .take(prefix.len())
        .collect::<String>();
    let mut is_first_line = true;

    let processed_value = value
        .as_ref()
        .to_string()
        .split('\n')
        .map(|x| {
            format!(
                "{}{}\n",
                if is_first_line {
                    is_first_line = false;
                    &prefix
                } else {
                    &indent
                },
                x
            )
        })
        .collect::<String>();

    _ = writer.write_all(processed_value.as_bytes());
}

pub fn list_dir_glob<P, G>(dir: P, glob: G) -> Result<Vec<String>, anyhow::Error>
where
    P: AsRef<Path>,
    G: AsRef<str>,
{
    let file_names: Result<Vec<String>, io::Error> = fs::read_dir(dir)?
        .map(|x| x.map(|x| x.file_name().to_string_lossy().to_string()))
        .collect();

    Ok(file_names?
        .into_iter()
        .filter(|x| glob_match(glob.as_ref(), x.as_str()))
        .collect())
}
