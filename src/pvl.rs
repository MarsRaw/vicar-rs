use anyhow::Result;
use regex::Regex;
use std::{borrow::Cow, fs, path::Path};

/// Parse error types
#[derive(Debug)]
pub enum Error {
    Eof,
    Syntax(String),
    CommentIsntComment,
    Programming(String),
    InvalidType,
    ValueTypeParseError,
    InvalidEncoding(String),
    General(String),
}

/// PVL Symbol types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Pointer(String),
    Key(String),
    Group,
    Object,
    BlankLine,
    ValueLineContinuation,
    GroupEnd,
    ObjectEnd,
    End,
}

impl Symbol {
    /// Extracts the value of pointer and key enums
    pub fn value(&self) -> Option<String> {
        match self {
            Symbol::Pointer(value) => Some(value.to_owned()),
            Symbol::Key(value) => Some(value.to_owned()),
            _ => None,
        }
    }
}

/// PVL measurement units
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueUnits {
    Celcius,
    Farenheit,
    Degrees,
    Radians,
    Milliseconds,
    Seconds,
}

/// PVL right-hand value data types
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
    static ref BOOL_DETERMINATE: Regex = Regex::new("^\"(TRUE|FALSE)\"$").unwrap();
    static ref STRING_DETERMINATE: Regex = Regex::new("^\".*\"$").unwrap();
    static ref ARRAY_DETERMINATE: Regex = Regex::new("^\\(.*\\)$").unwrap();
    static ref FLOAT_DETERMINATE: Regex = Regex::new("^-*[0-9]+\\.[0-9][ ]*").unwrap();
    static ref INTEGER_DETERMINATE: Regex = Regex::new("^[+-]*[0-9]+[^#a-zA-Z]*[ ]*").unwrap();
    static ref FLAG_DETERMINATE: Regex = Regex::new("^[a-zA-Z_]+[a-zA-Z0-9]+$").unwrap();
    static ref BITMASK_DETERMINATE: Regex = Regex::new("^[1-8]*#+[0-1]+#+$").unwrap();
}
const LINE_CONTINUATION_PREFIX: &str = "                                     ";

// I think you'll get a lot of value out of this sorta thing for parsing libraries.
/// Implements the miscellanous parsing functions for Value
#[macro_export]
macro_rules! impl_parse_pvl_fn {
    ($fn_name:ident, $type:ty, $value_type:expr) => {
        // the $values just get swapped in.
        pub fn $fn_name(&self) -> Result<$type, Error> {
            // I'm gonna allow parsing if the type is undetermined. A type being undetermined is my problem, but
            // the user will have the option (and risk) of parsing it
            if self.value_type != ValueType::Undetermined && self.value_type != $value_type {
                Err(Error::InvalidType)
            } else {
                match self.value_raw.parse::<$type>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(Error::ValueTypeParseError),
                }
            }
        }
    };
}

impl Value {
    impl_parse_pvl_fn!(parse_f32, f32, ValueType::Float);
    impl_parse_pvl_fn!(parse_f64, f64, ValueType::Float);
    impl_parse_pvl_fn!(parse_u8, u8, ValueType::Integer);
    impl_parse_pvl_fn!(parse_u16, u16, ValueType::Integer);
    impl_parse_pvl_fn!(parse_u32, u32, ValueType::Integer);
    impl_parse_pvl_fn!(parse_u64, u64, ValueType::Integer);
    impl_parse_pvl_fn!(parse_usize, usize, ValueType::Integer);
    impl_parse_pvl_fn!(parse_i8, i8, ValueType::Integer);
    impl_parse_pvl_fn!(parse_i16, i16, ValueType::Integer);
    impl_parse_pvl_fn!(parse_i32, i32, ValueType::Integer);
    impl_parse_pvl_fn!(parse_i64, i64, ValueType::Integer);
    impl_parse_pvl_fn!(parse_bool, bool, ValueType::Bool);
    impl_parse_pvl_fn!(parse_flag, String, ValueType::Flag);

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

