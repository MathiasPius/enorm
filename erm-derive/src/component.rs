use proc_macro2::{Ident, Literal, Punct, TokenStream};
use quote::{quote, TokenStreamExt as _};
use r#enum::EnumComponent;
use syn::{parse::Parse, DeriveInput, Token};

mod r#enum;
mod r#struct;
pub use r#struct::*;

use crate::{implement_for, reflect::reflect_component};

#[derive(Debug)]
pub enum Component {
    Struct(StructComponent),
    Enum(EnumComponent),
}

impl Component {
    pub fn typename(&self) -> &Ident {
        match self {
            Component::Struct(struct_component) => &struct_component.typename,
            Component::Enum(enum_component) => &enum_component.typename,
        }
    }

    pub fn table_name(&self) -> &str {
        match self {
            Component::Struct(struct_component) => &struct_component.table_name,
            Component::Enum(enum_component) => &enum_component.table_name,
        }
    }

    pub fn derive(&self) -> TokenStream {
        let implementation = |database: Ident, placeholder_char: char| {
            #[cfg(feature = "bundled")]
            let sqlx = quote! {::erm::sqlx};
            #[cfg(not(feature = "bundled"))]
            let sqlx = quote! {::sqlx};

            let database = quote! {#sqlx::#database};

            match self {
                Component::Struct(struct_component) => {
                    struct_component.implementation(&sqlx, &database, placeholder_char)
                }
                Component::Enum(enum_component) => {
                    enum_component.implementation(&sqlx, &database, placeholder_char)
                }
            }
        };

        let mut implementations = implement_for(implementation);

        match self {
            Component::Struct(struct_component) => {
                implementations.append_all(reflect_component(&struct_component));
            }
            Component::Enum(_) => (),
        }
        implementations.into()
    }
}

impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive = DeriveInput::parse(input)?;

        match &derive.data {
            syn::Data::Struct(_) => Ok(Component::Struct(StructComponent::parse(derive)?)),
            syn::Data::Enum(_) => Ok(Component::Enum(EnumComponent::parse(derive)?)),
            syn::Data::Union(_) => {
                panic!("Component can only be implemented for structs or enums")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentAttributeList(pub Vec<ComponentAttribute>);

impl Parse for ComponentAttributeList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = Vec::new();

        while !input.is_empty() {
            attributes.push(ComponentAttribute::parse(input)?);

            if input.peek(Token![,]) {
                input.parse::<Punct>()?;
            }
        }

        Ok(Self(attributes))
    }
}

#[derive(Debug, Clone)]
pub enum ComponentAttribute {
    /// Changes the name of the Component's sql table.
    Table { name: Literal },
}

impl ComponentAttribute {
    pub fn table(&self) -> Option<String> {
        #[allow(irrefutable_let_patterns)]
        if let ComponentAttribute::Table { name } = self {
            Some(name.to_string())
        } else {
            None
        }
    }
}

impl Parse for ComponentAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        Ok(match ident.to_string().as_str() {
            "table" => {
                input.parse::<Token![=]>()?;

                ComponentAttribute::Table {
                    name: input.parse()?,
                }
            }
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "unexpected Component attribute",
                ))
            }
        })
    }
}

/// Generates placeholder values corresponding to the number of columns.
pub fn placeholders(character: char, count: usize) -> Vec<String> {
    std::iter::repeat(character)
        .enumerate()
        .skip(1)
        .take(count)
        .map(|(i, character)| format!("{character}{i}"))
        .collect::<Vec<_>>()
}
