use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(short = 't', long = "time", value_name = "SECONDS")]
    pub duration: Option<f64>,

    #[arg(short = 'f', long = "frames", value_name = "FRAMES")]
    pub frames: Option<f64>,
}
pub fn parse_command_line_options() -> Cli {
    Cli::parse()
}
