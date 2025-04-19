use anyhow::anyhow;
use std::{
    fs,
    path::{self, Path},
    process::{Command, Output},
};

use crate::environment::list_dir_glob;

fn invoke_command(cmd: &mut Command, verbose: bool) -> Result<Output, anyhow::Error> {
    if verbose {
        Ok(cmd.spawn()?.wait_with_output()?)
    } else {
        Ok(cmd.output()?)
    }
}

pub fn read_symbols<P>(
    options: &super::cli::CliOptions,
    lib_path: P,
) -> Result<Vec<String>, anyhow::Error>
where
    P: AsRef<Path>,
{
    let lib_path = lib_path.as_ref().to_string_lossy().to_string();
    if cfg!(target_os = "macos") {
        let output = invoke_command(
            Command::new(
                options
                    .symbol_provider_tool
                    .as_ref()
                    .ok_or(anyhow!("Symbol provider tool is not specified"))?,
            )
            .args(["-jgUA", &lib_path]),
            options.verbose,
        )?;

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
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        let output = invoke_command(
            Command::new(
                options
                    .archiver_tool
                    .as_ref()
                    .ok_or(anyhow!("Archiver tool is not specified"))?,
            )
            .args(["x", &lib_path])
            .current_dir(output),
            options.verbose,
        )?;

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
    output_path: O,
    symbol_list: &Vec<String>,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    O: AsRef<Path>,
{
    const SYMBOLS_LIST_FILE: &str = "symbols-list.txt";
    const PRELINKED_FILE: &str = "prelinked.o";

    if cfg!(target_os = "macos") {
        // write symbols list
        fs::write(
            extract_path.as_ref().join(SYMBOLS_LIST_FILE),
            symbol_list.join("\n"),
        )?;

        let objects = list_dir_glob(&extract_path, "*.o")?;

        // invoke ld
        let output = invoke_command(
            Command::new(
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
                    PRELINKED_FILE,
                ]
                .into_iter()
                .chain(objects.iter().map(|x| x.as_str())),
            )
            .current_dir(extract_path.as_ref()),
            options.verbose,
        )?;

        // check ld error
        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        // check file gennerated
        let prelinked_path = extract_path.as_ref().join(PRELINKED_FILE);
        if !prelinked_path.is_file() {
            return Err(anyhow!("Cannot link the prelinked object"));
        }

        if let Err(e) = fs::rename(prelinked_path, output_path.as_ref()) {
            return Err(anyhow!("Cannot move the prelinked object: {}", e));
        }

        Ok(())
    } else if cfg!(target_os = "linux") {
        // write symbols list
        fs::write(
            extract_path.as_ref().join(SYMBOLS_LIST_FILE),
            symbol_list.join("\n"),
        )?;

        let objects = list_dir_glob(&extract_path, "*.o")?;

        // invoke ld
        let output = invoke_command(
            Command::new(
                options
                    .linker_tool
                    .as_ref()
                    .ok_or(anyhow!("Linker tool is not specified"))?,
            )
            .args(
                [
                    "-r",
                    format!("--export-dynamic-symbol-list={}", SYMBOLS_LIST_FILE).as_str(),
                    "-o",
                    PRELINKED_FILE,
                ]
                .into_iter()
                .chain(objects.iter().map(|x| x.as_str())),
            )
            .current_dir(extract_path.as_ref()),
            options.verbose,
        )?;

        // check ld error
        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        // check file gennerated
        let prelinked_path = extract_path.as_ref().join(PRELINKED_FILE);
        if !prelinked_path.is_file() {
            return Err(anyhow!("Cannot link the prelinked object"));
        }

        // invoke objcopy
        let output = invoke_command(
            Command::new(
                options
                    .generator_tool
                    .as_ref()
                    .ok_or(anyhow!("Generator tool is not specified"))?,
            )
            .args(
                [
                    "-r",
                    format!("--keep-global-symbols={}", SYMBOLS_LIST_FILE).as_str(),
                    PRELINKED_FILE,
                ]
                .into_iter()
                .chain(objects.iter().map(|x| x.as_str())),
            )
            .current_dir(extract_path.as_ref()),
            options.verbose,
        )?;

        // check objcopy error
        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        // move
        if let Err(e) = fs::rename(prelinked_path, output_path.as_ref()) {
            return Err(anyhow!("Cannot move the prelinked object: {}", e));
        }

        Ok(())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}

pub fn generate_static_lib_from_all_object<P, O>(
    options: &super::cli::CliOptions,
    lib_path: P,
    output: O,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    O: AsRef<Path>,
{
    let output_path = path::absolute(output)?;

    if cfg!(target_os = "macos") {
        let objects = list_dir_glob(&lib_path, "*.o")?;

        let output = invoke_command(
            Command::new(
                options
                    .generator_tool
                    .as_ref()
                    .ok_or(anyhow!("Generator tool is not specified"))?,
            )
            .args(
                [
                    "-static",
                    "-o",
                    output_path.to_string_lossy().to_string().as_str(),
                ]
                .into_iter()
                .chain(objects.iter().map(|x| x.as_str())),
            )
            .current_dir(&lib_path),
            options.verbose,
        )?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        if !output_path.is_file() {
            return Err(anyhow!("Cannot generate static lib"));
        }

        Ok(())
    } else if cfg!(target_os = "linux") {
        let objects = list_dir_glob(&lib_path, "*.o")?;

        let output = invoke_command(
            Command::new(
                options
                    .archiver_tool
                    .as_ref()
                    .ok_or(anyhow!("Archiver tool is not specified"))?,
            )
            .args(
                ["rs", output_path.to_string_lossy().to_string().as_str()]
                    .into_iter()
                    .chain(objects.iter().map(|x| x.as_str())),
            )
            .current_dir(&lib_path),
            options.verbose,
        )?;

        if !output.status.success() && !options.force {
            return Err(anyhow!(String::from_utf8(output.stderr)?));
        }

        if !output_path.is_file() {
            return Err(anyhow!("Cannot generate static lib"));
        }

        Ok(())
    } else {
        Err(anyhow!("Unsupported OS"))
    }
}
