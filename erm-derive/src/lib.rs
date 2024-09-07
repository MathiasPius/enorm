use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{Data, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let Data::Struct(data) = derive.data else {
        panic!("only structs can be stored as components");
    };

    let component_name = derive.ident;
    let table = component_name.to_string().to_lowercase();

    let columns: Vec<_> = data
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect();

    let unpack: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let typename = &field.ty;

            quote! {
                let #name = row.try_get::<#typename>();
            }
        })
        .collect();

    let repack: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();

            quote! {
                #name: #name?
            }
        })
        .collect();

    let binds: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();

            quote! {
                .bind(&self.#name)
            }
        })
        .collect();

    let implementation = |database: Ident| {
        quote! {
            impl ::erm::Component<::sqlx::#database> for #component_name {
                fn table() -> &'static str {
                    #table
                }

                fn columns() -> &'static [&'static str] {
                    &[#(#columns,)*]
                }

                fn deserialize_fields(row: &mut ::erm::OffsetRow<<::sqlx::#database as ::sqlx::Database>::Row>) -> Result<Self, ::sqlx::Error> {
                    #(#unpack;)*

                    Ok(#component_name {
                        #(#repack,)*
                    })
                }

                fn serialize_fields<'q>(
                    &'q self,
                    query: ::sqlx::query::Query<'q, ::sqlx::#database, <::sqlx::#database as ::sqlx::Database>::Arguments<'q>>,
                ) -> ::sqlx::query::Query<'q, ::sqlx::#database, <::sqlx::#database as ::sqlx::Database>::Arguments<'q>> {
                    query #(#binds)*
                }

            }
        }
    };

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

#[proc_macro_derive(Archetype)]
pub fn derive_archetype(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let Data::Struct(data) = derive.data else {
        panic!("only structs can act as archetypes");
    };

    let archetype_name = derive.ident;

    let implementation = |database: Ident| {
        let insert_statements = data.fields.iter().map(|field| {
            let typename = &field.ty;

            quote! {
                <#typename as ::erm::Archetype<::sqlx::#database>>::insert_statement()
            }
        });

        let field_names = data
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap());

        let mut field_iter = data.fields.iter();

        let first_item = &field_iter.next().unwrap().ty;
        let first = quote! {
            let join = <#first_item as Archetype<::sqlx::#database>>::select_statement();
        };

        let select_statements = field_iter.map(|field| {
            let field = &field.ty;

            quote! {
                let join = ::erm::cte::InnerJoin {
                    left: (
                        Box::new(join),
                        "entity".to_string(),
                    ),
                    right: (
                        Box::new(<#field as Archetype<::sqlx::#database>>::select_statement()),
                        "entity".to_string(),
                    ),
                }
            }
        });

        let unpack: Vec<_> = data
            .fields
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                let typename = &field.ty;

                quote! {
                    let #name = <#typename as ::erm::Archetype<::sqlx::#database>>::deserialize_components(row);
                }
            })
            .collect();

        let repack: Vec<_> = data
            .fields
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();

                quote! {
                    #name: #name?
                }
            })
            .collect();

        quote! {
            impl Archetype<::sqlx::#database> for #archetype_name
            {
                fn insert_statement() -> String {
                    vec![
                        #(#insert_statements,)*
                    ]
                    .join(";\n")
                }

                fn serialize_components<'q>(
                    &'q self,
                    query: ::sqlx::query::Query<'q, ::sqlx::#database, <::sqlx::#database as ::sqlx::Database>::Arguments<'q>>,
                ) -> ::sqlx::query::Query<'q, ::sqlx::#database, <::sqlx::#database as ::sqlx::Database>::Arguments<'q>> {
                    #(
                        let query = self.#field_names.serialize_components(query);
                    )*

                    query
                }

                fn select_statement() -> impl ::erm::cte::CommonTableExpression {
                    #first;
                    #(#select_statements;)*

                    join
                }

                fn deserialize_components(
                    row: &mut ::erm::OffsetRow<<::sqlx::#database as ::sqlx::Database>::Row>,
                ) -> Result<Self, ::sqlx::Error> {
                    #(#unpack;)*

                    Ok(#archetype_name {
                        #(#repack,)*
                    })
                }
            }
        }
    };

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
