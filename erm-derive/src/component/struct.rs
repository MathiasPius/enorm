use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, Data, DeriveInput};

use crate::{component::placeholders, field::Field};

use super::{ComponentAttribute, ComponentAttributeList};

#[derive(Debug)]
pub struct StructComponent {
    pub typename: Ident,
    pub table_name: String,
    pub fields: Vec<Field>,
}

impl StructComponent {
    pub fn implementation(
        &self,
        sqlx: &TokenStream,
        database: &TokenStream,
        placeholder_char: char,
    ) -> TokenStream {
        let component_name = &self.typename;

        let statements = self.statements(placeholder_char);
        let table = self.table();
        let columns = self.columns(sqlx, database);
        let table_creator = self.table_creator(sqlx, database);
        let remove = self.remove(sqlx, database);
        let insert = self.insert(sqlx, database);
        let update = self.update(database);
        let serialize = self.field_serializer(sqlx, database);
        let deserialize = self.field_deserializer(sqlx, database);

        quote! {
            impl ::erm::component::Component<#database> for #component_name {
                #statements
                #table
                #columns
                #table_creator
            }

            impl ::erm::archetype::Archetype<#database> for #component_name {}

            impl ::erm::serialization::Serializable<#database> for #component_name {
                #serialize
                #insert
                #update
            }

