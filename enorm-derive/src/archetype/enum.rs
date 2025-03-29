use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{field::Field, variant::Variant};

#[derive(Debug)]
pub struct EnumArchetype {
    pub typename: Ident,
    pub variants: Vec<Variant>,
}

impl EnumArchetype {
    pub fn implementation(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        let archetype_name = &self.typename;

        let deserializer = self.component_deserializer(sqlx, database);

        quote! {
            impl ::enorm::archetype::Archetype<#database> for #archetype_name
            {
            }

            impl ::enorm::serialization::Deserializeable<#database> for #archetype_name {
                #deserializer
            }
        }
    }

    pub fn component_deserializer(
        &self,
        sqlx: &TokenStream,
        database: &TokenStream,
    ) -> TokenStream {
        let archetype_name = &self.typename;

        let all_fields: HashSet<_> = self
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter())
            .collect();

        let sub_expressions = all_fields.iter().map(|field| {
            let typename = field.typename();

            quote! {
                <Option<#typename> as ::enorm::serialization::Deserializeable<#database>>::cte()
            }
        });

        let components = all_fields.iter().map(|field| {
            let name = field.field_name();
            let typename = field.typename();

            quote! {
                let #name = <Option<#typename> as ::enorm::serialization::Deserializeable<#database>>::deserialize(row)?;
            }
        });

        let component_names = all_fields.iter().map(|field| {
            let name = field.field_name();
            quote! {
                #name
            }
        });

        let variants = self.variants.iter().map(|variant| {
            let variant_name = &variant.ident;

            let pattern = all_fields.iter().map(|available_field| {
                if let Some(field) = variant
                    .fields
                    .iter()
                    .find(|field| field.column_name() == available_field.column_name())
                {
                    let ident = field.ident();

                    quote! { Some(#ident), }
                } else {
                    quote! { None, }
                }
            });

            let assignments = variant.fields.iter().map(Field::ident).collect::<Vec<_>>();

            if assignments.is_empty() {
                quote! {
                    ( #(#pattern),* ) => {
                        Ok(#archetype_name::#variant_name)
                    }
                }
            } else {
                quote! {
                    ( #(#pattern)* ) => {
                        Ok(#archetype_name::#variant_name {
                            #(#assignments,)*
                        })
                    }
                }
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
                #(#components)*;

                match (#(#component_names,)*) {
                    #(#variants)*,
                    _ => unimplemented!(),
                }
            }

            // fn deserialize(row: &mut ::enorm::row::OffsetRow<<#database as #sqlx::Database>::Row>) -> Result<Self, #sqlx::Error> {
            //     #(#components)*

            //     let archetype = #archetype_name {
            //         #(#assignments,)*
            //     };

            //     Ok(archetype)
            // }
        }
    }
}