    pub fn parse_string(&self) -> Result<String, Error> {
        // I'm gonna allow parsing if the type is undetermined. A type being undetermined is my problem, but
        // the user will have the option (and risk) of parsing it
        if self.value_type != ValueType::Undetermined && self.value_type != ValueType::String {
            Err(Error::InvalidType)
        } else {
            Ok(self.value_raw.replace('\"', ""))
        }
    }

    /// Parses the raw data value to an array of Values. Throws an error if we are not an array type
    pub fn parse_array(&self) -> Result<Vec<Value>, Error> {
        if self.value_type != ValueType::Array {
            Err(Error::InvalidType)
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
    pub key: Symbol,
    pub value: Value,
}

/// Defines the shared properties of both GROUP and OBJECT
pub trait PropertyGrouping {
    fn name(&self) -> String;
    fn properties(&self) -> Vec<KeyValuePair>;
    fn type_of(&self) -> Symbol;
    fn get_property(&self, name: &str) -> Option<KeyValuePair>;
    fn has_property(&self, name: &str) -> bool;
}

macro_rules! get_property {
    () => {
        fn get_property(&self, name: &str) -> Option<KeyValuePair> {
            Some(
                self.properties
                    .iter()
                    .filter(|p| match &p.key {
                        Symbol::Key(n) | Symbol::Pointer(n) => n == name,
                        _ => false,
                    })
                    .next()
                    .unwrap()
                    .to_owned(),
            )
        }
    };
}

macro_rules! has_property {
    () => {
        fn has_property(&self, name: &str) -> bool {
            self.properties
                .iter()
                .filter(|p| match &p.key {
                    Symbol::Key(n) | Symbol::Pointer(n) => n == name,
                    _ => false,
                })
                .collect::<Vec<&KeyValuePair>>()
                .len()
                > 0
        }
    };
}

/// Represents the PVL GROUP...END_GROUP structure
#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub properties: Vec<KeyValuePair>,
}

impl PropertyGrouping for Group {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn properties(&self) -> Vec<KeyValuePair> {
        self.properties.clone()
    }

    fn type_of(&self) -> Symbol {
        Symbol::Group
    }

    get_property! {}
    has_property! {}
}

/// Represents the PVL OBJECT...END_OBJECT structure
#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub properties: Vec<KeyValuePair>,
}

impl PropertyGrouping for Object {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn properties(&self) -> Vec<KeyValuePair> {
        self.properties.clone()
    }

    fn type_of(&self) -> Symbol {
        Symbol::Object
    }

    get_property! {}
    has_property! {}
}

/// Main PVL parsing engine
#[derive(Debug)]
pub struct PvlReader {
    content: String,
    pos: usize,
}

impl PvlReader {
    /// Constructs a new PVLReader object. Filters CRLF to LF. Expects UTF-8 encoded String
    pub fn new(content: &str) -> Self {
        PvlReader {
            content: PvlReader::filter_linefeeds(content),
            pos: 0,
        }
    }

    /// Filters out `\r` from the text
    fn filter_linefeeds(content: &str) -> String {
        content.chars().filter(|f| *f != '\r').collect()
    }

    /// Returns the character at the specified index, or `Error::Eof` if the  index is beyond the limit of the text
    pub fn char_at(&self, indx: usize) -> Result<char, Error> {
        if indx >= self.content.len() {
            Err(Error::Eof)
        } else {
            //Ok(self.content.chars().nth(indx).unwrap()) // Slow but correct(er)
            Ok(self.content.as_bytes()[indx] as char) // WAY faster, but won't work for non 8-bit text files
        }
    }

    /// Peeks at the character at the current caret position plus n. Returns Error::Eof if the file
    /// ends before that point
    pub fn char_at_pos_plus_n(&self, indx: usize) -> Result<char, Error> {
        if self.pos + indx >= self.content.len() {
            Err(Error::Eof)
        } else {
            //Ok(self.content.chars().nth(indx).unwrap()) // Slow but correct(er)
            Ok(self.content.as_bytes()[self.pos + indx] as char) // WAY faster, but won't work for non 8-bit text files
        }
    }

