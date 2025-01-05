use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub(crate) global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    // /// Verbosity level (can be specified multiple times)
    // #[clap(long, short, global = true, parse(from_occurrences))]
    // verbose: usize,
    // //... other global options
    #[clap(
        long = "physics-rate",
        short = 'p',
        global = true,
        default_value_t = 128f64
    )]
    pub(crate) physics_rate: f64, // Hz

    #[clap(long, short, global = true, default_value_t = 0)]
    pub(crate) seed: u64,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Help message for benchmark.
    Benchmark {
        #[arg(
            short = 't',
            long = "time",
            value_name = "SECONDS",
            help = "Run to duration"
        )]
        duration: Option<f64>,

        #[arg(
            short = 'f',
            long = "frames",
            value_name = "FRAMES",
            help = "Run to number of fixed update frames"
        )]
        frames: Option<f64>,
        // (can #[clap(flatten)] other argument structs here)
    },
    // ...other commands (can #[clap(flatten)] other enum variants here)
}

pub fn parse_command_line_options() -> Cli {
    Cli::parse()
}
