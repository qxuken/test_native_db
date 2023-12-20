use anyhow::Result;
use clap::{Parser, Subcommand};
use db::Artist;
use std::path::{Path, PathBuf};

mod csv;
mod db;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn execute(&self) -> Result<()> {
        self.command.execute()
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    Read {
        #[arg(short, long, value_name = "FILE", default_value = "data.csv")]
        csv: PathBuf,

        #[arg(value_name = "DIR", default_value = "./input")]
        input: PathBuf,
    },
    Save {
        #[arg(long, value_name = "DIR", default_value = "./data")]
        data_dir: PathBuf,

        #[arg(short, long, value_name = "FILE", default_value = "data.redb")]
        db: PathBuf,

        #[arg(short, long, value_name = "FILE", default_value = "data.csv")]
        csv: PathBuf,

        #[arg(value_name = "DIR", default_value = "./input")]
        input: PathBuf,
    },
}

impl Commands {
    fn read_csv(input: &Path, csv: &Path) -> Result<Vec<csv::CsvArtist>> {
        let csv_path = input.join(csv);
        println!("Reading from {csv_path:?}");
        csv::read_csv::<csv::CsvArtist>(&csv_path)
    }

    fn save_cmd(input: &Path, csv: &Path, db: &Path, data_dir: &Path) -> Result<()> {
        let db_path = data_dir.join(db);
        let db_builder = db::create_db_builder()?;
        let db = db_builder.create(db_path)?;
        let t_rw = db.rw_transaction()?;
        let artists = Self::read_csv(input, csv)?;
        for artist in artists.into_iter() {
            let artist: Artist = (artist, input, data_dir).try_into()?;
            t_rw.insert(artist)?;
        }
        t_rw.commit()?;
        Ok(())
    }

    fn execute(&self) -> Result<()> {
        match &self {
            Self::Read { input, csv } => {
                let artists = Self::read_csv(input, csv)?;

                for artist in artists.iter() {
                    println!("{artist:?}");
                }
                Ok(())
            }
            Self::Save {
                data_dir,
                db,
                csv,
                input,
            } => Self::save_cmd(input, csv, db, data_dir),
        }
    }
}

fn main() {
    if let Err(e) = Cli::parse().execute() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
