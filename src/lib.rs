//! Parse `multipart/form-data` (RFC 7578) from stream.
//! Read and parse headers (like `name`, `filename`), redirect stream for each part of data to user implementation `ProcessContent` trait.
//! To use `MultipartParser` user must implement traits `MultipartParserTarget`, `MultipartParserTargetGenerated` for whole form-data,
//! and implement `ProcessContent` for each form field.
//! Or just use `gnitive-multipart-derive`.

#![feature(vec_remove_item)]
#![feature(try_from)]

pub mod gnitive_multipart
{
    use std::cell::{RefCell};
    use std::rc::{Rc};
    use std::collections::{HashMap};
    use std::io::{Error as IOError};
    use std::num::{ParseIntError, ParseFloatError};
    use std::str::{ParseBoolError};
    use std::string::{FromUtf8Error};


    /// Parameters for processing form field, required for trait `ProcessContent`
    pub struct ProcessParams
    {
        /// Name of field
        pub name: String,

        /// max size of field (in bytes). `Option::None` = unlimited
        pub max_size: Option<usize>
    }


    /// Processing stream of multipart field data
    pub trait ProcessContent
    {
        /// Begin writing field data.
        ///
        /// * `headers` - headers for current field
        fn open(&mut self, headers: &Headers) -> ();

        /// Write `data` of multipart field. May be called many times (fragmentation by network packets, logic of boundary processing etc)
        ///
        /// * `headers` - headers for current field, equal to `headers` in `open`
        /// * `data` - part of multipart field
        fn write(&mut self, headers: &Headers, data: &Vec<u8>) -> ();

        /// Finish writing data. No `write` called for this field after `flush`.
        ///
        /// * `headers` - headers for current field, equal to `headers` in `open` and `write`
        fn flush(&mut self, headers: &Headers) -> ();

        /// Return parameters for processing current field.
        fn get_process_params(&self) -> &ProcessParams;
    }



    /// Type of error, used in `MultipartParserTarget::error` trait.
    pub enum MultipartParseError<'a>
    {
        NoError,

        /// Some of fields, marked as `required=true`, not present in multipart data.
        /// This error fired after finish receiving all data.
        ///
        /// * `Vec<String>` - list of field names, absent in multipart data.
        RequiredMissing(&'a Vec<String>),

        /// Limit of `max_size` was exceeded.
        ///
        /// * `String` - field name
        /// * `usize` - limit
        SizeLimit(String, usize),

        /// * `String` - field name
        /// * `Vec<u8>` - raw data
        /// * `ParseFloatError` - `std::num::ParseFloatError`
        ParseFloatError(String, &'a Vec<u8>, &'a ParseFloatError),

        /// * `String` - field name
        /// * `Vec<u8>` - raw data
        /// * `ParseIntError` - `std::num::ParseIntError`
        ParseIntError(String, &'a Vec<u8>, &'a ParseIntError),

        /// * `String` - field name
        /// * `Vec<u8>` - raw data
        /// * `ParseBoolError` - `std::str::ParseBoolError`
        ParseBoolError(String, &'a Vec<u8>, &'a ParseBoolError),

        /// * `String` - field name
        /// * `FromUtf8Error` - `std::string::FromUtf8Error`
        ParseStrError(String, &'a FromUtf8Error)
    }

    /// Action after processing `MultipartParseError` in `MultipartParserTarget::error`.
    #[derive(PartialEq)]
    pub enum OnError
    {
        /// Continue call write for current field, no `error` call more for this field.
        ContinueWithoutError,

        /// Continue call write for current field, `error` will be fired before every `write`.
        ContinueWithError,

        /// Skip current field, no `write` and `flush` calls for this field.
        Skip,
    }


    /// User must implement this trait for using `MultipartParser`
    pub trait MultipartParserTarget
    {
        /// Select parser for field. All field attributes (ex `name`, `filename`, `charset`) parsed and stored in _headers
        ///
        /// * `_self_` - workaround for generated `ProcessContent`
        /// * `_headers` - all headers for current field
        fn content_parser(&self, _self_: &Rc<RefCell<Self>>, _headers: &Headers) -> Option<Box<ProcessContent>> { None }


        /// Error handling.
        ///
        /// * `_error` - one of `MultipartParseError`.
        ///
        /// Return:
        ///
        /// * `Ok(OnError)` - `MultipartParser` continue read form data, current field may be skipped (`OnError::Skip`) or read more (`OnError::ContinueWithoutError`, OnError::ContinueWithError`)
        /// * `Error(std::io::Error)` - finish read data, `Error` will be return to stream reader
        fn error(&mut self, _error: &MultipartParseError) -> Result<OnError, IOError> { Ok(OnError::ContinueWithoutError) }

        /// Finish of all data, no `content_parser` or `error` will be called.
        fn finish(&mut self);
    }

    /// This trait implements in `gnitive-multipart-derive` crate
    pub trait MultipartParserTargetGenerated
    {
        fn get_all_required(&self) -> Vec<String>;
        fn content_parser_generated(&self, self_: &Rc<RefCell<Self>>, headers: &Headers) -> Option<Box<ProcessContent>>;
    }


    /// Multipart/form-data header
    ///
    /// ```text
    ///   name                 value      fields["name"]      fields["filename"]
    ///    /                    /               \                /
    /// Content-Disposition: form-data; name="file1"; filename="a.txt"
    /// ```
    pub struct Header
    {
        /// Header name (ex.: `Content-Type`, `Content-Disposition`)
        pub name: String,

        /// Header body (ex.: `text/plain`, `form-data`)
        pub value: String,

        /// Rest of header body (ex.: `charset` => `UTF-8`, `filename` => `a.txt`)
        pub fields: HashMap<String, String>
    }


    /// Multipart/form-data headers (for one part of data!)
    pub struct Headers
    {
        /// All headers for this part of data.
        /// Key = header name (ex.: `Content-Type`, `Content-Disposition`)
        pub headers: HashMap<String, Header>
    }


    /// Convert internal Rust parsing error (like `ParseIntError`) to `MultipartParseError`
    pub trait ToMultipartParseError<'a>
    {
        /// * `name` - name of field
        /// * `raw_data` - content of field
        fn to_multipart_parse_error(&'a self, name: String, raw_data: &'a Vec<u8>) -> MultipartParseError;
    }

    impl ProcessParams
    {
        pub fn new<T>(name: T, max_size: Option<usize>) -> ProcessParams
            where T: Into<String>
        {
            let name: String = name.into();
            ProcessParams
                {
                    name,
                    max_size
                }
        }
    }
}


mod boundary_builder;
mod header;
pub mod multipart_parser;
pub mod process_content;
mod to_multipart_parse_error;