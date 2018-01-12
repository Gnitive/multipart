//! Parse and store multipart field header

use std::collections::{HashMap};
use std::fmt;
use ::gnitive_multipart::{Header, Headers};


impl Header
{
    fn new(s: &str) -> Header
    {
        let mut strings: Vec<&str>  = s.split(';').collect();

        let first = strings.remove(0);
        let (name, value) =  Header::to_key_value(first.as_ref(), ':');

        let mut fields: HashMap<String,String> = HashMap::new();
        for string in strings
            {
                let (key, mut value) = Header::to_key_value(string, '=');
                let value = value.trim_matches('"').to_string();
                fields.insert(key, value);
            }

        Header
            {
                name,
                value,
                fields
            }
    }


    /// Split `s` by `separator` into 2 `String`
    fn to_key_value(s: &str, separator: char) -> (String, String)
    {
        let strings: Vec<&str> = s.split(separator).collect();
        if strings.len() != 2
            {
                panic!("Cannot parse header part '{}' with separator '{}'", s, separator);
            }
        (strings[0].trim().to_string(), strings[1].trim().to_string())
    }
}

impl fmt::Display for Header
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = format!("{}: {}", self.name, self.value);
        if self.fields.len() > 0
            {
                for (key, value) in self.fields.iter()
                    {
                        result.push_str(format!("; {}=\"{}\"", key, value).as_ref());
                    }
            }
        write!(f, "{}", result)
    }
}


impl Headers
{
    pub fn new(header_lines: &Vec<String>) -> Headers
    {
        let mut headers: HashMap<String, Header> = HashMap::new();
        for line in header_lines
            {
                let header = Header::new(line);
                headers.insert(header.name.clone(), header);
            }

        Headers
            {
                headers
            }
    }

    /// Get value from header body.
    /// Ex: get "name" from multipart data part.
    ///
    /// ```rust,ignore
    /// let headers: Headers = ...
    /// let s: String = headers.get("Content-Disposition", "name").unwrap();
    /// println!("{}", s);
    /// ```
    ///
    /// Output:
    ///
    /// ```text
    /// text1
    /// ```
    #[allow(dead_code)]
    pub fn get<S: Into<String>>(&self, name: S, field_name: S) -> Option<&String>
    {
        let name: String = name.into();
        let field_name: String = field_name.into();
        match self.headers.get(&name)
            {
                None => None,
                Some(header) =>
                    {
                        header.fields.get(&field_name)
                    }
            }
    }

    /// Get `name` from header
    #[allow(dead_code)]
    pub fn get_name(&self) -> Option<&String>
    {
        self.get("Content-Disposition", "name")
    }

    /// Get `filename` from header
    #[allow(dead_code)]
    pub fn get_filename(&self) -> Option<&String>
    {
        self.get("Content-Disposition", "filename")
    }
}


/// Build headers for multipart field data from internal temporary buffer.
pub struct HeadersBuilder
{
    tmp: Vec<u8>,
    lines: Vec<String>,
}

impl HeadersBuilder
{
    pub fn new() -> Self
    {
        Self
            {
                tmp: vec![],
                lines: vec![]
            }
    }

    pub fn write(&mut self, c: u8) -> ()
    {
        self.tmp.push(c);
    }

    pub fn flush(&mut self) -> ()
    {
        match String::from_utf8(self.tmp.clone())
            {
                Ok(line) => self.lines.push(line),
                Err(_) => ()
            }
        self.tmp.clear();
    }

    pub fn build(&mut self) -> Headers
    {
        let result = Headers::new(&self.lines);
        self.lines.clear();
        self.tmp.clear();
        result
    }
}

#[cfg(test)]
mod tests
{
    use super::{Headers};

    #[test]
    fn headers() -> ()
    {
        let v: Vec<String> = vec![
            "Content-Disposition: form-data; name=\"file1\"; filename=\"a.txt\"".to_string(),
            "Content-Type: application/octet-stream".to_string(),
        ];

        let headers = Headers::new(&v);
        assert_eq!("file1", headers.get_name().unwrap());
        assert_eq!("a.txt", headers.get_filename().unwrap());
    }
}