    pub fn current_char(&self) -> Result<char, Error> {
        self.char_at(self.pos)
    }

    pub fn peek_char(&self) -> Result<char, Error> {
        self.char_at(self.pos + 1)
    }

    pub fn next_char(&mut self) -> Result<char, Error> {
        self.pos += 1;
        self.current_char()
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.content.len()
    }

    pub fn has_n_remaining(&self, n: usize) -> bool {
        self.pos + n < self.content.len()
    }

    pub fn jump(&mut self, num_chars: usize) -> Result<(), Error> {
        if self.is_eof() {
            Err(Error::Eof)
        } else {
            // If the requested number of chars to skip is larger than the remaining chars, we limit to just at EOF
            let do_num_chars = if self.pos + num_chars >= self.content.len() {
                self.content.len() - self.pos
            } else {
                num_chars
            };
            self.pos += do_num_chars;
            Ok(())
        }
    }

    pub fn is_at_line_start(&self) -> Result<bool, Error> {
        if self.pos > 0 && self.pos - 1 > self.content.len() {
            Err(Error::Eof)
        } else if self.pos == 0 {
            Ok(true)
        } else {
            let c = self.char_at(self.pos - 1).unwrap();
            match c {
                '\r' | '\n' => Ok(true),
                _ => Ok(false),
            }
        }
    }

    pub fn is_at_multiline_comment_start(&self) -> Result<bool, Error> {
        if self.is_eof() || self.pos + 1 >= self.content.len() {
            Ok(false)
        } else {
            let c = self.current_char().unwrap();
            let n = self.peek_char().unwrap();
            Ok(c == '/' && n == '*')
        }
    }

    pub fn is_at_multiline_comment_end(&self) -> Result<bool, Error> {
        if self.pos + 1 >= self.content.len() {
            Ok(false)
        } else {
            let c = self.current_char().unwrap();
            let n = self.peek_char().unwrap();
            Ok(c == '*' && n == '/')
        }
    }

    pub fn skip_multiline_comment(&mut self) -> Result<String, Error> {
        if !self.is_at_multiline_comment_start().unwrap() {
            Err(Error::CommentIsntComment)
        } else {
            let mut comment_text = "".to_string();
            while !self.is_at_multiline_comment_end().unwrap() {
                comment_text.push(self.next_char().unwrap());
            }
            self.jump(2).unwrap();
            Ok(comment_text[1..(comment_text.len() - 2)].to_string())
        }
    }

    pub fn is_at_pointer(&self) -> Result<bool, Error> {
        match self.current_char() {
            Ok(c) => Ok(c == '^'),
            Err(why) => Err(why),
        }
    }

    pub fn is_at_group(&self) -> Result<bool, Error> {
        if !self.has_n_remaining(5) {
            Ok(false)
        } else if !self.is_at_line_start().unwrap() {
            Err(Error::Programming(t!(
                "Attempt to check if at group when not at start of line"
            )))
        } else {
            Ok(vec![
                self.char_at_pos_plus_n(0).unwrap(),
                self.char_at_pos_plus_n(1).unwrap(),
                self.char_at_pos_plus_n(2).unwrap(),
                self.char_at_pos_plus_n(3).unwrap(),
                self.char_at_pos_plus_n(4).unwrap(),
            ]
            .into_iter()
            .collect::<String>()
                == "GROUP")
        }
    }

    pub fn is_at_object(&self) -> Result<bool, Error> {
        if !self.has_n_remaining(6) {
            Ok(false)
        } else {
            Ok(vec![
                self.char_at_pos_plus_n(0).unwrap(),
                self.char_at_pos_plus_n(1).unwrap(),
                self.char_at_pos_plus_n(2).unwrap(),
                self.char_at_pos_plus_n(3).unwrap(),
                self.char_at_pos_plus_n(4).unwrap(),
                self.char_at_pos_plus_n(5).unwrap(),
            ]
            .into_iter()
            .collect::<String>()
                == "OBJECT")
        }
    }

