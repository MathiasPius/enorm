use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::field::Field;

#[derive(Debug)]
pub struct StructArchetype {
    pub typename: Ident,
    pub fields: Vec<Field>,
}

impl StructArchetype {
    pub fn implementation(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let archetype_name = &self.typename;

        let remove = self.remove(sqlx, database);

        let deserializer = self.component_deserializer(sqlx, database);

        quote! {
            impl ::enorm::archetype::Archetype<#database> for #archetype_name
            {
            }

            impl ::enorm::serialization::Deserializeable<#database> for #archetype_name {
                #deserializer
            }

            impl ::enorm::tables::Removable<#database> for #archetype_name {
                #remove
            }
        }
    }

    pub fn remove(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let sub_archetypes = self.fields.iter().map(|field| {
            let typename = field.typename();

            quote! {
                <#typename as ::enorm::tables::Removable<#database>>::remove(query);
            }
        });

        quote! {
            fn remove<'query, EntityId>(query: &mut ::enorm::entity::EntityPrefixedQuery<'query, #database, EntityId>)
            where
                EntityId: #sqlx::Encode<'query, #database> + #sqlx::Type<#database> + Clone + 'query
            {
                #(#sub_archetypes)*
            }
        }
    }

    pub fn component_deserializer(
        &self,
        sqlx: &TokenStream,
        database: &TokenStream,
    ) -> TokenStream {
        let archetype_name = &self.typename;

        let sub_expressions = self.fields.iter().map(|field| {
            let typename = field.typename();

            quote! {
                <#typename as ::enorm::serialization::Deserializeable<#database>>::cte()
            }
        });

        let components = self.fields.iter().map(|field| {
            let name = field.ident();
            let typename = field.typename();

            quote! {
                let #name = <#typename as ::enorm::serialization::Deserializeable<#database>>::deserialize(row);
            }
        });

        let assignments = self.fields.iter().map(|field| {
            let ident = field.ident();

            quote! {
                #ident: #ident?
            }
        });

        quote! {
            fn cte() -> Box<dyn ::enorm::cte::CommonTableExpression> {
                Box::new(::enorm::cte::Merge {
                    tables: vec![
                        #(#sub_expressions,)*
                    ]
                })
            }

            fn deserialize(row: &mut ::enorm::row::OffsetRow<<#database as #sqlx::Database>::Row>) -> Result<Self, #sqlx::Error> {
                #(#components)*

                let archetype = #archetype_name {
                    #(#assignments,)*
                };

                Ok(archetype)
            }
        }
    }
}
