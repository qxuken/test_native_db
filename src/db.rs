use crate::csv::CsvArtist;
use image::{imageops::FilterType, io::Reader as ImageReader};
use native_db::*;
use native_model::{native_model, Model};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Img {
    path: PathBuf,
    width: u32,
    height: u32,
}

impl Display for Img {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(w {:4}  h {:4})", self.width, self.height)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paintings {
    id: Uuid,
    full: Img,
    cropped: Img,
    thumbnail: Img,
}

impl Display for Paintings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ->  full {}  cropped {}  thumbnail {}",
            self.id, self.full, self.cropped, self.thumbnail
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(id))]
pub struct Artist {
    id: Uuid,
    name: String,
    born: String,
    died: String,
    genre: String,
    nationality: String,
    bio: String,
    wikipedia: String,
    paintings: Vec<Paintings>,
}

impl Display for Artist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} -> {}  {} - {} [{}] [{}]",
            self.id, self.name, self.born, self.died, self.nationality, self.genre
        )?;
        writeln!(f, "wikipedia: {}", self.wikipedia)?;
        writeln!(f, "{}", self.bio)?;
        for painting in &self.paintings {
            writeln!(f, "  p {}", painting)?;
        }
        Ok(())
    }
}

impl TryFrom<(CsvArtist, &Path, &Path)> for Artist {
    type Error = anyhow::Error;

    fn try_from(
        (csv_artist, input_path, data_path): (CsvArtist, &Path, &Path),
    ) -> Result<Self, Self::Error> {
        let img_dest_path = data_path.join("img");
        if !img_dest_path.is_dir() {
            fs::create_dir_all(&img_dest_path)
                .map_err(|e| anyhow::anyhow!("Failed to create img directory: {}", e))?;
        }
        let img_input_path = input_path.join("images").join(
            csv_artist
                .name
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("_"),
        );
        if !img_input_path.is_dir() {
            anyhow::bail!("Failed to find image directory: {:?}", img_input_path)
        }
        let paintings: Vec<Paintings> = fs::read_dir(img_input_path)?
            .par_bridge()
            .map(|entry| {
                entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))
            })
            .collect::<Result<Vec<DirEntry>, anyhow::Error>>()?
            .par_iter()
            .map(|image| {
                let painting_id = Uuid::new_v7(Timestamp::now(NoContext));
                let painting_dir = img_dest_path.join(painting_id.to_string());
                if painting_dir.is_dir() {
                    anyhow::bail!("Painting directory already exists")
                }
                if let Err(e) = fs::create_dir(&painting_dir) {
                    println!("{csv_artist:?}");
                    println!("{image:?}");
                    println!("{painting_dir:?}");
                    anyhow::bail!("Failed to create painting directory: {}", e)
                }

                let image_path = image.path();
                let full_image = ImageReader::open(&image_path)
                    .map_err(anyhow::Error::from)
                    .and_then(|i| i.decode().map_err(anyhow::Error::from))
                    .and_then(|image_view| {
                        let img = Img {
                            path: PathBuf::from(painting_id.to_string()).join("full.jpg"),
                            width: image_view.width(),
                            height: image_view.height(),
                        };
                        fs::copy(&image_path, img_dest_path.join(&img.path))
                            .map_err(anyhow::Error::from)
                            .map(|_| img)
                    });
                if let Err(e) = full_image {
                    println!("{csv_artist:?}");
                    println!("{image:?}");
                    println!("{painting_dir:?}");
                    anyhow::bail!("Failed to copy full image: {}", e)
                }
                let thumbnail_img = ImageReader::open(&image_path)
                    .map_err(anyhow::Error::from)
                    .and_then(|i| i.decode().map_err(anyhow::Error::from))
                    .map(|image_view| image_view.resize(150, 150, FilterType::Lanczos3))
                    .and_then(|image_view| {
                        let img = Img {
                            path: PathBuf::from(painting_id.to_string()).join("thumbnail.jpg"),
                            width: image_view.width(),
                            height: image_view.height(),
                        };

                        image_view
                            .save(img_dest_path.join(&img.path))
                            .map_err(anyhow::Error::from)
                            .map(|_| img)
                    });

                if let Err(e) = thumbnail_img {
                    println!("{csv_artist:?}");
                    println!("{image:?}");
                    println!("{painting_dir:?}");
                    anyhow::bail!("Failed to copy thumbnail image: {}", e)
                }
                let cropped_img = ImageReader::open(&image_path)
                    .map_err(anyhow::Error::from)
                    .and_then(|i| i.decode().map_err(anyhow::Error::from))
                    .map(|image_view| image_view.resize(600, 600, FilterType::Lanczos3))
                    .and_then(|image_view| {
                        let img = Img {
                            path: PathBuf::from(painting_id.to_string()).join("cropped.jpg"),
                            width: image_view.width(),
                            height: image_view.height(),
                        };

                        image_view
                            .save(img_dest_path.join(&img.path))
                            .map_err(anyhow::Error::from)
                            .map(|_| img)
                    });
                if let Err(e) = cropped_img {
                    println!("{csv_artist:?}");
                    println!("{image:?}");
                    println!("{painting_dir:?}");
                    anyhow::bail!("Failed to copy cropped image: {}", e)
                }

                Ok(Paintings {
                    id: painting_id,
                    full: full_image.unwrap(),
                    cropped: cropped_img.unwrap(),
                    thumbnail: thumbnail_img.unwrap(),
                })
            })
            .collect::<Result<Vec<Paintings>, anyhow::Error>>()?;

        let artist = {
            let (born, died) = csv_artist
                .years
                .split_once(|c| c == '-' || c == '–')
                .ok_or(anyhow::anyhow!(
                    "Failed to parse years for artist {}",
                    csv_artist.name
                ))?;
            let born = born.trim().to_string();
            let died = died.trim().to_string();
            Artist {
                id: Uuid::new_v7(Timestamp::now(NoContext)),
                name: csv_artist.name,
                born,
                died,
                genre: csv_artist.genre,
                nationality: csv_artist.nationality,
                bio: csv_artist.bio,
                wikipedia: csv_artist.wikipedia,
                paintings,
            }
        };
        Ok(artist)
    }
}

impl Artist {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

pub fn create_db_builder() -> Result<DatabaseBuilder, anyhow::Error> {
    let mut builder = DatabaseBuilder::new();
    builder.define::<Artist>()?;

    Ok(builder)
}
