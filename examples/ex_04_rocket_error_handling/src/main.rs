//! Read `multipart/form-data` content from POST form (see `static/index.html`),
//! save all files to `/tmp/` (see `impl ProcessContent` and `impl MultipartParserTarget`),
//! and parse rest of data into struct `Test`.
//! Handle all errors - `max_size` for content, missing fields, conversion error.

#![feature(plugin, decl_macro, try_from, attr_literals, match_default_bindings)]
#![plugin(rocket_codegen)]
#![allow(unused_imports)]

extern crate rocket;
#[macro_use]
extern crate gnitive_multipart_derive;
extern crate gnitive_multipart;
use gnitive_multipart::multipart_parser::{MultipartParser};
use gnitive_multipart::gnitive_multipart::{MultipartParserTarget, ProcessContent, Headers, OnError, MultipartParseError};

use rocket::response::{NamedFile};
use rocket::{Data};
use std::cell::{RefCell};
use std::io::{Cursor, Result, Error as IOError};
use std::rc::{Rc};

mod req;
use req::{Req};

mod file_writer;
use file_writer::{FileWriter};


#[get("/")]
fn index() -> Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[post("/form", data = "<data>")]
fn form<'a>(data: Data, request: Req)  -> rocket::Response<'a> {
    let s =
            match request.boundary
                {
                    Some(boundary) =>
                        {
                            let test = Test::new();
                            let target: Rc<RefCell<Test>> = Rc::new(RefCell::new(test));
                            let clone = target.clone();

                            let mut multipart_parser: MultipartParser<Test> = MultipartParser::new_from_str(boundary.as_str(), &target);
                            data.stream_to(&mut multipart_parser).unwrap();

                            let test = clone.borrow();
                            test.to_html()
                        }
                    None =>
                        {
                            "<html><body><p>Cannot found boundary</p></body></html>".to_string()
                        }
                };

    let mut result = rocket::Response::new();
    result.set_sized_body(Cursor::new(s));
    result
}



#[derive(MultipartDerive)]
#[multipart(dump=false)]
struct Test
{
    /// just integer
    #[multipart(required=true)]
    pub i: i32,

    /// string may be passed in form - `ParseIntError` will be fired
    #[multipart(required=true)]
    pub invalid_i: i32,

    /// just float
    #[multipart(required=true)]
    pub f: f32,

    /// string may be passed in form - `ParseFloatError` will be fired
    #[multipart(required=true)]
    pub invalid_f: f32,

    #[multipart(name="text1")]
    pub s: String,

    /// this field marked as `required`, but not present in form - `RequiredMissing` error will be fired
    #[multipart(name="missing_field_1", required=true)]
    missing_field_1: Vec<u8>,

    /// this field not present in form too - `RequiredMissing` error will be fired
    #[multipart(name="missing_field_2", required=true)]
    missing_field_2: Vec<u8>,

    writers: Vec<Rc<RefCell<FileWriter>>>,

    /// all errors will be dumped into `stdout` and html
    errors: String
}


impl Test{


    pub fn new() -> Test
    {
        Test
            {
                i: 0,
                invalid_i: 99,

                f: 0.0,
                invalid_f: 0.0,

                s: String::new(),

                missing_field_1: vec![],
                missing_field_2: vec![],

                writers: vec![],
                errors: String::new()
            }
    }

    fn vec_to_string (vec: &Vec<u8>) -> String
    {
        match String::from_utf8(vec.clone())
            {
                Ok(string) => string,
                Err(_) => format!("{:?}", &vec)
            }
    }

    fn add_error(&mut self, err: String) -> ()
    {
        self.errors.push_str(err.as_str());
        println!("{}", &err);
    }

