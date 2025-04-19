use clap::Parser;

#[derive(Parser)]
#[command(version)]
#[command(about = "A script to simplify the process of generating Single-Object-Prelink static library", long_about = None)]
pub struct CliOptions {
    #[clap(required = true)]
    pub files: Vec<String>,
    #[arg(
        short = 'o',
        long = "output",
        help = "Specify the path of output library"
    )]
    pub output: Option<String>,

    #[arg(
        short = 's',
        long = "symbol-name",
        help = "Specify which symbols to preserve, wildcards are supported"
    )]
    pub symbols: Vec<String>,

    #[arg(
        short = 'l',
        long = "symbol-list",
        help = "Specify a text file to read symbols from, one symbol per line",
        name = "SYMBOLS_FILE"
    )]
    pub symbol_lists: Option<String>,

    #[arg(
        short = 'P',
        long = "symbol-provider",
        name = "PROVIDER_TOOL",
        help = "Path of a tool of resolving symbols in static lib.\n[macOS, Linux] It should be `nm`"
    )]
    pub symbol_provider_tool: Option<String>,

    #[arg(
        short = 'L',
        long = "linker",
        help = "Path of linker.\n[macOS, Linux] It should be `ld`"
    )]
    pub linker_tool: Option<String>,

    #[arg(short = 'A', long = "archiver", help = "Path of `ar` tool")]
    pub archiver_tool: Option<String>,

    #[arg(
        short = 'G',
        long = "generater",
        help = "Path of a generator tool.\n[macOS] It should be `libtool`\n[Linux] It should be `objcopy`"
    )]
    pub generator_tool: Option<String>,

    #[arg(
        short = 'F',
        long = "force",
        help = "Ignore all error when invoking tools"
    )]
    pub force: bool,

    #[arg(short = 'v', long = "verbose", help = "Display all the output")]
    pub verbose: bool,
}