    pub fn is_at_end(&self) -> bool {
        if self.has_n_remaining(3) {
            let mut s = String::new();

            s.push(self.char_at_pos_plus_n(0).unwrap());
            s.push(self.char_at_pos_plus_n(1).unwrap());
            s.push(self.char_at_pos_plus_n(2).unwrap());

            s == "END"
        } else {
            false
        }
    }

    pub fn read_symbol(&mut self) -> Result<Symbol, Error> {
        if self.is_at_value_line_continuation().unwrap() {
            Err(Error::Syntax(
                "Value line continuation without a preceeding key value pair".to_owned(),
            ))
        } else if !self.is_at_line_start().unwrap() {
            Err(Error::Programming(
                "Attempt to read a key value pair when not at beginning of a line".to_owned(),
            ))
        } else {
            let mut symbol_text = String::new();
            while !self.is_eof() {
                let c = self.current_char().unwrap();
                if c != '\n' && c != '\r' && c != '=' {
                    symbol_text.push(c);
                } else {
                    break;
                }
                self.next_char().unwrap();
            }

            symbol_text = symbol_text.trim().to_owned();
            // println!("{} -> {}", symbol_text.len(), symbol_text);
            if symbol_text.is_empty() {
                Ok(Symbol::BlankLine)
            } else if symbol_text.starts_with('^') {
                Ok(Symbol::Pointer(symbol_text))
            } else if symbol_text == "GROUP" {
                Ok(Symbol::Group)
            } else if symbol_text == "OBJECT" {
                Ok(Symbol::Object)
            } else if symbol_text == "END_GROUP" {
                Ok(Symbol::GroupEnd)
            } else if symbol_text == "END_OBJECT" {
                Ok(Symbol::ObjectEnd)
            } else if symbol_text == "END" {
                Ok(Symbol::End)
            } else {
                Ok(Symbol::Key(symbol_text))
            }
        }
    }

    pub fn read_remaining_line(&mut self) -> Result<String, Error> {
        let mut line_text = String::new();
        while !self.is_eof() {
            if self.current_char().unwrap() == '=' {
                self.jump(2).unwrap();
            }
            let c = self.current_char().unwrap();
            if c != '\n' && c != '\r' {
                line_text.push(c);
            } else {
                break;
            }
            if !self.is_eof() {
                self.next_char()?;
            }
        }

        line_text = line_text.trim().to_owned();
        Ok(line_text)
    }

    pub fn is_blank_line(&self) -> Result<bool, Error> {
        if !self.is_at_line_start()? {
            Err(Error::Programming(t!(
                "Blank line check when not at start of line"
            )))
        } else if self.is_eof() {
            Err(Error::Eof)
        } else {
            let mut found_non_ws = false;
            for i in 0..100 {
                if self.pos + i >= self.content.len() || self.char_at_pos_plus_n(i).unwrap() == '\n'
                {
                    break;
                } else if self.char_at_pos_plus_n(i).unwrap() != ' ' {
                    found_non_ws = true;
                }
            }
            Ok(!found_non_ws)
        }
    }

    pub fn is_at_equals(&self) -> Result<bool, Error> {
        match self.current_char() {
            Ok(c) => Ok(c == '='),
            Err(why) => Err(why),
        }
    }

    pub fn is_at_value_line_continuation(&self) -> Result<bool, Error> {
        if !self.is_at_line_start().unwrap() {
            Ok(false)
        } else if self.pos + LINE_CONTINUATION_PREFIX.len() >= self.content.len() {
            Err(Error::Eof)
        } else {
            Ok(
                &self.content[self.pos..(self.pos + LINE_CONTINUATION_PREFIX.len())]
                    == LINE_CONTINUATION_PREFIX,
            )
        }
    }

