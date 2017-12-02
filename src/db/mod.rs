#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "rusqlite")]
mod rusqlite;

use syn;

use regex::Regex;

#[cfg(feature = "postgres")]
pub use self::postgres::*;
#[cfg(feature = "rusqlite")]
pub use self::rusqlite::*;

pub fn get_id_type(fields: &[syn::Field]) -> &syn::Ty {
    let raw_id_type = fields
            .iter()
            .filter(|f| f.ident.as_ref().map(|ident| ident == "id").unwrap_or(false))
            .map(|f| &f.ty)
            .next()
            .expect("Expected 'id' field");

    unwrap_option_type(raw_id_type)
}

/// Unwrap Option<T> to T
pub fn unwrap_option_type(outer_type: &syn::Ty) -> &syn::Ty {
    if let &syn::Ty::Path(_, ref path) = outer_type {
        if !path.segments.is_empty() {
            let segment = &path.segments[0];
            if segment.ident == syn::Ident::from("Option") {
                if let syn::PathParameters::AngleBracketed(ref params) = segment.parameters {
                    return &params.types[0];
                } else {
                    panic!("Option doesn't have AngleBracketed parameters???");
                }
            }
        }
    }

    outer_type
}


pub fn to_table_name<S: AsRef<str>>(struct_name: S) -> String {
    let re = Regex::new(r"(?P<l>[[:upper:]])").unwrap();

    re.replace_all(struct_name.as_ref(), "_$l")[1..].to_lowercase()
}
