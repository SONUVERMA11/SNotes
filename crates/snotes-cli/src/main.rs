//! S Notes CLI — export and batch operations
//!
//! Usage:
//!   snotes-cli export --format pdf --output ./out/ --input notebook.snotes
//!   snotes-cli export --format png --dpi 300 --output ./images/
//!   snotes-cli list --input notebook.snotes

use clap::{Parser, Subcommand};
use snotes_core::export::{ExportFormat, ExportOptions, Exporter};

#[derive(Parser)]
#[command(name = "snotes")]
#[command(about = "S Notes CLI — export and batch operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a notebook to PDF, PNG, SVG, or native format
    Export {
        /// Output format (pdf, png, svg, snotes)
        #[arg(short, long)]
        format: String,

        /// Output directory or file path
        #[arg(short, long)]
        output: String,

        /// Input .snotes file
        #[arg(short, long)]
        input: Option<String>,

        /// DPI for raster exports (default: 300)
        #[arg(long, default_value = "300")]
        dpi: u32,
    },

    /// List contents of a notebook
    List {
        /// Input .snotes file
        #[arg(short, long)]
        input: String,
    },

    /// Show info about a notebook
    Info {
        /// Input .snotes file
        #[arg(short, long)]
        input: String,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Export { format, output, input: _, dpi } => {
            let fmt = ExportFormat::from_str(&format).unwrap_or_else(|| {
                eprintln!("Unsupported format: {}", format);
                std::process::exit(1);
            });

            let options = ExportOptions {
                format: fmt,
                output_path: output.clone(),
                dpi,
                ..Default::default()
            };

            match Exporter::export(&options) {
                Ok(_) => println!("✓ Exported to {}", output),
                Err(e) => {
                    eprintln!("✗ Export failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List { input } => {
            println!("Listing contents of: {}", input);
            // TODO: implement listing
        }
        Commands::Info { input } => {
            println!("Notebook info: {}", input);
            // TODO: implement info display
        }
    }
}
