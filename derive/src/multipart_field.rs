use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens};
use syn::{Attribute, Expr, Field, Path, PathArguments, Type};
use attributes_utils::{get_string, get_bool, get_usize, ident_to_string, collect_attribute};


/// Wrapper for user field with `#[multipart(...)]`
///
/// ```text
///                  name       max_size            required
///                   /           /                   /
///     #[multipart(name="i", max_size=1073741824, required=false)]
///     pub i: Option<i32>,
///        ~~~ -----------
///        /        /
///   field_name  field_type
/// ```
pub struct MultipartField
{
    /// Field name in multipart struct
    pub field_name: Ident,

    /// Name for generated proxy struct in format `Multipart<StructName><FieldName>`
    pub proxy_struct_name: Ident,

    /// Field type in multipart struct
    pub field_type: Path,

    /// Name in multipart header (optional, default equal `field_name`)
    pub name: String,

    /// Is field required in multipart, default `false`
    pub required: bool,

    /// maximum size of data, default `None` (unlimited)
    pub max_size: Option<usize>,
}

impl MultipartField
{
    /// Convert `variable_name` to `VariableName`
    fn to_camel_case(s: &String) -> String
    {
        let char_to_uppercase = |c: &char| -> char
        {
            c.to_uppercase().nth(0).unwrap()
        };

        let vec: Vec<char> = s.chars().collect();
        let mut result: Vec<char> = vec![];

        let mut need_uppercase: bool = true;
        for c in &vec
            {
                if need_uppercase
                    {
                        result.push( char_to_uppercase(c) );
                        need_uppercase = false;
                    }
                else
                    {
                        need_uppercase = c == &'_';
                        if !need_uppercase
                            {
                                result.push( c.clone() );
                            }
                    }
            }
        let result: String = result.into_iter().collect();
        result
    }


    pub fn new(field: &Field, attribute: &Attribute, struct_name: &Ident) -> MultipartField
    {
        let field_name = match &field.ident
            {
                &None => panic!("Cannot get field name"),
                &Some(ref ident) => ident_to_string(&ident)
            };


        let proxy_struct_name: Ident =
            {
                let tmp = format!("Multipart{}{}"
                                                    , MultipartField::to_camel_case(&ident_to_string(struct_name) )
                                                    , MultipartField::to_camel_case(&field_name));
                Ident::new(tmp.as_str(), Span::call_site())
            };

        let field_type: Path = match field.ty
            {
                Type::Path(ref type_path) => type_path.path.clone(),
                _ => panic!("Only primitive types allowed: (bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64), Option<primitive type>, String, Option<String>, Vec<u8> and Option<Vec<u8>>, cannot process {}", &field_name)
            };

        let mut name = field_name.clone();
        let mut required = false;
        let mut max_size: Option<usize> = None;
        for (ident, lit) in &collect_attribute(&attribute)
            {
                let string_ident = ident_to_string(&ident);
                match string_ident.as_ref()
                    {
                        "name"     => name = get_string(&ident, &lit),
                        "required" => required = get_bool(&ident, &lit),
                        "max_size" => max_size = Some(get_usize(&ident, &lit)),
                        other      => panic!("Unknown multipart attribute '{}' in field '{}'", other, field_name)
                    }
            }

        let field_name = Ident::new(field_name.as_str(), Span::call_site());

        MultipartField
            {
                field_name,
                field_type,
                name,
                proxy_struct_name,
                required,
                max_size,
            }
    }



    /// Generate code line like
    /// `"<name>" => Some(Box::new(<proxy>::new(self_.clone()))),`
    pub fn parser_target_generated_item(&self) -> TokenStream
    {
        let name = self.name.as_str();
        let proxy = &self.proxy_struct_name;

        quote!(
            #name => Some(Rc::new(RefCell::new(#proxy::new(self_.clone())))),
        )
    }

    pub fn parser_required(&self) -> Option<String>
    {
        match self.required
            {
                true =>
                    {
                        let quoted_name = format!("\"{}\".to_string(), ", &self.name);
                        Some(quoted_name)
                    },
                false => None
            }
    }


