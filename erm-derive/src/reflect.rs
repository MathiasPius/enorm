use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{component::Component, field::Field};

pub fn reflect_component(component: &Component) -> TokenStream {
    let component_name = component.typename();

    let reflection_name = Ident::new(&format!("Reflected{component_name}"), component_name.span());

    let declarations = component.fields().iter().map(Field::reflected_column);

    let constructors = component.fields().iter().map(|field| {
        let name = field.field_name();
        let stringified = format!(
            "__cte_{table_name}__{column}",
            table_name = component.table_name(),
            column = field.column_name()
        );

        quote! {
            #[allow(unused)]
            #name: ::erm::reflect::ReflectedColumn::new(#stringified)
        }
    });

    quote! {
        #[allow(unused)]
        pub struct #reflection_name {
            #(#declarations),*
        }

        impl #reflection_name {
            #[allow(unused)]
            pub const fn new() -> Self {
                Self {
                    #(#constructors,)*
                }
            }
        }

        impl ::erm::reflect::Reflect for #component_name {
            type ReflectionType = #reflection_name;
            const FIELDS: Self::ReflectionType = #reflection_name::new();
        }
    }
}