            impl ::erm::serialization::Deserializeable<#database> for #component_name {
                #deserialize
            }

            impl ::erm::tables::Removable<#database> for #component_name {
                #remove
            }
        }
    }

    fn statements(&self, placeholder_char: char) -> TokenStream {
        let table = &self.table_name.trim_matches('"');

        let column_names: Vec<_> = self
            .fields
            .iter()
            .map(|field| format!(", {}", field.column_name()))
            .collect();

        let placeholders = placeholders(placeholder_char, column_names.len() + 1);

        let insert = format!(
            "insert into {table}(entity{column_names}) values({placeholders});",
            placeholders = placeholders.join(", "),
            column_names = column_names.join("")
        );

        let update = {
            let field_updates = column_names
                .iter()
                .zip(placeholders.iter().skip(1))
                .map(|(column, placeholder)| format!("{column} = {placeholder}"))
                .collect::<Vec<_>>();

            format!(
                "update {table} set {field_updates} where entity = {placeholder_char}1",
                field_updates = field_updates.join(", ")
            )
        };

        let delete = format!("delete from {table} where entity = {placeholder_char}1");

        quote! {
            const INSERT: &'static str = #insert;
            const UPDATE: &'static str = #update;
            const DELETE: &'static str = #delete;
        }
    }

    fn table_creator(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let table = &self.table_name.trim_matches('"');

        let columns = self
            .fields
            .iter()
            .map(Field::column_name)
            .map(|column| format!(",\n  {column} {{}} {{}}"))
            .collect::<Vec<_>>()
            .join("");

        let format_str =
            format!("create table if not exists {table}(\n  entity {{}} primary key{columns}\n);");

        let definitions = self
            .fields
            .iter()
            .map(|field| field.sql_definition(sqlx, database));

        quote! {
            fn create_component_table<'pool, EntityId>(
                pool: &'pool #sqlx::Pool<#database>,
            ) -> impl ::core::future::Future<Output = Result<<#database as #sqlx::Database>::QueryResult, #sqlx::Error>> + Send
            where
                EntityId: #sqlx::Type<#database>,
            {
                async move {
                    use sqlx::TypeInfo as _;
                    use sqlx::Executor as _;

                    let sql = format!(
                        #format_str,
                        <EntityId as #sqlx::Type<#database>>::type_info().name(),
                        #(#definitions,)*
                    );

                    pool.execute(sql.as_str()).await
                }
            }
        }
    }

    fn table(&self) -> TokenStream {
        let table_name = &self.table_name.trim_matches('"');
        quote! {
            fn table() -> &'static str {
                #table_name
            }
        }
    }

    fn columns(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let columns = self
            .fields
            .iter()
            .map(|field| field.column_definition(sqlx, database));

        quote! {
            fn columns() -> Vec<::erm::component::ColumnDefinition::<#database>> {
                vec![#(#columns,)*]
            }
        }
    }

    fn remove(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        quote! {
            fn remove<'query, EntityId>(query: &mut ::erm::entity::EntityPrefixedQuery<'query, #database, EntityId>)
            where
                EntityId: #sqlx::Encode<'query, #database> + #sqlx::Type<#database> + Clone + 'query,
            {
                query.query(<Self as Component<#database>>::DELETE, |query| query)
            }
        }
    }

    fn insert(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        quote! {
            fn insert<'query, EntityId>(&'query self, query: &mut ::erm::entity::EntityPrefixedQuery<'query, #database, EntityId>)
            where
                EntityId: #sqlx::Encode<'query, #database> + #sqlx::Type<#database> + Clone + 'query
            {
                query.query(<Self as Component<#database>>::INSERT, move |query| {
                    <Self as Serializable<#database>>::serialize(self, query)
                })
            }
        }
    }

    fn update(&self, database: &TokenStream) -> TokenStream {
        quote! {
            fn update<'query, EntityId>(&'query self, query: &mut ::erm::entity::EntityPrefixedQuery<'query, #database, EntityId>)
            where
                EntityId: sqlx::Encode<'query, #database> + sqlx::Type<#database> + Clone + 'query
            {
                query.query(<Self as Component<#database>>::UPDATE, move |query| {
                    <Self as Serializable<#database>>::serialize(self, query)
                })
            }
        }
    }

    fn field_serializer(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let binds = self.fields.iter().map(Field::serialize);

        quote! {
            fn serialize<'q>(
                &'q self,
                query: #sqlx::query::Query<'q, #database, <#database as #sqlx::Database>::Arguments<'q>>,
            ) -> #sqlx::query::Query<'q, #database, <#database as #sqlx::Database>::Arguments<'q>> {
                #(#binds)*

                query
            }
        }
    }

    fn field_deserializer(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let component_name = &self.typename;
        let deserialized_fields = self.fields.iter().map(Field::deserialize);

        let columns = self.fields.iter().map(|field| match field {
            Field::Numbered { ident, .. } => {
                let ident = format!("column{}", ident.to_string());
                quote! {
                    #ident
                }
            }
            Field::Named { ident, .. } => {
                let ident = ident.to_string();
                quote! {
                    #ident
                }
            }
        });

        let assignments = self.fields.iter().map(|field| match field {
            Field::Numbered { ident, .. } => {
                let ident = Ident::new(&format!("self_{ident}"), ident.span());

                quote! {
                    #ident?
                }
            }
            Field::Named { ident, .. } => quote! {
                #ident: #ident?
            },
        });

        let constructor = match self
            .fields
            .first()
            .map(|field| matches!(field, Field::Named { .. }))
        {
            None => quote! { #component_name },
            Some(true) => quote! {
                #component_name {
                    #(#assignments,)*
                }
            },
            Some(false) => quote! {
                #component_name(#(#assignments,)*)
            },
        };

        let table_name = &self.table_name;

        quote! {
            fn cte() -> Box<dyn ::erm::cte::CommonTableExpression> {
                Box::new(::erm::cte::Extract {
                    table: #table_name,
                    columns: &[
                        #(#columns,)*
                    ],
                })
            }

            fn deserialize(row: &mut ::erm::row::OffsetRow<<#database as #sqlx::Database>::Row>) -> Result<Self, #sqlx::Error> {
                #(#deserialized_fields;)*

                let component = #constructor;

                Ok(component)
            }
        }
    }

    pub fn parse(derive: DeriveInput) -> syn::Result<Self> {
        let Data::Struct(data) = derive.data else {
            panic!("Component can only be derived for struct types");
        };

        let attributes: Vec<_> = Result::<Vec<Vec<_>>, syn::Error>::from_iter(
            derive
                .attrs
                .iter()
                .filter(|attr| attr.meta.path().is_ident("erm"))
                .map(|attr| {
                    let list = attr.meta.require_list()?;

                    Ok(syn::parse2::<ComponentAttributeList>(list.tokens.clone())?.0)
                }),
        )?
        .into_iter()
        .flatten()
        .collect();

        let table_name = attributes
            .iter()
            .find_map(ComponentAttribute::table)
            .unwrap_or(derive.ident.to_string());

        let type_name = derive.ident.clone();

        let fields = Result::<Vec<Field>, _>::from_iter(
            data.fields.into_iter().enumerate().map(Field::try_from),
        )?;

        Ok(StructComponent {
            typename: type_name,
            table_name,
            fields,
        })
    }
}
