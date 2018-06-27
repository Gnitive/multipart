//! Read `boundary` from HTTP header

use rocket;

pub struct Req
{
    pub boundary: Option<String>,
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for Req
{
    type Error = ();

    fn from_request(request: &'a ::rocket::request::Request<'r>) -> ::rocket::request::Outcome<Req, ()>
    {
        let result = Req
            {
                boundary:
                {
                    if let Some (content_type) = request.headers().get_one("Content-Type")
                        {
                            if let Some(idx) = content_type.find("boundary=")
                                {
                                    Some(content_type[(idx + "boundary=".len())..].to_string())
                                }
                                else
                                {
                                    None
                                }
                        }
                        else
                        {
                            None
                        }
                }
            };

        rocket::Outcome::Success(result)
    }
}