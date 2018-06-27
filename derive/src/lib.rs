//! Generate implementation of `gnitive_multipart::MultipartParserTargetGenerated` trait for
//! user struct and `gnitive_multipart::ProcessContent` for each struct field marked as `#[multipart()]`.
//!
//! ```toml
//! [dependencies]
//! gnitive-multipart-derive = "0.3"
//! gnitive-multipart = "0.3"
//! ```
//!
//! ```rust,ignore
//! #![feature(attr_literals)]
//! #![feature(try_from)]
//! #[macro_use]
//! extern crate gnitive_multipart_derive;
//! extern crate gnitive_multipart;
//! use gnitive_multipart::multipart_parser::MultipartParser;
//! use gnitive_multipart::gnitive_multipart::{MultipartParserTarget, MultipartParseError,  ProcessContent, OnError, Headers, ToMultipartParseError};
//! use std::convert::TryFrom;
//!
//! #[derive(MultipartDerive)]
//! #[multipart(debug=false)]
//! struct Test
//! {
//!     #[multipart(name="i", required=false)]
//!     pub i: Option<i32>,
//!
//!     #[multipart(name="file1")]
//!     pub file1: Vec<u8>,
//!
//!     #[multipart(name="file12", max_size=1073741824, required=true)]
//!     pub file2: Vec<u8>,
//!
//!     #[multipart(name="file3", required=false)]
//!     pub file3: Option<Vec<u8>>,
//!
//!     #[multipart(name="text1")]
//!     pub s: String,
//! }
//! ```
//!
//! # Struct attributes
//!
//! `#[multipart(debug=false)]`
//!
//! ## `debug`
//!
//! Dump all generated code to stdio.
//!
//! *Type*: `bool`.
//!
//! *Required*: `false`.
//!
//! *Default*: `false`.
//!
//! In case when `debug=true` all generated code will be dumped to stdio at compilation time.
//! Can be useful for debug.
//!
//! ### Example
//!
//! ```rust,ignore
//! #[derive(MultipartDerive)]
//! #[multipart(debug=true)]
//! struct Test
//! {
//!     #[multipart(name="file")]
//!     pub file: Vec<u8>,
//! }
//! ```
//! After execute command `cargo build` this code will be displayed:
//!
//! <details><p>
//!
//! ```rust,ignore
//! impl gnitive_multipart::gnitive_multipart::MultipartParserTargetGenerated for Test
//! {
//!     fn content_parser_generated(&self, self_: &Rc<RefCell<Self>>, headers: &Headers) -> Option<Box<ProcessContent>>
//!     {
//!         let name = headers.get_name().unwrap().as_ref();
//!         match name
//!             {
//!                 "file" => Some(Box::new(MultipartTestFile::new(self_.clone()))),
//!                 _ => self.content_parser(self_, headers)
//!             }
//!     }
//! }
//!
//! struct MultipartTestFile
//! {
//!     processor: gnitive_multipart::process_content::DefaultProcessor,
//!     target: Rc<RefCell<Test>>
//! }
//!
//! impl MultipartTestFile
//! {
//!     pub fn new(target: Rc<RefCell<Test>>) -> Self
//!     {
//!         Self
//!             { processor: gnitive_multipart::process_content::DefaultProcessor::new(gnitive_multipart::gnitive_multipart::ProcessParams::new("file", None)),
//!                 target: target.clone()
//!             }
//!     }
//! }
//!
//! impl gnitive_multipart::gnitive_multipart::ProcessContent for MultipartTestFile
//! {
//!     fn open(&mut self, headers: &Headers) -> ()
//!     {
//!         self.processor.open(headers);
//!     }
//!
//!     fn write(&mut self, headers: &Headers, data: &Vec<u8>) -> ()
//!     {
//!         self.processor.write(headers, data);
//!     }
//!
//!     fn flush(&mut self, headers: &Headers) -> ()
//!     {
//!         self.processor.flush(headers);
//!         let processor = &self.processor;
//!         let result = Vec::<u8>::try_from(processor);
//!         match result
//!             {
//!                 Ok(value) => self.target.borrow_mut().file = value,
//!                 Err(error) => { let _unused = self.target.borrow_mut().error(&error.to_multipart_parse_error("#name".to_string(), processor.raw_data())); }
//!             }
//!     }
//!
//!     fn get_process_params(&self) -> &gnitive_multipart::gnitive_multipart::ProcessParams
//!     {
//!         self.processor.get_process_params()
//!     }
//! }
//! ```
//!
//! </details>
//!
//! # Field attributes
//!
//! `#[multipart(name="file", max_size=1073741824, required=true)]`
//!
//! ## `name`
//!
//! Name, as presented in `content-disposition: form-data; name=<name>` header.
//!
//! *Type*: `String`.
//!
//! *Required*: `false`.
//!
//! *Default*: same as field name.
//!
//! If `name` is not present in macro attributes, field name will be used.
//!
//! ### Example 1 (without `name`)
//!
//! ```rust,ignore
//! #[derive(MultipartDerive)]
//! #[multipart(debug=true)]
//! struct Test
//! {
//!     #[multipart]
//!     pub i: i32,
//! }
//! ```
//!
//! This struct can handle this html form:
//!
//! ```text
//! <form enctype="multipart/form-data">
//!     <input type="number" name="i" min="1" max="5">
//! </form>
//! ```
//!
//! ### Example 2 (with `name`)
//!
//! ```rust,ignore
//! #[derive(MultipartDerive)]
//! #[multipart(debug=true)]
//! struct Test
//! {
//!     #[multipart(name="count")]
//!     pub i: i32,
//! }
//! ```
//!
//! This struct can handle this html form:
//!
//! ```text
//! <form enctype="multipart/form-data">
//!     <input type="number" name="count" min="1" max="5">
//! </form>
//! ```
//!
//! ## `max_size`
//!
//! Maximum size (in bytes) of current field. Can be usable for process multiple file upload.
//!
//! *Type*: `usize`.
//!
//! *Required*: `false`.
//!
//! *Default*: unlimited.
//!
//! If content of field exceed `max_size`, `MultipartParserTarget::error` will be called.
//!
//! ## `required`
//!
//! If `true` and content is not present in form data, `MultipartParserTarget::error` will be called after processing of all data.
//!
//! *Type*: `bool`.
//!
//! *Required*: `false`.
//!
//! *Default*: `false`.
//!
//! ### Example
//!
//! <details><p>
//!
//! ```rust,ignore
//! #[derive(MultipartDerive)]
//! #[multipart(debug=true)]
//! struct Test
//! {
//!     #[multipart(name="i", required=true)]
//!     pub i: i32,
//!
//!     #[multipart(name="file")]
//!     pub file: Vec<u8>,
//!
//!     #[multipart(name="file12", max_size=1073741824, required=true)]
//!     pub file2: Vec<u8>,
//!
//!     #[multipart(name="file3", required=false)]
//!     pub file3: Option<Vec<u8>>,
//!
//!     #[multipart(name="text1")]
//!     pub s: String,
//! }
//!
//!
//! impl MultipartParserTarget for Test
//! {
//!     fn error(&mut self, error: &MultipartParseError) -> Result<OnError, IOError>
//!     {
//!         match error
//!             {
//!                 &MultipartParseError::RequiredMissing(missing_fields) =>
//!                     {
//!                         for missing_field in missing_fields
//!                             {
//!                                 println!("Required field '{}' missing in payload", missing_field);
//!                             }
//!                     },
//!                 _ => ()
//!             }
//!         Ok(OnError::ContinueWithoutError)
//!     }
//! }
//! ```
//!
//! And upload form like
//!
//! ```text
//! <form enctype="multipart/form-data">
//!     <input type="number" name="i" min="1" max="5">
//!     <input type="file" name="file">
//! </form>
//! ```
//!
//! Shows this output:
//!
//! ```text
//! Required field 'file12' missing in payload
//! ```
//!
//! Because the `i` and `file12` fields marked as `required=true`, but only `i` present in form, and another fields marked as `required=false`.
//!
//! </details>
//!
//!
//! # Field type
//!
//! Field can be one of those types:
//!
//! * Integer: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
//! * Optional integer: `Option<u8>`, `Option<u16>`, `Option<u32>`, `Option<u64>`, `Option<i8>`, `Option<i16>`, `Option<i32>`, `Option<i64>`
//! * Float and optional float: `f32`, `f64`, `Option<f32>`, `Option<f64>`
//! * Bool and optional bool: `bool`, `Option<bool>`
//! * String and optional string: `String`, `Option<String>`
//! * Vectors: `Vec<u8>`, `Option<Vec<u8>>`
#![feature(proc_macro)]
#![recursion_limit = "128"]
#![feature(extern_prelude)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{TokenStream};
use quote::{TokenStreamExt};
use syn::{DeriveInput};

mod multipart_field;
mod multipart_struct;
mod attributes_utils;

use multipart_struct::{MultipartStruct};

#[proc_macro_derive(MultipartDerive, attributes(multipart))]
pub fn multipart(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let ast: DeriveInput = syn::parse(input).unwrap();

    let multipart_struct = MultipartStruct::new(&ast);
    let multipart_parser_target_generated = multipart_struct.impl_multipart_parser_target_generated();

    let mut process_contents = TokenStream::new();
    for mut filed_attribute in multipart_struct.fields
        {
            let process_content = filed_attribute.impl_process_content(&multipart_struct.name);
            process_contents.append_all(process_content);
        }

    let result: TokenStream = quote!(
        #multipart_parser_target_generated
        #process_contents
    );

    if multipart_struct.debug
        {
            let s = result.to_string();
            println!("-------------- <multipart> --------------");
            println!("{}", s);
            println!("-------------- </multipart> --------------");
        }

    result.into()
}
