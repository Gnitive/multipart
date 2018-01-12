//! Convert internal Rust parsing error (like `ParseIntError`) to `MultipartParseError`

use ::gnitive_multipart::{MultipartParseError, ToMultipartParseError};
use std::convert::{Infallible};
use std::string::{FromUtf8Error};
use std::str::{ParseBoolError};
use std::num::{ParseIntError, ParseFloatError};

impl <'a>ToMultipartParseError<'a> for Infallible
{
    fn to_multipart_parse_error(&'a self, _name: String, _raw_data: &'a Vec<u8>) -> MultipartParseError
    {
        MultipartParseError::NoError
    }
}

impl <'a>ToMultipartParseError<'a> for FromUtf8Error
{
    fn to_multipart_parse_error(&'a self, name: String, _raw_data: &'a Vec<u8>) -> MultipartParseError
    {
        MultipartParseError::ParseStrError(name, self)
    }

}

impl <'a>ToMultipartParseError<'a> for ParseBoolError
{
    fn to_multipart_parse_error(&'a self, name: String, raw_data: &'a Vec<u8>) -> MultipartParseError
    {
        MultipartParseError::ParseBoolError(name, raw_data, self)
    }
}


impl <'a>ToMultipartParseError<'a> for ParseIntError
{
    fn to_multipart_parse_error(&'a self, name: String, raw_data: &'a Vec<u8>) -> MultipartParseError
    {
        MultipartParseError::ParseIntError(name, raw_data,self)
    }
}

impl <'a>ToMultipartParseError<'a> for ParseFloatError
{
    fn to_multipart_parse_error(&'a self, name: String, raw_data: &'a Vec<u8>) -> MultipartParseError
    {
        MultipartParseError::ParseFloatError(name, raw_data,self)
    }
}
