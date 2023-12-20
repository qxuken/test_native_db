use crate::csv::CsvArtist;
use image::{imageops::FilterType, io::Reader as ImageReader};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Serialize, Deserialize)]
pub enum ImgSize {
    Full,
    Cropped,
    Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Img {
    id: Uuid,
    path: PathBuf,
    size: ImgSize,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paintings {
    id: Uuid,
    imgs: Vec<Img>,
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

impl TryFrom<(CsvArtist, &Path, &Path)> for Artist {
    type Error = anyhow::Error;

    fn try_from(
        (csv_artist, input_path, data_path): (CsvArtist, &Path, &Path),
    ) -> Result<Self, Self::Error> {
        let img_dest_path = data_path.join("img");
        fs::create_dir_all(&img_dest_path)?;
        let img_input_path = input_path.join("images").join(
            csv_artist
                .name
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("_"),
        );
        let resized_img_input_path = input_path.join("resized");
        #[allow(clippy::never_loop)]
        for entry in fs::read_dir(img_input_path)? {
            let ts = Timestamp::from_unix(NoContext, 1497624119, 1234);
            let img_id = Uuid::new_v7(ts);
            let image = entry?;
            let image_path = image.path();
            let image_view = ImageReader::open(&image_path)?.decode()?;
            let resized_img_path = resized_img_input_path.join(image.file_name());
            println!("resized_img_path: {resized_img_path:?}");
            println!("resized_img_path exists: {:?}", resized_img_path.is_file());
            let dest_full_img_path = img_dest_path.join(format!("{}_full.jpg", &img_id));
            let dest_resized_img_path = img_dest_path.join(format!("{img_id}_resized.jpg"));
            let dest_thumnail_img_path = img_dest_path.join(format!("{img_id}_thumbnail.jpg"));
            fs::copy(image_path, dest_full_img_path)?;
            fs::copy(resized_img_path, dest_resized_img_path)?;
            image_view
                .resize(150, 150, FilterType::Lanczos3)
                .save(dest_thumnail_img_path)?;

            anyhow::bail!("not implemented")
        }
        anyhow::bail!("not implemented")
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
