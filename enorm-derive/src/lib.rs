mod archetype;
mod component;
mod field;
mod reflect;

use archetype::Archetype;
use component::Component;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};

#[proc_macro_derive(Component, attributes(erm))]
pub fn derive_component(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let component: Component = syn::parse2(stream).unwrap();

    component.derive().into()
}

#[proc_macro_derive(Archetype, attributes(erm))]
pub fn derive_archetype(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let archetype: Archetype = syn::parse2(stream).unwrap();

    let implementation = |database: Ident, _: char| {
        #[cfg(feature = "bundled")]
        let sqlx = quote! {::enorm::sqlx};
        #[cfg(not(feature = "bundled"))]
        let sqlx = quote! {::sqlx};
        let database = quote! {#sqlx::#database};

        archetype.implementation(&sqlx, &database)
    };

    implement_for(implementation).into()
}

#[allow(unused)]
fn implement_for(implementer: impl Fn(Ident, char) -> TokenStream) -> TokenStream {
    #[allow(unused_mut)]
    let mut implementations = TokenStream::new();

    let span = Span::call_site();

    #[cfg(feature = "sqlite")]
    implementations.append_all(implementer(Ident::new("Sqlite", span), '?'));

    #[cfg(feature = "postgres")]
    implementations.append_all(implementer(Ident::new("Postgres", span), '$'));

    #[cfg(feature = "mysql")]
    implementations.append_all(implementer(Ident::new("MySql", span), '?'));

    implementations
}