    ///Generate proxy struct and `impl gnitive_multipart::ProcessContent`
    pub fn impl_process_content(&mut self, target: &Ident) -> TokenStream
    {
        let name = &self.name;

        let max_size = match self.max_size
            {
                None => quote!( None ),
                Some(max_size) => quote!( Some(#max_size) )
            };

        let proxy_name = &self.proxy_struct_name;

        let process_params = quote!(gnitive_multipart::gnitive_multipart::ProcessParams);
        let process_content = quote!(gnitive_multipart::gnitive_multipart::ProcessContent);
        let default_processor = quote!(gnitive_multipart::process_content::DefaultProcessor);

        let field_name = &self.field_name;


        let mut token_stream_field_type = TokenStream::new();
        self.field_type.to_tokens(&mut token_stream_field_type);
        let tokens = quote!(#token_stream_field_type);
        let mut field_type = tokens.to_string();
        // change Option<t> to Option::<t> - avoid "chained comparison operators require parentheses" error
        {
            for segment in &self.field_type.segments
                {
                    if let PathArguments::AngleBracketed(ref _arguments) = segment.arguments
                        {
                            let pos: Option<usize> =
                                {
                                    let mut result: Option<usize> = None;
                                    if let Some(pos) = field_type.find('<')
                                        {
                                            if let Some (chr) = field_type.get(pos + 1..pos + 2)
                                                {
                                                    if chr != ":"
                                                        {
                                                            result = Some(pos);
                                                        }
                                                }
                                        }
                                    result
                                };
                            if pos.is_some()
                                {
                                    field_type.insert_str(pos.unwrap(), "::");
                                }
                            break;
                        }
                }
        }

        field_type.push_str("::try_from");

        let (error_ident, error_exp) = {
            if field_type.find("Vec").is_some()

                {
                    (quote!(_error), quote!())
                }
                else
                {
                    (quote!(error), quote!(let _unused = self.target.borrow_mut().error(&error.to_multipart_parse_error(#name.to_string(), processor.raw_data()));))
                }
        };


        let fixed_path = syn::parse_str::<Expr>(field_type.as_str()).unwrap();
        let field_type = quote!(#fixed_path);



        let proxy_struct_decl : TokenStream = quote!(
            struct #proxy_name
            {
                processor: #default_processor,
                target: Rc<RefCell<#target>>
            }
        );
        let proxy_struct_impl : TokenStream = quote!(

            impl #proxy_name
            {
                pub fn new(target: Rc<RefCell<#target>>) -> Self
                {
                    Self
                        {
                            processor: #default_processor::new( #process_params::new(#name, #max_size) ),
                            target: target.clone()
                        }
                }
            }
        );

        let fn_open: TokenStream = quote!(
            fn open(&mut self, headers: &Headers) -> ()
            {
                self.processor.open(headers);
            }
        );

        let fn_write: TokenStream = quote!(
            fn write(&mut self, headers: &Headers, data: &Vec<u8>) -> ()
            {
                self.processor.write(headers, data);
            }
        );


        let fn_flush: TokenStream = quote!(
            fn flush(&mut self, headers: &Headers) -> ()
            {
                self.processor.flush(headers);
                let processor = &self.processor;

                let result = #field_type(processor);
                match result
                {
                    Ok(value) => self.target.borrow_mut().#field_name = value,
                    Err(#error_ident) =>
                    {
                        #error_exp
                    }
                }
            });

        let fn_get_process_params: TokenStream = quote!(
            fn get_process_params(&self) -> &#process_params
            {
                self.processor.get_process_params()
            });

        let proxy_struct_impl_process_content: TokenStream = quote!(

            impl #process_content for #proxy_name
            {
                #fn_open
                #fn_write
                #fn_flush
                #fn_get_process_params
            }
        );

        quote!(
            #proxy_struct_decl
            #proxy_struct_impl
            #proxy_struct_impl_process_content
        )
    }
}
