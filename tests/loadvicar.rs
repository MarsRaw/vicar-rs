use itertools::iproduct;
use sciimg::prelude::*;
use vicar::*;

macro_rules! test_from_img {
    ($fn_name:ident, $img_path:expr, $out_file:expr, $lbl_byte:expr, $lbl_size:expr) => {
        #[test]
        pub fn $fn_name() {
            let ncam = $img_path;

            let vr = VicarReader::new(ncam).unwrap();

            assert!(vr.has_internal_label());
            assert!(vr.get_property("LBLSIZE").is_ok());
            assert_eq!(vr.scan_for_property("LBLSIZE").unwrap(), $lbl_byte);
            assert_eq!(
                vr.get_property("LBLSIZE")
                    .unwrap()
                    .value
                    .parse_usize()
                    .unwrap(),
                $lbl_size
            );

            let mut image =
                Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

            iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
                let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
                image.put(x, y, pixel_value as f32, b);
            });
            image.normalize_between(0.0, 65535.0);
            assert!(image.save($out_file).is_ok());
        }
    };
}

macro_rules! test_from_detached_label {
    ($fn_name:ident, $img_path:expr, $out_file:expr) => {
        #[test]
        pub fn $fn_name() {
            let vr = VicarReader::new_from_detached_label($img_path).unwrap();

            let mut image =
                Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

            iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
                let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
                image.put(x, y, pixel_value as f32, b);
            });
            image.normalize_between(0.0, 65535.0);
            assert!(image.save($out_file).is_ok());
        }
    };
}

test_from_img!(
    test_load_msl_navcam,
    "pvl/tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.IMG",
    "tests/testdata/msl_navcam_test.png",
    30720,
    18432
);

test_from_img!(
    test_load_msl_hazcam,
    "pvl/tests/testdata/msl/hazcam/RLB_701384675RAS_F0933408RHAZ00337M1.IMG",
    "tests/testdata/msl_hazcam_test.png",
    30720,
    18432
);

test_from_img!(
    test_load_cassini_wac,
    "pvl/tests/testdata/cassini/wac/W1884114531_2.IMG",
    "tests/testdata/cassini_wac_test.png",
    0,
    3144
);

test_from_img!(
    test_load_mer2_navcam,
    "pvl/tests/testdata/mer/mer2/navcam/1n579700548ffld2fcp1981l0m1.img",
    "tests/testdata/mer2_navcam_test.png",
    26624,
    16384
);

test_from_img!(
    test_load_mer2_pancam,
    "pvl/tests/testdata/mer/mer2/pancam/1p581379812rsdd2fcp2398l2m1.img",
    "tests/testdata/mer2_pancam_test.png",
    26624,
    16384
);

test_from_img!(
    test_load_mer2_hazcam,
    "pvl/tests/testdata/mer/mer2/hazcam/1f581291004ednd2fcp1121r0m1.img",
    "tests/testdata/mer2_hazcam_test.png",
    24576,
    14336
);

test_from_img!(
    test_load_mer2_mi,
    "pvl/tests/testdata/mer/mer2/mi/1m581290805ilfd2fcp2907m2m1.img",
    "tests/testdata/mer2_mi_test.png",
    26624,
    14336
);

test_from_img!(
    test_load_voyager1_issn,
    "pvl/tests/testdata/voyager/v1/issn/C3580800_RAW.IMG",
    "tests/testdata/voyager1_issn_test.png",
    0,
    1024
);

test_from_img!(
    test_load_voyager1_issw,
    "pvl/tests/testdata/voyager/v1/issw/C3501111_RAW.IMG",
    "tests/testdata/voyager1_issw_test.png",
    0,
    1024
);

test_from_img!(
    test_load_voyager2_issn,
    "pvl/tests/testdata/voyager/v2/issn/C1201604_RAW.IMG",
    "tests/testdata/voyager2_issn_test.png",
    0,
    1024
);

test_from_img!(
    test_load_voyager2_issw,
    "pvl/tests/testdata/voyager/v2/issw/C1201656_RAW.IMG",
    "tests/testdata/voyager2_issw_test.png",
    0,
    1024
);

test_from_detached_label!(
    test_load_msl_mahli_detatched_label,
    "pvl/tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL",
    "tests/testdata/msl_mahli_test.png"
);

test_from_detached_label!(
    test_load_msl_mardi_detatched_label,
    "pvl/tests/testdata/msl/mardi/3420MD0012740000202655E01_DRCX.LBL",
    "tests/testdata/msl_mardi_test.png"
);

test_from_detached_label!(
    test_load_msl_mastcam_detatched_label,
    "pvl/tests/testdata/msl/mcam/3423MR1016960081600825C00_DRCX.LBL",
    "tests/testdata/msl_mastcam_test.png"
);
