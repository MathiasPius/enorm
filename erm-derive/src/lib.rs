mod attributes;
mod queries;
mod reflect;
mod serde;

use attributes::ComponentAttributes;
use proc_macro2::{Ident, TokenStream};
use queries::{
    create_archetype_component_tables, insert_archetype, remove_archetype, select_query,
    update_archetype,
};
use quote::{quote, TokenStreamExt};
use reflect::reflect_component;
use serde::{deserialize_components, deserialize_fields, serialize_components, serialize_fields};
use syn::{Data, DeriveInput};

#[proc_macro_derive(Component, attributes(erm))]
pub fn derive_component(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let Data::Struct(data) = derive.data else {
        panic!("only structs can be stored as components");
    };

    let component_name = derive.ident;
    let table = component_name.to_string().to_lowercase();

    #[allow(unused)]
    let implementation = |database: Ident, character: char| {
        let database = quote! {::sqlx::#database};

        let columns = data.fields.iter().map(|field| {
            let name = field.ident.as_ref().unwrap().to_string();
            let typename = &field.ty;

            let attributes = ComponentAttributes::from_attributes(&field.attrs).unwrap();

            if let Some(storage_type) = attributes.store_as {
                quote! {
                    ::erm::component::ColumnDefinition::<#database> {
                        name: #name,
                        type_info: <#storage_type as ::sqlx::Type<#database>>::type_info(),
                    }
                }
            } else {
                quote! {
                    ::erm::component::ColumnDefinition::<#database> {
                        name: #name,
                        type_info: <#typename as ::sqlx::Type<#database>>::type_info(),
                    }
                }
            }
        });

        let deserialize_fields = deserialize_fields(&database, &component_name, &data.fields);
        let serialize_fields = serialize_fields(&database, &data.fields);

        let insert_component = queries::insert_component(&table, character, &data);
        let update_component = queries::update_component(&table, character, &data);
        let remove_component = queries::remove_component(&table, character);
        let create_component_table = queries::create_component_table(&database, &table, &data);

        quote! {
            impl ::erm::component::Component<#database> for #component_name {
                const INSERT: &'static str = #insert_component;
                const UPDATE: &'static str = #update_component;
                const DELETE: &'static str = #remove_component;

                fn table() -> &'static str {
                    #table
                }

                fn columns() -> Vec<::erm::component::ColumnDefinition::<#database>> {
                    vec![#(#columns,)*]
                }

                #create_component_table
                #deserialize_fields
                #serialize_fields
            }
        }
    };

    let mut implementations = TokenStream::new();
    #[cfg(feature = "sqlite")]
    implementations.append_all(implementation(
        Ident::new("Sqlite", data.struct_token.span),
        '?',
    ));

    #[cfg(feature = "postgres")]
    implementations.append_all(implementation(
        Ident::new("Postgres", data.struct_token.span),
        '$',
    ));

    #[cfg(feature = "mysql")]
    implementations.append_all(implementation(
        Ident::new("MySql", data.struct_token.span),
        '?',
    ));

    let reflection_impl = reflect_component(&component_name, &data.fields);

    implementations.append_all(reflection_impl);

    implementations.into()
}

#[proc_macro_derive(Archetype)]
pub fn derive_archetype(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let Data::Struct(data) = derive.data else {
        panic!("only structs can act as archetypes");
    };

    let archetype_name = derive.ident;

    #[allow(unused)]
    let implementation = |database: Ident| {
        let database = quote! {::sqlx::#database};

        let select_query = select_query(&database, &data.fields);

        let serialize_components = serialize_components(&database, &data.fields);

        let deserialize_components =
            deserialize_components(&archetype_name, &database, &data.fields);

        let insert_archetype = insert_archetype(&database, &data.fields);
        let update_archetype = update_archetype(&database, &data.fields);
        let remove_archetype = remove_archetype(&database, &data.fields);

        let create_archetype_component_tables =
            create_archetype_component_tables(&database, &data.fields);

        quote! {
            impl ::erm::archetype::Archetype<#database> for #archetype_name
            {
                #create_archetype_component_tables

                #insert_archetype
                #update_archetype
                #remove_archetype

                #select_query

                #deserialize_components
                #serialize_components
            }
        }
    };

    #[allow(unused_mut)]
    let mut implementations = TokenStream::new();
    #[cfg(feature = "sqlite")]
    implementations.append_all(implementation(Ident::new("Sqlite", data.struct_token.span)));

    #[cfg(feature = "postgres")]
    implementations.append_all(implementation(Ident::new(
        "Postgres",
        data.struct_token.span,
    )));

    #[cfg(feature = "mysql")]
    implementations.append_all(implementation(Ident::new("MySql", data.struct_token.span)));

    implementations.into()
}
