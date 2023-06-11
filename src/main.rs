use itertools::iproduct;
use pvl::{parse_and_print_pvl, print_grouping, print_kvp, PropertyGrouping, Pvl};
use sciimg::prelude::*;
use sciimg::{binfilereader::*, enums::ImageMode, image, imagebuffer};
use std::path::Path;
pub fn main() {
    //"pvl/tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL"
    //pvl/tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.LBL
    //parse_and_print_pvl("pvl/tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.LBL");

    let ncam = "pvl/tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.LBL";
    let mahli = "pvl/tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL";

    let p = Path::new(ncam);
    if let Ok(pvl) = Pvl::load(p) {
        if let Some(image_object) = pvl.get_object("IMAGE") {
            // print_grouping(image_object);

            let lines = image_object
                .get_property("LINES")
                .unwrap()
                .value
                .parse_usize()
                .unwrap_or(0);
            let samples = image_object
                .get_property("LINE_SAMPLES")
                .unwrap()
                .value
                .parse_usize()
                .unwrap_or(0);
            let sample_bits = image_object
                .get_property("SAMPLE_BITS")
                .unwrap()
                .value
                .parse_i32()
                .unwrap_or(0);
            let bands = image_object
                .get_property("BANDS")
                .unwrap()
                .value
                .parse_usize()
                .unwrap_or(0);

            println!("{:?}", pvl.get_property("^IMAGE"));

            // Holy function chain, batman!
            let filename = pvl
                .get_property("^IMAGE")
                .unwrap()
                .value
                .parse_array()
                .unwrap()
                .first()
                .unwrap()
                .to_owned()
                .parse_string()
                .unwrap();

            let referenced_image_file_path = p.parent().unwrap().join(Path::new(&filename));
            println!("File Path: {:?}", referenced_image_file_path);

            println!("Lines: {}", lines);
            println!("Samples: {}", samples);
            println!("Bits per Sample: {}", sample_bits);
            println!("Bands: {}", bands);

            let mut file_reader = BinFileReader::new(&referenced_image_file_path);
            let mut image = Image::new_with_bands(
                samples as usize,
                lines as usize,
                bands as usize,
                match sample_bits {
                    8 => ImageMode::U8BIT,
                    12 => ImageMode::U12BIT,
                    16 => ImageMode::U16BIT,
                    _ => panic!("Unsupported pixel depth: {}", sample_bits),
                },
            )
            .unwrap();

            iproduct!(0..lines, 0..samples, 0..bands).for_each(|(y, x, b)| {
                let byte_index = (lines * samples * b) + y * samples + x;
                let pixel_value = file_reader.read_u8(byte_index);
                image.put(x, y, pixel_value as f32, b);
                //image.get_band(b).put(x, y, pixel_value as Dn)
            });

            image.save("test.png").expect("Failed to save image");
        }

        // pvl.properties.into_iter().for_each(|p| {
        //     print_kvp(&p, false);
        // });
        // pvl.groups.into_iter().for_each(|g| {
        //     print_grouping(&g);
        // });
        // pvl.objects.into_iter().for_each(|g| {
        //     print_grouping(&g);
        // });
    }
}
