use quote;
use syn;

use super::{get_id_type, to_table_name};

pub fn expand_create(ast: &syn::MacroInput) -> quote::Tokens {
    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().map(|f| f.ident.as_ref().unwrap()).filter(|f| f.to_string() != "id").collect(),
        syn::Body::Enum(_) => panic!("Unable to serialize enums to the database"),
    };

    let name = &ast.ident;
    let table = to_table_name(name.to_string());
    
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let idents = fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    let values = (1..fields.len() + 1).fold(String::new(), |acc, i| if acc.is_empty() { acc } else { acc + ", " } + &format!("${}", i));

    let create_str = quote! { concat!("INSERT INTO ", #table, "(", #idents, ") VALUES (", #values, ") RETURNING id") };
    
    let idents = &fields;
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn create(mut self, conn: &::postgres::Connection) -> ::std::result::Result<#name, ::postgres::Error> {
                assert!(self.id.is_none(), "Cannot insert if an id is present");
                let stmt = conn.prepare_cached(#create_str)?;
                let result = stmt.query(&[#(&self.#idents as &::postgres::types::ToSql),*])?;

                self.id = result.get(0).get(0);
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
    let id_type = get_id_type(fields);

    let idents1 = &idents;
    let idents2 = &idents;
    
    let select_str = quote! { concat!("SELECT * FROM ", #table, " WHERE id = $1")};

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the id of the object. This method will panic if the object has no id
            #[allow(dead_code)]
            pub fn id(&self) -> #id_type {
                self.id.expect("No id on this object")
            }
            
            #[allow(dead_code)]
            pub fn from_row(row: ::postgres::rows::Row) -> #name {
                #name {
                    #(
                        #idents1: row.get(stringify!(#idents2))
                    ),*
                }
            }
            
            #[allow(dead_code)]
            pub fn read(conn: &::postgres::Connection, id: #id_type) -> ::std::result::Result<Option<#name>, ::postgres::Error> {
                let stmt = conn.prepare_cached(#select_str)?;
                let result = stmt.query(&[&id])?.iter().map(#name::from_row).next();
                Ok(result)
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
    
    let idents = fields.iter().enumerate().fold(String::new(), |acc, (i, f)| if acc.is_empty() { acc } else { acc + ", " } + &format!("{} = ${}", f.ident.as_ref().unwrap(), i + 1));

    let id_index = fields.len() + 1;
    let update_str = quote! { concat!("UPDATE ", #table, " SET ", #idents, " WHERE id = $", #id_index)};
    
    let idents = fields.iter().map(|f| f.ident.as_ref().unwrap());
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn update(&self, conn: &::postgres::Connection) -> ::std::result::Result<(), ::postgres::Error> {
                let stmt = conn.prepare_cached(#update_str)?;

                stmt.execute(&[#(&self.#idents as &::postgres::types::ToSql),*, &self.id as &::postgres::types::ToSql])?;
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
    let delete_str = quote! { concat!("DELETE FROM ", #table, " WHERE id = $1") };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            pub fn delete(&self, conn: &::postgres::Connection) -> ::std::result::Result<(), ::postgres::Error> {
                let mut stmt = conn.prepare_cached(#delete_str)?;
                stmt.execute(&[&self.id])?;
                Ok(())
            }
        }
    }
}

