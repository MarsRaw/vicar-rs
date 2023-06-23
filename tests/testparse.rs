use std::path::Path;
use vicar::pvl::*;

/// These tests are inexhustive checks to make sure the files basically load.
#[test]
fn test_msl_navcam_pvl_loaded_lbl() {
    // Navcam
    assert!(Pvl::load(Path::new(
        "tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.LBL"
    ))
    .is_ok());
}

#[test]
fn test_msl_navcam_pvl_loaded_img() {
    let p = Pvl::load(Path::new(
        "tests/testdata/msl/navcam/NRB_701384494RAD_F0933408NCAM00200M1.IMG",
    ));
    // Navcam
    assert!(p.is_ok());
    let lbl = p.unwrap();
    assert!(lbl.has_property("MISSION_PHASE_NAME"));

    let prop = lbl.get_property("MISSION_PHASE_NAME").unwrap();
    assert_eq!(
        prop.value.parse_string().unwrap(),
        "EXTENDED SURFACE MISSION"
    );
}

#[test]
fn test_msl_mastcampvl_loaded() {
    // Mastcam
    assert!(Pvl::load(Path::new(
        "tests/testdata/msl/mcam/3423MR1016960081600825C00_DRCX.LBL"
    ))
    .is_ok());
}

#[test]
fn test_msl_hazcam_pvl_loaded() {
    // Hazcam
    assert!(Pvl::load(Path::new(
        "tests/testdata/msl/hazcam/RLB_701384675RAS_F0933408RHAZ00337M1.LBL"
    ))
    .is_ok());
}

#[test]
fn test_msl_mardi_pvl_loaded() {
    // MARDI
    assert!(Pvl::load(Path::new(
        "tests/testdata/msl/mardi/3420MD0012740000202655E01_DRCX.LBL"
    ))
    .is_ok());
}

#[test]
fn test_msl_mahli_pvl_loaded() {
    // MAHLI
    assert!(Pvl::load(Path::new(
        "tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL"
    ))
    .is_ok());
}

#[test]
fn test_mer2_pancam_pvl_loaded() {
    // pancam
    assert!(Pvl::load(Path::new(
        "tests/testdata/mer/mer2/pancam/1p581379812rsdd2fcp2398l2m1.img.lbl"
    ))
    .is_ok());
}

#[test]
fn test_mer2_mi_pvl_loaded() {
    // mi
    assert!(Pvl::load(Path::new(
        "tests/testdata/mer/mer2/mi/1m581290805ilfd2fcp2907m2m1.img.lbl"
    ))
    .is_ok());
}

#[test]
fn test_mer2_navcam_pvl_loaded() {
    // navcam
    assert!(Pvl::load(Path::new(
        "tests/testdata/mer/mer2/navcam/1n579700548ffld2fcp1981l0m1.img.lbl"
    ))
    .is_ok());
}

#[test]
fn test_mer2_hazcam_pvl_loaded() {
    // hazcam
    assert!(Pvl::load(Path::new(
        "tests/testdata/mer/mer2/hazcam/1f581291004ednd2fcp1121r0m1.img.lbl"
    ))
    .is_ok());
}

#[test]
fn test_cassini_nac_pvl_loaded() {
    // nac
    assert!(Pvl::load(Path::new("tests/testdata/cassini/nac/N1884111831_1.LBL")).is_ok());
}

#[test]
fn test_cassini_wac_pvl_loaded() {
    // wac
    assert!(Pvl::load(Path::new("tests/testdata/cassini/wac/W1884114531_2.LBL")).is_ok());
}

#[test]
#[ignore = "Failing, needs fix"]
fn test_cassini_vims_pvl_loaded() {
    // vims
    assert!(Pvl::load(Path::new("tests/testdata/cassini/vims/v1883935188_1.lbl")).is_ok());
}

#[test]
fn test_voyager1_issn_pvl_loaded() {
    // issn
    assert!(Pvl::load(Path::new("tests/testdata/voyager/v1/issn/C3580800_RAW.LBL")).is_ok());
}

#[test]
fn test_voyager1_issw_pvl_loaded() {
    // issw
    assert!(Pvl::load(Path::new("tests/testdata/voyager/v1/issw/C3501111_RAW.LBL")).is_ok());
}

#[test]
fn test_voyager2_issn_pvl_loaded() {
    // issn
    assert!(Pvl::load(Path::new("tests/testdata/voyager/v2/issn/C1201604_RAW.LBL")).is_ok());
}

#[test]
fn test_voyager2_issw_pvl_loaded() {
    // issw
    assert!(Pvl::load(Path::new("tests/testdata/voyager/v2/issw/C1201656_RAW.LBL")).is_ok());
}
