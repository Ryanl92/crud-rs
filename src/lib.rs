#![recursion_limit="128"]

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate regex;
extern crate syn;

mod db;

use proc_macro::TokenStream;

use db::*;

#[proc_macro_derive(Create)]
pub fn derive_create(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).unwrap();
    expand_create(&ast).parse().unwrap()
}

#[proc_macro_derive(Read)]
pub fn derive_read(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).unwrap();
    expand_read(&ast).parse().unwrap()
}

#[proc_macro_derive(Update)]
pub fn derive_update(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).expect("Unable to parse input");

    let expanded = expand_update(&ast);
    
    expanded.parse().expect("Failed to parse expanded update macro")
}

#[proc_macro_derive(Delete)]
pub fn derive_delete(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).unwrap();
    expand_delete(&ast).parse().unwrap()
}


