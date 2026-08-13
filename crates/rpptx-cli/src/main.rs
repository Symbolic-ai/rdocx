//! Command-line access to the public `rpptx` facade.

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "rpptx", version, about = "CLI tool for PPTX files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print presentation structure and metadata
    Inspect {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Extract slide text in presentation order
    Text { file: PathBuf },
    /// Convert a presentation to deterministic PDF or PNG output
    Convert {
        file: PathBuf,
        #[arg(long, short = 't')]
        to: String,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        #[arg(long, default_value = "150")]
        dpi: f64,
    },
    /// Compare slide text using a longest-common-subsequence diff
    Diff { file_a: PathBuf, file_b: PathBuf },
    /// Replace literal presentation text while retaining run formatting
    Replace {
        file: PathBuf,
        #[arg(long, short = 'p')]
        placeholder: String,
        #[arg(long, short = 'v')]
        value: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Validate package and PresentationML invariants
    Validate { file: PathBuf },
    /// Render selected slides to deterministic PNG files
    Render {
        file: PathBuf,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        #[arg(long, default_value = "150")]
        dpi: f64,
        #[arg(long)]
        slide: Option<String>,
    },
    /// Render slide one as a proportional 320-pixel-wide PNG
    Thumbnail {
        file: PathBuf,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Print each slide title and recursive paragraph outline
    Outline { file: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    if let Command::Validate { file } = &cli.command {
        match commands::validate(file) {
            Ok(true) => return,
            Ok(false) => process::exit(1),
            Err(error) => {
                eprintln!("Error: {error}");
                process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Command::Inspect { file, json } => commands::inspect(&file, json),
        Command::Text { file } => commands::text(&file),
        Command::Convert {
            file,
            to,
            output,
            dpi,
        } => commands::convert(&file, &to, output.as_deref(), dpi),
        Command::Diff { file_a, file_b } => commands::diff(&file_a, &file_b),
        Command::Replace {
            file,
            placeholder,
            value,
            output,
        } => commands::replace(&file, &placeholder, &value, &output),
        Command::Validate { .. } => unreachable!("validate is dispatched above"),
        Command::Render {
            file,
            output,
            dpi,
            slide,
        } => commands::render(&file, output.as_deref(), dpi, slide.as_deref()),
        Command::Thumbnail { file, output } => commands::thumbnail(&file, output.as_deref()),
        Command::Outline { file } => commands::outline(&file),
    };
    if let Err(error) = result {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
