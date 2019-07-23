extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Error,
    spanned::Spanned,
};


/// Custom derive for the `Atom` trait. Please see the trait's documentation
/// for more information on this derive.
#[proc_macro_derive(Atom)]
pub fn derive_atom(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    gen_atom_impl(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// The actual implementation for `derive(Atom)`
fn gen_atom_impl(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let out = match &input.data {
        Data::Struct(s) => {
            let mut it = s.fields.iter();

            // Get first field
            let field = it.next().ok_or_else(|| {
                let msg = "struct has no fields, but `derive(Atom)` works only for \
                    structs with exactly one field";
                Error::new(s.fields.span(), msg)
            })?;

            // Make sure there are no more fields
            if it.next().is_some() {
                let msg = "struct has more than one field, but `derive(Atom)` works only for \
                    structs with exactly one field";
                return Err(Error::new(s.fields.span(), msg));
            }

            let (field_access, struct_init) = match &field.ident {
                Some(name) => (quote! { self.#name }, quote! { Self { #name: src } }),
                None => (quote! { self.0 }, quote!{ Self(src) }),
            };

            let field_type = &field.ty;
            let type_name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            quote! {
                impl #impl_generics atomig::Atom for #type_name #ty_generics #where_clause {
                    // TODO: this line should have the span of the field once
                    // https://github.com/rust-lang/rust/issues/41817 is fixed
                    type Repr = #field_type;

                    fn pack(self) -> Self::Repr {
                        #field_access
                    }
                    fn unpack(src: Self::Repr) -> Self {
                        #struct_init
                    }
                }
            }
        }
        _ => unimplemented!(),
    };

    Ok(out)
}