    pub fn jump_to_next_line(&mut self) -> Result<(), Error> {
        while self.pos <= self.content.len() {
            if self.char_at(self.pos).unwrap() == '\n' {
                self.next_char()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn rewind_to_line_beginning(&mut self) -> Result<(), Error> {
        while self.pos != 0 && !self.is_at_line_start()? {
            self.pos -= 1;
        }
        Ok(())
    }

    pub fn read_key_value_pair_raw(&mut self) -> Result<KeyValuePair, Error> {
        if self.is_at_value_line_continuation().unwrap() {
            Err(Error::Syntax(
                "Value line continuation without a preceeding key value pair".to_owned(),
            ))
        } else if !self.is_at_line_start().unwrap() {
            Err(Error::Programming(
                "Attempt to read a key value pair when not at beginning of a line".to_owned(),
            ))
        } else {
            let mut value_string = String::new();
            let key_res = self.read_symbol().unwrap();
            value_string += self.read_remaining_line().unwrap().as_ref();

            self.next_char()?;
            while let Ok(b) = self.is_at_value_line_continuation() {
                if b {
                    value_string += self.read_remaining_line().unwrap().to_string().as_ref();
                    self.next_char()?;
                } else {
                    break;
                }
            }
            Ok(KeyValuePair {
                key: key_res,
                value: Value::new(&value_string),
            })
        }
    }

    pub fn read_group(&mut self) -> Result<Group, Error> {
        if self.is_eof() {
            Err(Error::Eof)
        } else if !self.is_at_group()? {
            Err(Error::Programming(t!(
                "Attempted to read a group when not at a group start"
            )))
        } else {
            let group_start = self.read_key_value_pair_raw()?;

            let mut group = Group {
                name: group_start.value.parse_flag()?,
                properties: vec![],
            };

            while !self.is_eof() {
                if !self.is_blank_line()? {
                    let kvp = self.read_key_value_pair_raw()?;

                    match &kvp.key {
                        Symbol::GroupEnd => break,
                        _ => group.properties.push(kvp),
                    }
                } else {
                    self.next_char()?;
                }
            }

            Ok(group)
        }
    }

    pub fn read_object(&mut self) -> Result<Object, Error> {
        if self.is_eof() {
            Err(Error::Eof)
        } else if !self.is_at_object()? {
            Err(Error::Programming(t!(
                "Attempted to read an object when not at an object start"
            )))
        } else {
            let object_start = self.read_key_value_pair_raw()?;

            let mut object: Object = Object {
                name: object_start.value.parse_flag()?,
                properties: vec![],
            };

            while !self.is_eof() {
                if !self.is_blank_line()? {
                    let kvp = self.read_key_value_pair_raw()?;

                    match &kvp.key {
                        Symbol::ObjectEnd => break,
                        _ => object.properties.push(kvp),
                    }
                } else {
                    self.next_char()?;
                }
            }

            Ok(object)
        }
    }
}

/// The primary user-facing PVL structure
pub struct Pvl {
    pub properties: Vec<KeyValuePair>,
    pub groups: Vec<Group>,
    pub objects: Vec<Object>,
}

impl Pvl {
    /// Loads and parses a PVL file from the requested file path
    /// # Example
    /// ```
    /// use vicar::pvl::{Pvl, print_kvp,print_grouping};
    /// use std::path::Path;
    ///
    /// let p = "tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL";
    /// if let Ok(pvl) = Pvl::load(Path::new(p)) {
    ///     pvl.properties.into_iter().for_each(|p| {
    ///     print_kvp(&p, false);
    ///     });
    ///     pvl.groups.into_iter().for_each(|g| {
    ///         print_grouping(&g);
    ///     });
    ///     pvl.objects.into_iter().for_each(|g| {
    ///         print_grouping(&g);
    ///     });
    /// }
    ///
    /// ```
    pub fn load(file_path: &Path) -> Result<Self, Error> {
        match fs::read(file_path) {
            Ok(b) => match String::from_utf8_lossy(&b) {
                Cow::Borrowed(s) => Pvl::from_string(s),
                Cow::Owned(s) => Pvl::from_string(&s),
            },
            Err(why) => Err(Error::General(t!(why))),
        }
    }

    /// Parses the contents of a supplied PVL-formatted String
    /// # Example
    /// ```
    /// use vicar::pvl::{Pvl,print_kvp, print_grouping};
    /// use std::fs;
    ///
    /// let file_path = "tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL";
    /// let s = fs::read_to_string(file_path).expect("Failed to load PVL label");
    /// if let Ok(pvl) = Pvl::from_string(&s) {
    ///     pvl.properties.into_iter().for_each(|p| {
    ///     print_kvp(&p, false);
    ///     });
    ///     pvl.groups.into_iter().for_each(|g| {
    ///         print_grouping(&g);
    ///     });
    ///     pvl.objects.into_iter().for_each(|g| {
    ///         print_grouping(&g);
    ///     });
    /// }
    /// ```
    pub fn from_string(content: &str) -> Result<Self, Error> {
        let mut pvl = Pvl {
            properties: vec![],
            groups: vec![],
            objects: vec![],
        };

        let mut reader = PvlReader::new(content);

        while !reader.is_eof() && !reader.is_at_end() {
            if reader.is_at_multiline_comment_start().unwrap() {
                let _ = reader.skip_multiline_comment().unwrap();
            } else if reader.is_at_line_start().unwrap() && !reader.is_blank_line().unwrap() {
                if reader.is_at_group().unwrap() {
                    pvl.groups.push(reader.read_group().unwrap());
                } else if reader.is_at_object().unwrap() {
                    pvl.objects.push(reader.read_object().unwrap());
                } else if let Ok(kvp) = reader.read_key_value_pair_raw() {
                    if kvp.key == Symbol::End {
                        break;
                    } else {
                        pvl.properties.push(kvp.clone())
                    }
                }
            }
            if !reader.is_eof() && !reader.is_at_end() {
                reader.jump_to_next_line()?;
            }
        }
        Ok(pvl)
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.get_property(name).is_some()
    }

    pub fn get_property(&self, name: &str) -> Option<&KeyValuePair> {
        self.properties.iter().find(|p| match &p.key {
            Symbol::Key(n) | Symbol::Pointer(n) => n == name,
            _ => false,
        })
    }

    pub fn get_group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.name() == name)
    }

    pub fn get_object(&self, name: &str) -> Option<&Object> {
        self.objects.iter().find(|o| o.name() == name)
    }
}

/// Simple utility function to print a KeyValuePair to stdout
pub fn print_kvp(kvp: &KeyValuePair, indent: bool) {
    if indent {
        print!("    ");
    }
    match &kvp.key {
        Symbol::Group | Symbol::Object => {
            println!("GROUP/OBJECT: {:?}", kvp)
        }
        Symbol::Key(v) | Symbol::Pointer(v) => {
            println!("KEY/POINTER: {} -> {:?}", v, kvp.value)
        }
        _ => {}
    };
}

/// Simple utility function to print a GROUP/OBJECT property grouping
/// to stdout
pub fn print_grouping<G: PropertyGrouping>(g: &G) {
    println!("***************************************");
    println!("GROUPING: {}", g.name());
    println!("    TYPE: {:?}", g.type_of());
    g.properties().into_iter().for_each(|kvp| {
        print_kvp(&kvp, true);
    });
    println!("    ** END GROUPING");
}

//let p = "tests/testdata/msl/mahli/3423MH0002970011201599C00_DRCX.LBL";

/// Parses and prints a PVL file to stdout. Nominally for validation/compliance.
pub fn parse_and_print_pvl(file_path: &str) {
    if let Ok(pvl) = Pvl::load(Path::new(file_path)) {
        pvl.properties.into_iter().for_each(|p| {
            print_kvp(&p, false);
        });
        pvl.groups.into_iter().for_each(|g| {
            print_grouping(&g);
        });
        pvl.objects.into_iter().for_each(|g| {
            print_grouping(&g);
        });
    }
}
