mod cli;
mod environment;
mod linker;

use std::{
    fs::{self, read_to_string},
    io::stderr,
    path::{self, Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use clap::Parser;
use cli::CliOptions;
use environment::{find_all_tool_from_env, print_with_prefix};
use fatal::fatal;
use glob_match::glob_match;
use linker::{
    extract_static_lib, generate_static_lib_from_all_object, link_static_lib, read_symbols,
};

const TEMP_ROOT: &str = ".soplink.tmp";

#[derive(Debug)]
struct StaticLib {
    path: String,
    name: String,
    symbols: Vec<String>,
}

fn load_symbol_list(options: &mut CliOptions) -> Result<(), anyhow::Error> {
    if options.symbol_lists == None {
        return Ok(());
    }
    read_to_string(options.symbol_lists.as_ref().unwrap())?
        .split('\n')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty() && !x.starts_with('#'))
        .into_iter()
        .for_each(|x| options.symbols.push(x.to_string()));

    Ok(())
}

fn create_workspace() -> Result<(), anyhow::Error> {
    if fs::exists(TEMP_ROOT)? {
        fs::remove_dir_all(TEMP_ROOT)?;
    }

    fs::create_dir(TEMP_ROOT)?;
    Ok(())
}

fn create_extract_dir(lib_name: &str) -> Result<PathBuf, anyhow::Error> {
    let path = PathBuf::from_str(TEMP_ROOT)?.join(lib_name);
    if fs::exists(&path)? {
        fs::remove_dir_all(&path)?;
    }

    fs::create_dir(&path)?;

    Ok(path)
}

fn release_workspace() -> Result<(), anyhow::Error> {
    fs::remove_dir_all(TEMP_ROOT)?;
    Ok(())
}

fn resolve_lib<L>(options: &CliOptions, lib_path: L) -> Result<StaticLib, anyhow::Error>
where
    L: AsRef<str>,
{
    let lib_path = lib_path.as_ref();
    Ok(StaticLib {
        path: lib_path.to_owned(),
        name: PathBuf::from_str(lib_path.as_ref())?
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        symbols: read_symbols(&options, lib_path)?
            .into_iter()
            .filter(|x| options.symbols.iter().any(|y| glob_match(&y, x)))
            .collect(),
    })
}

fn link_lib(options: &CliOptions, lib: &StaticLib) -> Result<(), anyhow::Error> {
    let extract_path = create_extract_dir(&lib.name)?;
    extract_static_lib(options, &lib.path, &extract_path)?;

    let output_path = PathBuf::from_str(TEMP_ROOT)?.join(format!("{}.prelink.o", &lib.name));
    link_static_lib(options, &extract_path, output_path, &lib.symbols)?;

    Ok(())
}

fn run(options: &mut CliOptions) -> Result<(), anyhow::Error> {
    // find all undefined toolls
    find_all_tool_from_env(options)?;

    // load symbol list if have
    load_symbol_list(options)?;

    // begin
    create_workspace()?;

    // resolve libs
    let mut libs: Vec<StaticLib> = vec![];
    for file_path in options.files.iter().as_ref() {
        match resolve_lib(&options, file_path) {
            Ok(x) => libs.push(x),
            Err(e) => {
                print_with_prefix(
                    &mut stderr().lock(),
                    PathBuf::from_str(file_path.as_str())?
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                    e.to_string(),
                );
                return Err(anyhow!("Failed to resolve one or more library"));
            }
        }
    }

    // link libs
    for lib in libs.iter() {
        if let Err(e) = link_lib(&options, lib) {
            print_with_prefix(&mut stderr().lock(), &lib.name, e.to_string());
            return Err(anyhow!("Failed to link one or more library"));
        }
    }

    // generate static lib
    generate_static_lib_from_all_object(
        &options,
        TEMP_ROOT,
        path::absolute(PathBuf::from_str(
            options
                .output
                .as_ref()
                .map_or("soplink-out.a", |x| x.as_str()),
        )?)?,
    )?;

    release_workspace()?;

    Ok(())
}

fn main() {
    // parse options
    let mut options = CliOptions::parse();

    // check os
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        // supported
    } else {
        // on a not supported OS
        fatal!("Current operation system is not supported");
    }

    if let Err(e) = run(&mut options) {
        fatal!("soplink: {}", e);
    }
}
