//! Write form data content to file
use gnitive_multipart::gnitive_multipart::{ProcessContent, ProcessParams, Headers};

use std::env;
use std::fs::{File};
use std::io::prelude::*;
use std::path::{PathBuf};


pub struct FileWriter
{
    /// path to file
    path: PathBuf,

    file: Option<File>,

    /// size of writed bytes
    size: usize,

    /// require for `ProcessContent` trait
    process_params: ProcessParams
}


impl FileWriter
{
    pub fn new(name: &String) -> Self
    {
        // default file name - used if `filename` not present in form data
        let mut path = env::temp_dir();
        path.push("upload.tmp");

        FileWriter
            {
                path,
                file: None,
                size: 0,
                process_params: ProcessParams::new(name.clone(), None)
            }
    }


    /// Dump `FileWriter` content to html table row
    pub fn to_html(&self) -> String
    {
        let path = self.path.to_str().unwrap().to_string();
        format!("<tr><td>{name}</td><td>{path} ({size} bytes)</tr>",
                name=self.process_params.name,
                path=path,
                size=self.size)
    }
}


impl ProcessContent for FileWriter
{
    /// Start write data to file
    fn open(&mut self, headers: &Headers) -> ()
    {
        // try get filename from request headers
        if let Some(filename) = headers.get_filename()
            {
                if !filename.is_empty()
                    {
                        let mut path = env::temp_dir();
                        path.push(&filename);
                        self.path = path;
                    }
            }
        self.file = Some(File::create(&self.path).unwrap());
    }

    fn write(&mut self, _headers: &Headers, data: &Vec<u8>) -> ()
    {
        if let Some(ref mut file) = self.file
            {
                file.write(data).unwrap();
                self.size += data.len();
            }
    }

    fn flush(&mut self, _headers: &Headers) -> ()
    {
        if let Some(ref mut file) = self.file
            {
                file.flush().unwrap();
            }
        self.file = None;
    }

    /// Return parameters for processing current field.
    fn get_process_params(&self) -> &ProcessParams
    {
        &self.process_params
    }
}