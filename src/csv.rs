use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct CsvArtist {
    pub id: u64,
    pub name: String,
    pub years: String,
    pub genre: String,
    pub nationality: String,
    pub bio: String,
    pub wikipedia: String,
    pub paintings: u64,
}

pub fn read_csv<R>(path: &Path) -> Result<Vec<R>, anyhow::Error>
where
    R: for<'a> Deserialize<'a>,
{
    if !path.is_file() {
        anyhow::bail!("`{}` is not a file", path.display())
    }
    let mut rdr = csv::Reader::from_path(path)?;
    rdr.deserialize()
        .collect::<Result<Vec<R>, csv::Error>>()
        .map_err(anyhow::Error::from)
}
