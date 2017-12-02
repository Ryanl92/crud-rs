use quote;
use syn;

use std::iter;

use super::{get_id_type, to_table_name};

pub fn expand_create(ast: &syn::MacroInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(ref data) => data.fields(),
        syn::Body::Enum(_) => panic!("Unable to serialize enums to the database"),
    };

    let idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).filter(|f| f.to_string() != "id").collect();

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let idents_string = idents.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    let values_string = iter::repeat("?").take(idents.len()).collect::<Vec<_>>().join(", ");
    let id_type = get_id_type(fields);

    let create_str = quote! { concat!("INSERT INTO ", #table, "(", #idents_string, ") VALUES (", #values_string, ")") };
    
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn create(mut self, conn: &::rusqlite::Connection) -> Result<#name, ::rusqlite::Error> {
                assert!(self.id.is_none(), "Cannot insert if an id is present");
                let mut stmt = conn.prepare_cached(#create_str)?;
                stmt.execute(&[#(&self.#idents as &::rusqlite::types::ToSql),*])?;

                self.id = Some(conn.last_insert_rowid() as #id_type);
                Ok(self)
            }
        }
    }
}

pub fn expand_read(ast: &syn::MacroInput) -> quote::Tokens {
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
    let id_type = get_id_type(&fields);
    
    let select_str = quote! { concat!("SELECT * FROM ", #table, " WHERE id = (?)")};

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the id of the object. This method will panic if the object has no id
            #[allow(dead_code)]
            pub fn id(&self) -> #id_type {
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
            pub fn read(conn: &::rusqlite::Connection, id: #id_type) -> Result<Option<#name>, ::rusqlite::Error> {
                let mut stmt = conn.prepare_cached(#select_str)?;
                match stmt.query_row(&[&id], #name::from_row) {
                    Ok(value) => Ok(Some(value)),
                    Err(::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(why) => Err(why),
                }
            }
        }
    }
}

pub fn expand_update(ast: &syn::MacroInput) -> quote::Tokens {
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

pub fn expand_delete(ast: &syn::MacroInput) -> quote::Tokens {
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
