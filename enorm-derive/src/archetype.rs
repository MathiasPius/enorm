mod r#enum;
mod r#struct;

use proc_macro2::TokenStream;
use quote::quote;
use r#enum::EnumArchetype;
use r#struct::StructArchetype;
use syn::{parse::Parse, Data, DeriveInput};

use crate::{field::Field, variant::Variant};

pub enum Archetype {
    Struct(StructArchetype),
    Enum(EnumArchetype),
}

impl Archetype {
    pub fn implementation(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        match self {
            Archetype::Struct(struct_archetype) => struct_archetype.implementation(sqlx, database),
            Archetype::Enum(enum_archetype) => enum_archetype.implementation(sqlx, database),
        }
    }

    fn remove(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        match self {
            Archetype::Struct(struct_archetype) => struct_archetype.remove(sqlx, database),
            Archetype::Enum(_) => {
                quote! { compiler_error("Can't delete enum Archetypes")}
            }
        }
    }

    fn component_deserializer(&self, sqlx: &TokenStream, database: &TokenStream) -> TokenStream {
        match self {
            Archetype::Struct(struct_archetype) => {
                struct_archetype.component_deserializer(sqlx, database)
            }
            Archetype::Enum(enum_archetype) => {
                enum_archetype.component_deserializer(sqlx, database)
            }
        }
    }
}

impl Parse for Archetype {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive = DeriveInput::parse(input)?;

        let typename = derive.ident.clone();

        match derive.data {
            Data::Struct(data) => {
                let fields = Result::<Vec<Field>, _>::from_iter(
                    data.fields.into_iter().enumerate().map(Field::try_from),
                )?;

                Ok(Archetype::Struct(StructArchetype { typename, fields }))
            }
            Data::Enum(data) => {
                let variants = Result::<Vec<Variant>, _>::from_iter(
                    data.variants.into_iter().map(Variant::try_from),
                )?;

                Ok(Archetype::Enum(EnumArchetype { typename, variants }))
            }
            Data::Union(_) => {
                panic!("Archetype can only be implemented for structs or enums")
            }
        }
    }
}
