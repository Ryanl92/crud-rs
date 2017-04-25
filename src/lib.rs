extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate regex;
extern crate syn;

use proc_macro::TokenStream;
use regex::Regex;

use std::iter;

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

fn to_table_name<S: AsRef<str>>(struct_name: S) -> String {
    let re = Regex::new(r"(?P<l>[[:upper:]])").unwrap();

    re.replace_all(struct_name.as_ref(), "_$l")[1..].to_lowercase()
}

fn expand_create(ast: &syn::MacroInput) -> quote::Tokens {
    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().map(|f| f.ident.as_ref().unwrap()).filter(|f| f.to_string() != "id").collect(),
        syn::Body::Enum(_) => panic!("Unable to serialize enums to the database"),
    };

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let idents = fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    let values = iter::repeat("?").take(fields.len()).collect::<Vec<_>>().join(", ");

    let create_str = quote! { concat!("INSERT INTO ", #table, "(", #idents, ") VALUES (", #values, ")") };
    
    let idents = &fields;
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn create(mut self, conn: &::rusqlite::Connection) -> Result<#name, ::rusqlite::Error> {
                assert!(self.id.is_none(), "Cannot insert if an id is present");
                let mut stmt = conn.prepare_cached(#create_str)?;
                stmt.execute(&[#(&self.#idents as &::rusqlite::types::ToSql),*])?;

                self.id = Some(conn.last_insert_rowid());
                Ok(self)
            }
        }
    }
}

fn expand_read(ast: &syn::MacroInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(ref data) => data.fields(),
        syn::Body::Enum(_) => panic!("Unable to serialize enums to the database"),
    };

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let idents = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let idents1 = &idents;
    let idents2 = &idents;
    
    let select_str = quote! { concat!("SELECT * FROM ", #table, " WHERE id = (?)")};

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the id of the object. This method will panic if the object has no id
            #[allow(dead_code)]
            pub fn id(&self) -> i64 {
                self.id.expect("No id on this object")
            }
            
            #[allow(dead_code)]
            pub fn from_row(row: &::rusqlite::Row) -> #name {
                #name {
                    #(
                        #idents1: row.get(stringify!(#idents2))
                    ),*
                }
            }
            
            #[allow(dead_code)]
            pub fn read(conn: &::rusqlite::Connection, id: i64) -> Result<#name, ::rusqlite::Error> {
                let mut stmt = conn.prepare_cached(#select_str)?;
                stmt.query_row(&[&id], #name::from_row)
            }
        }
    }
}

fn expand_update(ast: &syn::MacroInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(ref data) => data.fields(),
        syn::Body::Enum(_) => panic!("Unable to serialize enums to the database"),
    };

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    
    let idents = fields.iter().fold(String::new(), |acc, f| if acc.is_empty() { acc } else { acc + ", " } + &format!("{} = (?)", f.ident.as_ref().unwrap()));

    let update_str = quote! { concat!("UPDATE ", #table, " SET ", #idents, " WHERE id = (?)")};
    
    let idents = fields.iter().map(|f| f.ident.as_ref().unwrap());
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn update(&self, conn: &::rusqlite::Connection) -> Result<(), ::rusqlite::Error> {
                let mut stmt = conn.prepare_cached(#update_str)?;

                stmt.execute(&[#(&self.#idents as &::rusqlite::types::ToSql),*, &self.id as &::rusqlite::types::ToSql])?;
                Ok(())
            }
        }
    }
}

fn expand_delete(ast: &syn::MacroInput) -> quote::Tokens {
    if let syn::Body::Enum(_) = ast.body {
        panic!("Unable to serialize enums to the database");
    }

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let delete_str = quote! { concat!("DELETE FROM ", #table, " WHERE id = (?)") };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn delete(&self, conn: &::rusqlite::Connection) -> Result<(), ::rusqlite::Error> {
                let mut stmt = conn.prepare_cached(#delete_str)?;
                stmt.execute(&[&self.id])?;
                Ok(())
            }
        }
    }
}
