use std::env;
use std::io::{BufRead, Write};

use clap::Parser;
use log::{error, info};

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(
        long,
        short,
        help = "File to write UIDs to (if not specified, UIDs will be printed to stdout)"
    )]
    file: Option<std::path::PathBuf>,
    #[arg(
        long,
        short,
        default_value_t = false,
        help = "Whether to create a new file (overwrites existing file if it exists)"
    )]
    new: bool,
    #[arg(
        long,
        short,
        default_value_t = false,
        help = "Whether to resume from an existing file (uploads file contents to server if enabled and file exists)"
    )]
    resume: bool,
}

#[derive(Debug)]
pub struct State {
    pub file: Option<std::fs::File>,
    pub key: String,
}

impl State {
    pub fn init() -> Self {
        println!("Pordisto - A tool for monitoring smart card readers and logging UIDs");

        let cli = Cli::parse();

        let file = cli.file.as_ref().map(|file| {
            if cli.new {
                let mut f = std::fs::File::create(file).expect("Failed to create new file");
                info!("Created new file: {:?}", f);
                writeln!(f, "UIDS").expect("Failed to write header to new file");
                f
            } else {
                let f = std::fs::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(file)
                    .expect("Failed to open file");
                // read header
                let mut buf = String::new();
                std::io::BufReader::new(&f)
                    .read_line(&mut buf)
                    .expect("Failed to read header from file");
                if buf.trim() != "UIDS" {
                    error!(
                        "Invalid file format: expected header \"UIDS\", got \"{}\"",
                        buf.trim()
                    );
                    panic!("Invalid file format: expected header \"UIDS\"");
                } else {
                    info!("Opened existing file: {:?}", f);
                    f
                }
            }
        });

        info!("Getting key");
        let key = env::var("PORDISTO_KEY")
            .inspect(|v| println!("Using key from env var: {}", v))
            .unwrap_or_else(|_| {
                print!("Enter key (or use \"PORDISTO_KEY\" env var): ");
                std::io::stdout().flush().expect("Failed to flush stdout");
                let mut buf = String::new();
                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("Failed to read key from stdin");
                buf
            });

        if let Some(f) = &cli.file
            && cli.resume
        {
            if key.trim().is_empty() {
                error!("Resume enabled but no key provided");
                panic!("Resume enabled but no key provided");
            } else {
                info!("Resume enabled with key: {}", key.trim());
            }

            print!("Resume {} to server? (y/N): ", f.display());
            std::io::stdout().flush().expect("Failed to flush stdout");
            let mut buf = String::new();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read key from stdin");
            if buf.trim().to_lowercase() == "y" {
                let contents = std::fs::read_to_string(f).expect("Failed to read file for upload");
                // TODO: upload contents to server
                info!("Uploaded file contents to server: {}", contents);
            } else {
                info!("User cancelled upload");
            }
        } else if cli.resume {
            error!("Resume enabled but no file provided");
            panic!("Resume enabled but no file provided");
        }

        Self { file, key }
    }
}
