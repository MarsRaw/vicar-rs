use itertools::iproduct;
use sciimg::prelude::*;
use vicar::*;

#[test]
fn test_load_msl_navcam() {
    let ncam = "pvl/tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.IMG";

    let vr = VicarReader::new(ncam).unwrap();

    assert!(vr.has_internal_label());
    assert!(vr.get_property("LBLSIZE").is_ok());
    assert_eq!(vr.scan_for_property("LBLSIZE").unwrap(), 30720);
    assert_eq!(
        vr.get_property("LBLSIZE")
            .unwrap()
            .value
            .parse_usize()
            .unwrap(),
        18432
    );

    let mut image =
        Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

    iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
        let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
        // println!("Pixel Value: {}", pixel_value);
        image.put(x, y, pixel_value as f32, b);
    });
    image.normalize_between(0.0, 65535.0);
    assert!(image.save("tests/testdata/msl_navcam_test.png").is_ok());
}

#[test]
fn test_load_cassini_wac() {
    let cassini_wac = "pvl/tests/testdata/cassini/wac/W1884114531_2.IMG";
    let vr = VicarReader::new(cassini_wac).unwrap();

    assert!(vr.has_internal_label());
    assert!(vr.get_property("LBLSIZE").is_ok());
    assert_eq!(vr.scan_for_property("LBLSIZE").unwrap(), 0);
    assert_eq!(
        vr.get_property("LBLSIZE")
            .unwrap()
            .value
            .parse_usize()
            .unwrap(),
        3144
    );

    let mut image =
        Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

    iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
        let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
        // println!("Pixel Value: {}", pixel_value);
        image.put(x, y, pixel_value as f32, b);
    });
    image.normalize_between(0.0, 65535.0);
    assert!(image.save("tests/testdata/cassini_wac_test.png").is_ok());
}

#[test]
fn test_load_msl_mahli_detatched_label() {
    let mahli = "pvl/tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL";

    let vr = VicarReader::new_from_detached_label(mahli).unwrap();

    let mut image =
        Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

    iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
        let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
        image.put(x, y, pixel_value as f32, b);
    });
    image.normalize_between(0.0, 65535.0);
    assert!(image.save("tests/testdata/msl_mahli_test.png").is_ok());
}
