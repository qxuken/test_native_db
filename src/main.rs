use anyhow::Result;
use clap::{Parser, Subcommand};
use db::Artist;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

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
    All {
        #[arg(long, value_name = "DIR", default_value = "./data")]
        data_dir: PathBuf,

        #[arg(short, long, value_name = "FILE", default_value = "data.redb")]
        db: PathBuf,
    },
    FindById {
        #[arg(long, value_name = "DIR", default_value = "./data")]
        data_dir: PathBuf,

        #[arg(short, long, value_name = "FILE", default_value = "data.redb")]
        db: PathBuf,

        #[arg()]
        id: String,
    },
    ParseCsv {
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

    fn parse_csv(input: &Path, csv: &Path, db: &Path, data_dir: &Path) -> Result<()> {
        let db_path = data_dir.join(db);
        if !data_dir.is_dir() {
            fs::create_dir_all(data_dir)?;
        }
        let db_builder = db::create_db_builder()?;
        let db = db_builder.create(db_path)?;
        let t_rw = db.rw_transaction()?;
        let artists = Self::read_csv(input, csv)?
            .into_par_iter()
            .map(|artist| (artist, input, data_dir).try_into())
            .collect::<Result<Vec<Artist>>>()?;
        for artist in artists.into_iter() {
            t_rw.insert(artist)?;
        }
        t_rw.commit()?;
        Ok(())
    }

    fn all(data_dir: &Path, db: &Path) -> Result<Vec<Artist>> {
        let db_path = data_dir.join(db);
        if !data_dir.is_dir() {
            fs::create_dir_all(data_dir)?;
        }
        let db_builder = db::create_db_builder()?;
        let db = db_builder.create(db_path)?;
        let t_r = db.r_transaction()?;
        let artists = t_r.scan().primary()?.all().collect();

        Ok(artists)
    }
    fn find_by_id(data_dir: &Path, db: &Path, id: &str) -> Result<Artist> {
        let db_path = data_dir.join(db);
        if !data_dir.is_dir() {
            fs::create_dir_all(data_dir)?;
        }
        let db_builder = db::create_db_builder()?;
        let db = db_builder.create(db_path)?;
        let t_r = db.r_transaction()?;
        t_r.get()
            .primary(id)?
            .ok_or(anyhow::anyhow!("Artist not found"))
    }

    fn execute(&self) -> Result<()> {
        match &self {
            Self::All { data_dir, db } => {
                let artists = Self::all(data_dir, db)?;

                for artist in artists.iter() {
                    println!("{artist}");
                }
                Ok(())
            }
            Self::FindById { data_dir, db, id } => {
                let artist = Self::find_by_id(data_dir, db, id)?;

                println!("{artist}");
                Ok(())
            }
            Self::ParseCsv {
                data_dir,
                db,
                csv,
                input,
            } => Self::parse_csv(input, csv, db, data_dir),
        }
    }
}

fn main() {
    if let Err(e) = Cli::parse().execute() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
