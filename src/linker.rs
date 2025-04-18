use anyhow::anyhow;
use std::{fs, path::Path, process::Command};

use crate::environment::list_dir_glob;

pub fn read_symbols<P>(
    options: &super::cli::CliOptions,
    lib_path: P,
) -> Result<Vec<String>, anyhow::Error>
where
    P: AsRef<Path>,
{
    let lib_path = lib_path.as_ref().to_string_lossy().to_string();
    if cfg!(target_os = "macos") {
        let output = Command::new(
            options
                .symbol_provider_tool
                .as_ref()
                .ok_or(anyhow!("Symbol provider tool is not specified"))?,
        )
        .args(["-jgUA", &lib_path])
        .output()?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        let output = String::from_utf8(output.stdout)?;

        Ok(output
            .split('\n')
            .filter_map(|line| line.split(':').last())
            .map(|x| x.trim().to_owned())
            .filter(|x| !x.is_empty())
            .collect())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}

pub fn extract_static_lib<P, O>(
    options: &super::cli::CliOptions,
    lib_path: P,
    output: O,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    O: AsRef<Path>,
{
    let lib_path = lib_path.as_ref().to_string_lossy().to_string();
    if cfg!(target_os = "macos") {
        let output = Command::new(
            options
                .archiver_tool
                .as_ref()
                .ok_or(anyhow!("Archiver tool is not specified"))?,
        )
        .args(["x", &lib_path])
        .current_dir(output)
        .output()?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        Ok(())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}

pub fn link_static_lib<P, O>(
    options: &super::cli::CliOptions,
    extract_path: P,
    output_name: O,
    symbol_list: &Vec<String>,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    O: AsRef<str>,
{
    if cfg!(target_os = "macos") {
        const SYMBOLS_LIST_FILE: &str = "symbols-list.txt";
        // write symbols list
        fs::write(
            extract_path.as_ref().join(SYMBOLS_LIST_FILE),
            symbol_list.join("\n"),
        )?;

        // invoke ld
        let objects = list_dir_glob(&extract_path, "*.o")?;

        let output = Command::new(
            options
                .linker_tool
                .as_ref()
                .ok_or(anyhow!("Linker tool is not specified"))?,
        )
        .args(
            [
                "-r",
                "-exported_symbols_list",
                SYMBOLS_LIST_FILE,
                "-o",
                output_name.as_ref(),
            ]
            .into_iter()
            .chain(objects.iter().map(|x| x.as_str())),
        )
        .current_dir(extract_path.as_ref())
        .output()?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        Ok(())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}

pub fn generate_static_lib_from_all<P, O>(
    options: &super::cli::CliOptions,
    lib_path: P,
    output: O,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    O: AsRef<Path>,
{
    if cfg!(target_os = "macos") {
        let output = output.as_ref().to_string_lossy().to_string();
        let objects = list_dir_glob(&lib_path, "*.o")?;

        let output = Command::new(
            options
                .generator_tool
                .as_ref()
                .ok_or(anyhow!("Generator tool is not specified"))?,
        )
        .args(
            ["-static", "-o", &output]
                .into_iter()
                .chain(objects.iter().map(|x| x.as_str())),
        )
        .current_dir(&lib_path)
        .spawn()?
        .wait_with_output()?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        Ok(())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}