    pub fn to_html(&self) -> String
    {
        let value_i = self.i.to_string();
        let value_invalid_i = self.invalid_i.to_string();

        let value_f = self.f.to_string();
        let value_invalid_f = self.invalid_f.to_string();

        let value_s: String = self.s.clone();

        let mut writers = String::new();
        for writer in &self.writers
            {
                writers.push_str(writer.borrow().to_html().as_str() );
            }

        format!(
            r#"
<!DOCTYPE HTML>
<html>
<head>
	<meta charset="utf-8">
    <title>ex_04_rocket_error_handling</title>
</head>
<body>
    <table rules="all">
        <tr><th>Field</th> <th>Value</th></tr>

        <tr><td>i</td>          <td>{value_i}</td></tr>
        <tr><td>invalid_i</td>  <td>{value_invalid_i}</td></tr>

        <tr><td>f</td>          <td>{value_f}</td></tr>
        <tr><td>invalid_f</td>  <td>{value_invalid_f}</td></tr>


        <tr><td>s</td>     <td>{value_s}</td></tr>
        {writers}
    </table>
        <h3>Errors</h3>
        {errors}
</body>
</html>"#,
            value_i = value_i,
            value_invalid_i = value_invalid_i,

            value_f = value_f,
            value_invalid_f = value_invalid_f,

            value_s = value_s,
            writers = writers,
            errors = self.errors)
    }
}


impl MultipartParserTarget for Test
{

    fn error(&mut self, error: &MultipartParseError) -> Result<OnError>
    {
        match error
            {
                // some fields not present in POST data (`missing_field_1` and `missing_field_2` in this exampe)
                &MultipartParseError::RequiredMissing(missing_fields) =>
                    {
                        for missing_field in missing_fields
                            {
                                self.add_error(format!("<p>Missing field <b>{}</b></p>", &missing_field))
                            }
                        // all POST data parsed - we can return any `OnError` - it will be ignored
                        Ok(OnError::ContinueWithoutError)

                    }

                &MultipartParseError::SizeLimit(ref _field_name, ref _max_size) =>
                    {
                        Ok(OnError::ContinueWithoutError)
                    }

                // cannot parse form field as float
                &MultipartParseError::ParseFloatError(ref field_name, raw_data, ref _parse_float_error) =>
                    {
                        let raw = Test::vec_to_string(raw_data);
                        self.add_error(format!("<p>Cannot parse <b>{}</b> field as float. Raw data: <b>{:?}</b></p>", field_name, raw));
                        Ok(OnError::ContinueWithoutError)
                    }

                // cannot parse form field as int
                &MultipartParseError::ParseIntError(ref field_name, raw_data, ref _parse_int_error) =>
                    {
                        let raw = Test::vec_to_string(raw_data);
                        self.add_error(format!("<p>Cannot parse <b>{}</b> field as int. Raw data: <b>{:?}</b></p>", field_name, raw));
                        Ok(OnError::ContinueWithoutError)
                    }

                &MultipartParseError::ParseBoolError(ref _field_name, ref _raw_data, ref _parse_bool_error) =>
                    {
                        Ok(OnError::ContinueWithoutError)
                    }

                &MultipartParseError::ParseStrError(ref _field_name, ref _parse_str_error) =>
                    {
                        Ok(OnError::ContinueWithoutError)
                    }


                &MultipartParseError::NoError => { Ok(OnError::ContinueWithoutError) }
            }
    }


    fn content_parser(&mut self, _self_: &Rc<RefCell<Self>>, headers: &Headers) -> Option<Rc<RefCell<ProcessContent>>>
    {
        if let Some(name) = headers.get_name()
            {
                match name.as_str()
                    {
                        "file1" |
                        "file2" |
                        "file3" =>
                            {
                                let file_writer = FileWriter::new(name);
                                let rc = Rc::new(RefCell::new(file_writer));
                                self.writers.push(rc.clone());
                                Some (rc)
                            }
                        _ => None
                    }
            }
        else
        {
            None
        }
    }
}


fn main() {
    rocket::ignite().mount("/", routes![index, form]).launch();
}
