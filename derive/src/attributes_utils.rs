use proc_macro2::{TokenStream, Span};
use quote::{ToTokens};
use std::any::{Any};
use syn::{Attribute, Ident, Lit, Meta, NestedMeta};


/// Get all attributes from `macro_name` macro declaration to list of `(name, value)` tuples
///
/// Input:
///
/// ```text
///   ignored
///      /
/// #derive(Debug)
///
///   ignored
///      /
/// #[another_macro]
///
///             name    value      name      value         name    value
///               /      /          /         /             /       /
/// #[multipart(name="file12",   max_size=1073741824,    required=true)]
///```
/// Output:
///
/// ```text
/// [
///     ("name", "file12"),
///     ("max_size", 1073741824),
///     ("required", true)
/// ]
///```
pub fn collect_attributes(macro_name: &str, attributes: &Vec<Attribute>) -> Vec<(Ident, Lit)>
{
    let attribute = find_attribute(macro_name, attributes);
    match attribute
        {
            None => panic!(format!("Cannot found '{}'", macro_name)),
            Some(attribute) =>
                {
                    collect_attribute(&attribute)
                }
        }
}


/// Find attribute in `attributes` with `name` = `macro_name`
///
pub fn  find_attribute<'a>(macro_name: &str, attributes: &'a Vec<Attribute>) -> Option<&'a Attribute>
{
    attributes
        .iter()
        .find(|attr| attr.interpret_meta().unwrap().name().eq(macro_name))

}



/// Get all attributes from `attribute` to list of `(name, value)` tuples
///
/// Input:
///
/// ```text
///             name    value      name      value         name    value
///               /      /          /         /             /       /
/// #[multipart(name="file12",   max_size=1073741824,    required=true)]
///```
///
/// Output:
///
/// ```text
/// [
///     ("name", "file12"),
///     ("max_size", 1073741824),
///     ("required", true)
/// ]
/// ```
pub fn collect_attribute(attribute: &Attribute) -> Vec<(Ident, Lit)>
{
    let item_to_ident_lit= |item: &NestedMeta| -> Option<(Ident, Lit)>
    {
        if let &NestedMeta::Meta(ref meta) = item
            {
                if let &Meta::NameValue(ref meta_name_value) = meta
                    {
                        return Some((meta_name_value.ident.clone(), meta_name_value.lit.clone()))
                    }
            };
        None
    };


    let mut result: Vec<(Ident, Lit)> = vec![];
    if let Some(ref meta) = attribute.interpret_meta()
        {
            if let Meta::List(ref meta_list) = meta
                {
                    let iter = meta_list.nested
                                       .iter()
                                       .filter_map(|item| item_to_ident_lit(&item));
                    result = iter.collect();

                }
        }
    result
}



/// Get `String` value from macro attribute
///
/// ```text
///           ident   result
///              \      /
/// #[multipart(name="file12")
/// ```
#[allow(dead_code)]
pub fn get_string(ident: &Ident, lit: &Lit) -> String
{
    if let Lit::Str(ref lit_str) = lit
        {
            return lit_str.value().clone();
        }
        else
        {
            panic!("'{}' must be string, but '{}' found", ident, lit_to_string(lit));
        }
}



/// Get `bool` value from macro attribute
///
/// ```text
///            ident    result
///             \       /
/// #[multipart(debug=true)
/// ```
#[allow(dead_code)]
pub fn get_bool(ident: &Ident, lit: &Lit) -> bool
{
    if let Lit::Bool(lit_bool) = lit
        {
            return lit_bool.value.clone();
        }
        else
        {
            panic!("'{}' must be bool, but '{}' found", ident, lit_to_string(lit))
        }
}



/// Get `int` value from macro attribute
/// ```text
///            ident  result
///              \     /
/// #[multipart(count=12)
/// ```
#[allow(dead_code)]
pub fn get_int(ident: &Ident, lit: &Lit) -> u64
{
    if let Lit::Int(lit_int) = lit
        {
            return lit_int.value().clone();
        }
        else
        {
            panic!("'{}' must be int, but '{}' found", ident, lit_to_string(lit))
        }
}



/// Get `usize` value from macro attribute
/// ```text
///             ident       result
///               \          /
/// #[multipart(max_size=1048576)
#[allow(dead_code)]
pub fn get_usize(ident: &Ident, lit: &Lit) -> usize
{
    get_int(&ident, &lit) as usize
}



/// Get `Ident` value from macro attribute
/// ```text
///               ident       result
///                  \          /
/// #[multipart(variable_name="i")
/// ```
#[allow(dead_code)]
pub fn get_ident(ident: &Ident, lit: &Lit) -> Ident
{
    let s = get_string(ident, lit);
    let result: Ident = Ident::new(s.as_str(), Span::call_site());
    result
}


/// Get `Any` value from macro attribute
/// ```text
///              ident        result
///                 \          /
///  #[multipart(default="Option::None")
/// ```
#[allow(dead_code)]
pub fn get_any(_ident: &Ident, lit: &Lit) -> Box<Any>
{
    match lit
        {
            Lit::Str(lit_str) =>
                {
                    Box::new(lit_str.value().clone())
                }
            Lit::ByteStr(_lit_byte_str) =>
                {
                    panic!("Unimplemented")
                }
            Lit::Byte(lit_byte) =>
                {
                    Box::new(lit_byte.value().clone())
                }
            Lit::Char(lit_char) =>
                {
                    Box::new(lit_char.value().clone())
                }
            Lit::Int(lit_int) =>
                {
                    Box::new(lit_int.value().clone())
                }
            Lit::Float(lit_float) =>
                {
                    Box::new(lit_float.value().clone())
                }
            Lit::Bool(lit_bool) =>
                {
                    Box::new(lit_bool.value.clone())
                }
            Lit::Verbatim(_lit_verbatim) =>
                {
                    panic!("Unimplemented")
                }
        }
}

pub fn ident_to_string(ident: &Ident) -> String
{
    let mut ts = TokenStream::new();
    ident.to_tokens(&mut ts);
    ts.to_string()
}


/// For debug purpose
fn lit_to_string(lit: &Lit) -> &str
{
    match lit
        {
            Lit::Str(_) => "Str",
            Lit::ByteStr(_) => "ByteStr",
            Lit::Byte(_) => "Byte",
            Lit::Char(_) => "Char",
            Lit::Int(_) => "Int",
            Lit::Float(_) => "Float",
            Lit::Bool(_) => "Bool",
            Lit::Verbatim(_) => "Verbatim",
        }
}
