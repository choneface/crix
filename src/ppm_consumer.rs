use image::ImageReader;
use image::RgbImage;

pub struct PpmFile {
    pub width: u32,
    pub height: u32,
    pub image: RgbImage,
}

pub(crate) fn read_ppm_file(file_path: &str) -> PpmFile{
    let img_reader =ImageReader::open(file_path)
        .expect("Failed to open file");
    let img = img_reader.decode()
        .expect("Failed to decode image");
    PpmFile {
        width: img.width(),
        height: img.height(),
        image: img.to_rgb8(),
    }
}
