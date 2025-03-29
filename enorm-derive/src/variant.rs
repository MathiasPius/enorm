use proc_macro2::Ident;

use crate::field::Field;

#[derive(Debug)]
pub struct Variant {
    pub ident: Ident,
    pub fields: Vec<Field>,
}

impl TryFrom<syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(variant: syn::Variant) -> Result<Self, Self::Error> {
        let ident = variant.ident;

        if variant.discriminant.is_some() {
            unimplemented!("can't derive Archetype for enums with discrimants.");
        }

        let fields = Result::<Vec<Field>, _>::from_iter(
            variant.fields.into_iter().enumerate().map(Field::try_from),
        )?;

        Ok(Variant { ident, fields })
    }
}
