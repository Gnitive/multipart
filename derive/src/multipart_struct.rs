use syn::{Ident, DeriveInput, Data, Fields};
use quote::{TokenStreamExt};
use proc_macro2::{TokenStream};
use multipart_field::{MultipartField};
use attributes_utils::{collect_attributes, get_bool, ident_to_string, find_attribute};


/// Wrapper for user struct with `#[derive(MultipartDerive)]`
pub struct MultipartStruct
{
    /// Struct name
    pub name: Ident,

    /// Value of `debug` attribute in `#[multipart()]`, default `false`
    pub debug: bool,

    /// All fields, marked with `#[multipart()]`
    pub fields: Vec<MultipartField>
}

impl MultipartStruct
{
    pub fn new (ast: &DeriveInput) -> Self
    {
        let name = ast.ident.clone();
        let mut debug = false;
        for (ident, lit) in collect_attributes("multipart",&ast.attrs)
            {
                let string_ident = ident_to_string(&ident);
                match string_ident.as_str()
                    {
                        "debug" =>
                            {
                                debug = get_bool(&ident, &lit);
                            },
                        _ =>
                            {
                                panic!("Unknown attribute '{}' in struct '{}'", &string_ident, &ast.ident);
                            }
                    }
            }

        let fields: Vec<MultipartField> =
            {
                if let &Data::Struct(ref data_struct) = &ast.data
                    {
                        if let Fields::Named(ref fields_named) = data_struct.fields
                            {
                                fields_named.named
                                    .iter()
                                    .filter_map(
                                        |field|
                                            match find_attribute("multipart", &field.attrs)
                                                {
                                                    None => None,
                                                    Some(attr) => Some(MultipartField::new(field, attr, &name))
                                                })
                                    .collect()
                            }
                            else
                            {
                                panic!("Only struct allowed");
                            }
                    }
                    else
                    {
                        panic!("Only struct allowed");
                    }
            };


        MultipartStruct
            {
                name,
                debug,
                fields
            }
    }



    /// Generate impl trait `MultipartParserTargetGenerated`
    pub fn impl_multipart_parser_target_generated(&self) -> TokenStream
    {
        let mut matches = TokenStream::new();
        for field in &self.fields
            {
                let tokens = field.parser_target_generated_item();
                matches.append_all(tokens);
            }

        let mut required = TokenStream::new();
        for field in &self.fields
            {
                if let Some(_tokens) = field.parser_required()
                    {
                        required.append_all(quote!(_tokens));
                    }
            }

        let name = &self.name;

        let trait_name: TokenStream = quote!(gnitive_multipart::gnitive_multipart::MultipartParserTargetGenerated);

        quote!(
            impl #trait_name for #name
            {
                fn get_all_required(&self) -> Vec<String>
                {
                    let result: Vec<String> = vec![#required];
                    result
                }

                fn content_parser_generated(&mut self, self_: &Rc<RefCell<Self>>, headers: &Headers) -> Option<Rc<RefCell<ProcessContent>>>
                {
                    let name = headers.get_name().unwrap().as_ref();

                    match name
                        {
                            #matches
                            _ => self.content_parser(self_, headers)
                        }
                }

            }
        )
    }
}