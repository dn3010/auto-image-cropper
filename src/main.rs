use clap::{App, Arg};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::fs;
use std::fs::DirEntry;
use std::path::Path;

use auto_image_cropper::ImageCrop;

#[derive(Debug)]
struct Crop {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn adjust_square_center(crop: Crop) -> Crop {
    if crop.width > crop.height {
        let diff = crop.width - crop.height;
        let new_y = crop.y - (diff / 2);
        let new_height = crop.height + diff;
        return Crop {
            x: crop.x,
            y: new_y,
            width: crop.width,
            height: new_height,
        };
    }

    let diff = crop.height - crop.width;
    let new_x = crop.x - (diff / 2);
    let new_width = crop.width + diff;
    return Crop {
        x: new_x,
        y: crop.y,
        width: new_width,
        height: crop.height,
    };
}

fn crop_image(input_path: &str, output_path: &str) {
    let mut image = ImageCrop::open(&Path::new(input_path))
        .expect(&format!("Failed to read input file: '{}'", input_path));

    let (top_left_corner, bottom_right_corner) = image.calculate_corners();

    println!(
        "Cropping rectangle (({0}, {1}), ({2}, {3})) from {4} to {5}",
        top_left_corner.x,
        top_left_corner.y,
        bottom_right_corner.x,
        bottom_right_corner.y,
        input_path,
        output_path
    );

    let crop_padding = 10;

    let target: Crop = Crop {
        x: top_left_corner.x + crop_padding,
        y: top_left_corner.y + crop_padding,
        width: bottom_right_corner.x - top_left_corner.x - crop_padding * 2,
        height: bottom_right_corner.y - top_left_corner.y - crop_padding * 2,
    };

    let adjusted: Crop = adjust_square_center(target);

    let sub_image = image
        .original
        .crop(adjusted.x, adjusted.y, adjusted.width, adjusted.height);

    sub_image
        .save_with_format(output_path, image::ImageFormat::Png)
        .expect(&format!(
            "Failed to save input file: '{}' to output path: '{}'",
            input_path, output_path
        ));
}

fn main() {
    let arguments = App::new("auto-image-cropper")
        .version("0.1.5")
        .author("Ritiek Malhotra <ritiekmalhotra123@gmail.com>")
        .about("Removes extra white boundaries from images to correctly resize canvas.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("LOCATION")
                .required(true)
                .takes_value(true)
                .help("Location of input image/directory"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("LOCATION")
                .takes_value(true)
                .help("Location of output image/directory"),
        )
        .get_matches();

    let input_path = arguments.value_of("input").unwrap();
    let output_path = arguments.value_of("output").unwrap_or(input_path);
    let path_type = fs::metadata(input_path).unwrap();

    if path_type.is_file() {
        crop_image(input_path, output_path);
    } else {
        let input_files = fs::read_dir(input_path).unwrap();

        if !Path::new(output_path).exists() {
            let _ = fs::create_dir(output_path);
        }
        let mut files_to_crop: Vec<DirEntry> = Vec::with_capacity(10);
        for file in input_files {
            files_to_crop.push(file.unwrap());
        }
        println!("Vector length is {}", files_to_crop.len());

        files_to_crop.par_iter().for_each(|file| {
            let img_in = file.path();
            let img_name = img_in.file_name();
            let img_out = Path::new(output_path).join(img_name.unwrap());

            crop_image(
                &(img_in.display().to_string()),
                &(img_out.into_os_string().into_string().unwrap()),
            );
        });
    }
}
