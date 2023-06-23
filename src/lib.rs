use pvl::{PropertyGrouping, Pvl};
use regex::Regex;
use sciimg::binfilereader::*;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::{error::Error, fmt};

#[macro_use]
extern crate lazy_static;

/// Formats an error object to a string via {:?} Debug derived method
macro_rules! t {
    ($error_message:expr) => {
        format!("{:?}", $error_message)
    };
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PixelFormat {
    Byte,    // One byte, u8
    Half,    // Two byte signed, i16
    Word,    // Two byte signed, i16, Deprecated
    Full,    // Four byte signed, i32
    Long,    // Four byte signed, i32, Deprecated
    Real,    // Single precision float, f16
    Doub,    // Double precision float, f32
    Comp,    // Complex,, composed of two reals in the order (real, imaginary)
    Complex, // Complex,, composed of two reals in the order (real, imaginary), Deprecated
}

impl PixelFormat {
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            PixelFormat::Byte => 1,
            PixelFormat::Half => 2,
            PixelFormat::Word => 2,
            PixelFormat::Full => 4,
            PixelFormat::Long => 4,
            PixelFormat::Real => 2,
            PixelFormat::Doub => 2,
            PixelFormat::Comp => 4,
            PixelFormat::Complex => 4,
        }
    }

    pub fn from_string(s: &str) -> Result<PixelFormat, VicarError> {
        match s.to_uppercase().as_str() {
            "BYTE" => Ok(PixelFormat::Byte),
            "HALF" => Ok(PixelFormat::Half),
            "WORD" => Ok(PixelFormat::Half), // Word is deprecated, routing to Half
            "FULL" => Ok(PixelFormat::Full),
            "LONG" => Ok(PixelFormat::Full), // Long is deprecated, routing to Full
            "REAL" => Ok(PixelFormat::Real),
            "DOUB" => Ok(PixelFormat::Doub),
            "COMP" => Ok(PixelFormat::Comp),
            "COMPLEX" => Ok(PixelFormat::Comp), // Complex is deprecated, routing to Comp
            _ => Err(VicarError::UnexpectedEnum(t!(s))),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DataType {
    Image,
    Parms,
    Parm,
    Param,
    Graph1,
    Graph2,
    Graph3,
    Tabular,
}

impl DataType {
    pub fn from_string(s: &str) -> Result<DataType, VicarError> {
        match s.to_uppercase().as_str() {
            "IMAGE" => Ok(DataType::Image),
            "PARMS" => Ok(DataType::Parms),
            "PARM" => Ok(DataType::Parm),
            "PARAM" => Ok(DataType::Param),
            "GRAPH1" => Ok(DataType::Graph1),
            "GRAPH2" => Ok(DataType::Graph2),
            "GRAPH3" => Ok(DataType::Graph3),
            "TABULAR" => Ok(DataType::Tabular),
            _ => Err(VicarError::UnexpectedEnum(t!(s))),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DataOrganization {
    Bsq, // Band SeQuential, N1=Samples, N2=Lines, N3=Bands
    Bil, // Band Interleaved, N1=Samples, N2=Bands, N3=Lines
    Bip, // Band Interleaved by Pixel, N1=Bands, N2=Samples, N3=Lines
}

impl DataOrganization {
    pub fn from_string(s: &str) -> Result<DataOrganization, VicarError> {
        match s.to_uppercase().as_str() {
            "BSQ" => Ok(DataOrganization::Bsq),
            "BIL" => Ok(DataOrganization::Bil),
            "BIP" => Ok(DataOrganization::Bip),
            _ => Err(VicarError::UnexpectedEnum(t!(s))),
        }
    }
}

#[derive(Debug)]
pub enum VicarError {
    Eof,
    Syntax(String),
    Programming(String),
    InvalidType,
    ValueTypeParseError,
    InvalidEncoding(String),
    General(String),
    PropertyNotFound(String),
    UnexpectedEnum(String),
    LabelError(String),
}

impl Error for VicarError {}

impl fmt::Display for VicarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error of type {:?}", self)
    }
}

impl From<anyhow::Error> for VicarError {
    fn from(value: anyhow::Error) -> Self {
        VicarError::General(t!(value))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ValueType {
    Undetermined,
    Array,
    String,
    Float,
    Integer,
    Bool,
    Flag, // A string but not wrapped in quotes
    BitMask,
}

/// Contains PVL right-hand values and flags
#[derive(Debug, Clone)]
pub struct Value {
    value_raw: String,
    value_type: ValueType,
}

lazy_static! {
    static ref BOOL_DETERMINATE: Regex = Regex::new("^['\"](TRUE|FALSE)['\"]$").unwrap();
    static ref STRING_DETERMINATE: Regex = Regex::new("^['\"].*['\"]$").unwrap();
    static ref ARRAY_DETERMINATE: Regex = Regex::new("^\\(.*\\)$").unwrap();
    static ref FLOAT_DETERMINATE: Regex = Regex::new("^-*[0-9]+\\.[0-9][ ]*").unwrap();
    static ref INTEGER_DETERMINATE: Regex = Regex::new("^[+-]*[0-9]+[^#a-zA-Z]*[ ]*").unwrap();
    static ref FLAG_DETERMINATE: Regex = Regex::new("^[a-zA-Z_]+[a-zA-Z0-9]+$").unwrap();
    static ref BITMASK_DETERMINATE: Regex = Regex::new("^[1-8]*#+[0-1]+#+$").unwrap();
}

#[macro_export]
macro_rules! impl_parse_fn {
    ($fn_name:ident, $type:ty, $value_type:expr) => {
        // the $values just get swapped in.
        pub fn $fn_name(&self) -> Result<$type, VicarError> {
            // I'm gonna allow parsing if the type is undetermined. A type being undetermined is my problem, but
            // the user will have the option (and risk) of parsing it
            if self.value_type != ValueType::Undetermined && self.value_type != $value_type {
                Err(VicarError::InvalidType)
            } else {
                match self.value_raw.parse::<$type>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(VicarError::ValueTypeParseError),
                }
            }
        }
    };
}

impl Value {
    impl_parse_fn!(parse_f32, f32, ValueType::Float);
    impl_parse_fn!(parse_f64, f64, ValueType::Float);
    impl_parse_fn!(parse_u8, u8, ValueType::Integer);
    impl_parse_fn!(parse_u16, u16, ValueType::Integer);
    impl_parse_fn!(parse_u32, u32, ValueType::Integer);
    impl_parse_fn!(parse_u64, u64, ValueType::Integer);
    impl_parse_fn!(parse_usize, usize, ValueType::Integer);
    impl_parse_fn!(parse_i8, i8, ValueType::Integer);
    impl_parse_fn!(parse_i16, i16, ValueType::Integer);
    impl_parse_fn!(parse_i32, i32, ValueType::Integer);
    impl_parse_fn!(parse_i64, i64, ValueType::Integer);
    impl_parse_fn!(parse_bool, bool, ValueType::Bool);
    impl_parse_fn!(parse_flag, String, ValueType::Flag);

    /// Constructs a new Value object and determines type of provided raw data
    pub fn new(value_raw: &str) -> Self {
        Value {
            value_raw: value_raw.to_owned(),
            value_type: Value::determine_type(value_raw),
        }
    }

    /// Determines the data type of the raw value based on regex matches.
    fn determine_type(value_raw: &str) -> ValueType {
        if BOOL_DETERMINATE.is_match(value_raw) {
            ValueType::Bool
        } else if STRING_DETERMINATE.is_match(value_raw) {
            ValueType::String
        } else if ARRAY_DETERMINATE.is_match(value_raw) {
            ValueType::Array
        } else if FLOAT_DETERMINATE.is_match(value_raw) {
            ValueType::Float
        } else if BITMASK_DETERMINATE.is_match(value_raw) {
            ValueType::BitMask
        } else if INTEGER_DETERMINATE.is_match(value_raw) {
            ValueType::Integer
        } else if FLAG_DETERMINATE.is_match(value_raw) {
            ValueType::Flag
        } else {
            ValueType::Undetermined
        }
    }

    pub fn parse_string(&self) -> Result<String, VicarError> {
        // I'm gonna allow parsing if the type is undetermined. A type being undetermined is my problem, but
        // the user will have the option (and risk) of parsing it
        if self.value_type != ValueType::Undetermined && self.value_type != ValueType::String {
            Err(VicarError::InvalidType)
        } else {
            Ok(self.value_raw.replace(['\"', '\''], ""))
        }
    }

    /// Parses the raw data value to an array of Values. Throws an error if we are not an array type
    pub fn parse_array(&self) -> Result<Vec<Value>, VicarError> {
        if self.value_type != ValueType::Array {
            Err(VicarError::InvalidType)
        } else {
            Ok(self.value_raw[1..(self.value_raw.len() - 1)]
                .split(',')
                .map(Value::new)
                .collect())
        }
    }
}

/// Represents the basic KEY = VALUE pair in a PVL file
#[derive(Debug, Clone)]
pub struct KeyValuePair {
    pub key: String,
    pub value: Value,
}

/// Main PVL parsing engine
pub struct VicarReader {
    reader: BinFileReader,
    data_start: usize,
    pub label_size: usize,
    pub dimensions: usize,
    pub binary_bytes_before_record: usize,
    pub binary_bytes_header: usize,
    pub recsize: usize,
    pub lines: usize,
    pub samples: usize,
    pub bands: usize,
    pub org: DataOrganization,
    pub format: PixelFormat,
    pub data_type: DataType,
    pub strings: String,
}

impl fmt::Display for VicarReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Dimensions {}, Record Size: {}, Data Start: {}, Lines: {}, Samples: {}, Bands: {}, Organization: {:?}, Format: {:?}, Type: {:?}, NBB: {}, NLB: {}",
            self.dimensions, self.recsize, self.data_start, self.lines, self.samples, self.bands, self.org, self.format, self.data_type, self.binary_bytes_before_record,
            self.binary_bytes_header
        )
    }
}

impl VicarReader {
    pub fn new_from_detached_label<S>(label_file_path: &S) -> Result<Self, VicarError>
    where
        S: AsRef<Path> + ?Sized + AsRef<OsStr>,
    {
        let p = Path::new(label_file_path);
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
                let bands = image_object
                    .get_property("BANDS")
                    .unwrap()
                    .value
                    .parse_usize()
                    .unwrap_or(0);

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

                let strings = VicarReader::read_vicar_to_string_lossy(&referenced_image_file_path)?;
                let reader = BinFileReader::new(&referenced_image_file_path);

                Ok(VicarReader {
                    reader,
                    data_start: 0,
                    label_size: 0,
                    dimensions: bands,
                    recsize: 0,
                    lines,
                    samples,
                    bands,
                    org: DataOrganization::Bsq,
                    format: PixelFormat::Byte,
                    data_type: DataType::Image,
                    strings,
                    binary_bytes_before_record: 0,
                    binary_bytes_header: 0,
                })
            } else {
                Err(VicarError::PropertyNotFound(t!("IMAGE")))
            }
        } else {
            Err(VicarError::LabelError(t!("Error loading detached label")))
        }
    }

    pub fn new<S>(file_path: &S) -> Result<Self, VicarError>
    where
        S: AsRef<Path> + ?Sized + AsRef<OsStr>,
    {
        let strings = VicarReader::read_vicar_to_string_lossy(file_path)?;
        let reader = BinFileReader::new(file_path);

        let label_start = VicarReader::_scan_for_property(&strings, "LBLSIZE")?;

        let lblsize = VicarReader::_get_property(&strings, "LBLSIZE")?
            .value
            .parse_usize()?;

        let recsize = VicarReader::_get_property(&strings, "RECSIZE")?
            .value
            .parse_usize()?;

        let dim = VicarReader::_get_property(&strings, "DIM")?
            .value
            .parse_usize()?;

        let n1 = VicarReader::_get_property(&strings, "N1")?
            .value
            .parse_usize()?;

        let n2 = VicarReader::_get_property(&strings, "N2")?
            .value
            .parse_usize()?;

        let n3 = VicarReader::_get_property(&strings, "N3")?
            .value
            .parse_usize()?;

        let nlb = VicarReader::_get_property(&strings, "NLB")?
            .value
            .parse_usize()?;

        let nbb = VicarReader::_get_property(&strings, "NBB")?
            .value
            .parse_usize()?;

        let data_type = DataType::from_string(
            &VicarReader::_get_property(&strings, "TYPE")?
                .value
                .parse_string()?,
        )?;

        let binary_header_size = nlb * recsize;
        let binary_header_start = lblsize;
        let binary_header_stop = binary_header_size + binary_header_start;
        let _binary_header = reader.read_bytes(binary_header_start, binary_header_size);

        let format = PixelFormat::from_string(
            &VicarReader::_get_property(&strings, "FORMAT")?
                .value
                .parse_string()?,
        )?;

        let organization = DataOrganization::from_string(
            &VicarReader::_get_property(&strings, "ORG")?
                .value
                .parse_string()?,
        )?;

        let (lines, samples, bands) = VicarReader::to_lines_samples_bands(n1, n2, n3, organization);

        Ok(VicarReader {
            reader,
            data_start: label_start + binary_header_stop + nbb,
            label_size: lblsize,
            dimensions: dim,
            recsize,
            lines,
            samples,
            bands,
            org: organization,
            format,
            data_type,
            strings,
            binary_bytes_before_record: nbb,
            binary_bytes_header: nlb,
        })
    }

    fn to_lines_samples_bands(
        n1: usize,
        n2: usize,
        n3: usize,
        org: DataOrganization,
    ) -> (usize, usize, usize) {
        match org {
            DataOrganization::Bsq => (n2, n1, n3),
            DataOrganization::Bil => (n3, n1, n2),
            DataOrganization::Bip => (n3, n2, n1),
        }
    }

    // fn to_n1_n2_n3(&self, line: usize, sample: usize, band: usize) -> (usize, usize, usize) {
    //     match self.org {
    //         DataOrganization::Bsq => (sample, line, band),
    //         DataOrganization::Bil => (sample, band, line),
    //         DataOrganization::Bip => (band, sample, line),
    //     }
    // }

    fn read_vicar_to_string_lossy<S>(file_path: &S) -> Result<String, VicarError>
    where
        S: AsRef<Path> + ?Sized + AsRef<OsStr>,
    {
        match fs::read(file_path) {
            Ok(b) => match String::from_utf8_lossy(&b) {
                Cow::Borrowed(s) => Ok(s.to_string()),
                Cow::Owned(s) => Ok(s),
            },
            Err(why) => Err(VicarError::General(t!(why))),
        }
    }

    /// Returns the character at the specified index, or `Error::Eof` if the  index is beyond the limit of the text
    fn _char_at(strings: &String, indx: usize) -> Result<char, VicarError> {
        if indx >= strings.len() {
            Err(VicarError::Eof)
        } else {
            //Ok(self.content.chars().nth(indx).unwrap()) // Slow but correct(er)
            Ok(strings.as_bytes()[indx] as char) // WAY faster, but won't work for non 8-bit text files
        }
    }

    /// Returns the character at the specified index, or `Error::Eof` if the  index is beyond the limit of the text
    pub fn char_at(&self, indx: usize) -> Result<char, VicarError> {
        if indx >= self.strings.len() {
            Err(VicarError::Eof)
        } else {
            //Ok(self.content.chars().nth(indx).unwrap()) // Slow but correct(er)
            Ok(self.strings.as_bytes()[indx] as char) // WAY faster, but won't work for non 8-bit text files
        }
    }

    pub fn is_index_at_eof(&self, index: usize) -> bool {
        index >= self.strings.len()
    }

    fn _scan_for_property(strings: &String, key: &str) -> Result<usize, VicarError> {
        let key_eq = format!("{}=", key);
        for i in 0..(strings.len() - key_eq.len()) {
            if strings[i..(i + key_eq.len())] == key_eq {
                return Ok(i);
            }
        }

        Err(VicarError::Eof)
    }

    pub fn scan_for_property(&self, key: &str) -> Result<usize, VicarError> {
        VicarReader::_scan_for_property(&self.strings, key)
    }

    fn _has_property(strings: &String, key: &str) -> bool {
        VicarReader::_scan_for_property(strings, key).is_ok()
    }

    pub fn has_property(&self, key: &str) -> bool {
        VicarReader::_has_property(&self.strings, key)
    }

    fn _has_internal_label(strings: &String) -> bool {
        VicarReader::_has_property(strings, "LBLSIZE")
    }

    pub fn has_internal_label(&self) -> bool {
        VicarReader::_has_internal_label(&self.strings)
    }

    pub fn _extract_property_raw(strings: &String, key: &str) -> Result<String, VicarError> {
        let index = VicarReader::_scan_for_property(strings, key)?;
        let mut end_index = index;

        for i in (index + 1)..strings.len() {
            if VicarReader::_char_at(strings, i).unwrap() == ' ' {
                end_index = i;
                break;
            }
        }

        let property_raw = strings[index..end_index].to_string();

        Ok(property_raw)
    }

    pub fn extract_property_raw(&self, key: &str) -> Result<String, VicarError> {
        VicarReader::_extract_property_raw(&self.strings, key)
    }

    fn _get_property(strings: &String, key: &str) -> Result<KeyValuePair, VicarError> {
        let property_raw = VicarReader::_extract_property_raw(strings, key)?;

        let parts: Vec<String> = property_raw.split('=').map(|p| p.to_string()).collect();

        if parts.len() != 2 {
            Err(VicarError::Syntax(format!(
                "Property syntax error, missing or too many equality indicators: {:?}",
                property_raw
            )))
        } else {
            Ok(KeyValuePair {
                key: parts[0].to_owned(),
                value: Value::new(parts[1].as_str()),
            })
        }
    }

    pub fn get_property(&self, key: &str) -> Result<KeyValuePair, VicarError> {
        VicarReader::_get_property(&self.strings, key)
    }

    fn get_pixel_index(&self, line: usize, sample: usize, band: usize) -> usize {
        (self.lines * self.samples * self.format.bytes_per_sample() * band
            + line * self.binary_bytes_before_record)
            + line * self.samples * self.format.bytes_per_sample()
            + sample * self.format.bytes_per_sample()
    }

    pub fn get_pixel_value(
        &self,
        line: usize,
        sample: usize,
        band: usize,
    ) -> Result<f32, VicarError> {
        let byte_index = self.get_pixel_index(line, sample, band);

        let start = self.data_start + byte_index;
        match self.format {
            PixelFormat::Byte => Ok(self.reader.read_u8(start)? as f32),
            PixelFormat::Half | PixelFormat::Word => Ok(self
                .reader
                .read_i16_with_endiness(start, Endian::BigEndian)?
                as f32),
            PixelFormat::Full | PixelFormat::Long => Ok(self
                .reader
                .read_i32_with_endiness(start, Endian::BigEndian)?
                as f32),
            PixelFormat::Real => Ok(self
                .reader
                .read_f32_with_endiness(start, Endian::BigEndian)?),
            PixelFormat::Doub => Ok(self
                .reader
                .read_i64_with_endiness(start, Endian::BigEndian)?
                as f32),
            PixelFormat::Comp | PixelFormat::Complex => todo!(),
        }
    }
}
