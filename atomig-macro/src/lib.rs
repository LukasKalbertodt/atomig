extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Error, Fields,
    Meta, NestedMeta,
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

/// Custom derive for the `AtomLogic` trait. Please see the trait's
/// documentation for more information on this derive.
#[proc_macro_derive(AtomLogic)]
pub fn derive_atom_logic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    gen_marker_trait_impl("AtomLogic", &input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Custom derive for the `AtomInteger` trait. Please see the trait's
/// documentation for more information on this derive.
#[proc_macro_derive(AtomInteger)]
pub fn derive_atom_integer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    gen_marker_trait_impl("AtomInteger", &input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn gen_marker_trait_impl(trait_name: &str, input: &DeriveInput) -> Result<TokenStream2, Error> {
    match input.data {
        Data::Struct(_) => {
            let type_name = &input.ident;
            let trait_name = Ident::new(trait_name, Span::call_site());
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            Ok(quote! {
                impl #impl_generics atomig::#trait_name
                    for #type_name #ty_generics #where_clause {}
            })
        }
        Data::Enum(_) => {
            let msg = format!(
                "`{}` cannot be derived for enums as this is almost always incorrect to do. \
                    Please read the documentation of `{}` carefully. If you still think you \
                    want to implement this trait, you have to do it manually.",
                trait_name,
                trait_name,
            );
            Err(Error::new(Span::call_site(), msg))
        }
        Data::Union(_) => {
            let msg = format!("`{}` cannot be derived for unions", trait_name);
            Err(Error::new(Span::call_site(), msg))
        }
    }
}

/// The actual implementation for `derive(Atom)`.
fn gen_atom_impl(input: &DeriveInput) -> Result<TokenStream2, Error> {
    // Generate the body of the impl block.
    let impl_body = match &input.data {
        Data::Struct(s) => atom_impl_for_struct(s),
        Data::Enum(e) => atom_impl_for_enum(input, e),
        Data::Union(_) => Err(Error::new(Span::call_site(), "unions cannot derive `Atom`")),
    }?;

    // Combine everything into a finshed impl block.
    let type_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics atomig::Atom for #type_name #ty_generics #where_clause {
            #impl_body
        }
    })
}

/// Generates the body of the `impl Atom` block for the given struct definition.
fn atom_impl_for_struct(s: &DataStruct) -> Result<TokenStream2, Error> {
    let mut it = s.fields.iter();

    // Get first field
    let field = it.next().ok_or_else(|| {
        let msg = "struct has no fields, but `derive(Atom)` works only for \
            structs with exactly one field";
        Error::new(s.fields.span(), msg)
    })?;

    // Make sure there are no other fields
    if it.next().is_some() {
        let msg = "struct has more than one field, but `derive(Atom)` works only for \
            structs with exactly one field";
        return Err(Error::new(s.fields.span(), msg));
    }

    // Generate the code for `pack` and `unpack` which depends on weather it is
    // a named or tuple-struct field.
    let (field_access, struct_init) = match &field.ident {
        Some(name) => (quote! { self.#name }, quote! { Self { #name: src } }),
        None => (quote! { self.0 }, quote!{ Self(src) }),
    };

    let field_type = &field.ty;
    Ok(quote! {
        // TODO: this line should have the span of the field once
        // https://github.com/rust-lang/rust/issues/41817 is fixed
        type Repr = #field_type;

        fn pack(self) -> Self::Repr {
            #field_access
        }
        fn unpack(src: Self::Repr) -> Self {
            #struct_init
        }
    })
}

/// Generates the body of the `impl Atom` block for the given enum definition.
fn atom_impl_for_enum(input: &DeriveInput, e: &DataEnum) -> Result<TokenStream2, Error> {
    // Make sure we have a `repr` attribute on the enum.
    let repr_attr = input.attrs.iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .find(|meta| meta.name() == "repr")
        .ok_or_else(|| {
            let msg = format!(
                "no `repr(_)` attribute on enum '{}', but such an attribute is \
                    required to automatically derive `Atom`",
                input.ident,
            );
            Error::new(Span::call_site(), msg)
        })?;

    // Make sure the `repr` attribute has the correct syntax and actually
    // specifies the primitive representation.
    const INTEGER_NAMES: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize",
        "i8", "i16", "i32", "i64", "i128", "isize",
    ];
    let repr_type = match &repr_attr {
        Meta::List(list) => {
            list.nested.iter()
                .find_map(|nested| {
                    match &nested {
                        NestedMeta::Meta(Meta::Word(w))
                            if INTEGER_NAMES.contains(&&*w.to_string()) => Some(w),
                        _ => None
                    }
                })
                .ok_or_else(|| {
                    let msg = "`repr(_)` attribute does not specify the primitive \
                        representation (a primitive integer), but this is required \
                        for `derive(Atom)`";
                    Error::new(repr_attr.span(), msg)
                })?
        }
        _ => {
            let msg = format!(
                "`repr` attribute on enum '{}' does not have the form `repr(_)`, but \
                    it should have for `derive(Atom)`",
                input.ident,
            );
            return Err(Error::new(repr_attr.span(), msg));
        }
    };

    // Check that all variants have no fields. In other words: that the enum is
    // C-like.
    let variant_with_fields = e.variants.iter().find(|variant| {
        match variant.fields {
            Fields::Unit => false,
            _ => true,
        }
    });
    if let Some(v) = variant_with_fields  {
        let msg = "this variant has fields, but `derive(Atom)` only works \
            for C-like enums";
        return Err(Error::new(v.span(), msg));
    }

    // Generate the code for `unpack` which is more complicated than the `pack`
    // code. For `pack` we can simply use the `as` cast, but for unpack we have
    // to assemble a list of `if` statements. If you would hand code such a
    // method, you would use a `match` statement. But we use 'ifs' so that we
    // don't have to check for the discriminant values ourselves. That might be
    // very hard.
    let type_name = &input.ident;
    let unpack_code = {
        let checks: Vec<_> = e.variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            quote! {
                if src == #type_name::#variant_name as #repr_type {
                    return #type_name::#variant_name;
                }
            }
        }).collect();

        let error = format!(
            "invalid '{}' value '{{}}' for enum '{}' in `Atom::unpack`",
            repr_type,
            type_name,
        );
        quote! {
            #(#checks)*
            panic!(#error, src);
        }
    };


    Ok(quote! {
        type Repr = #repr_type;

        fn pack(self) -> Self::Repr {
            self as #repr_type
        }
        fn unpack(src: Self::Repr) -> Self {
            #unpack_code
        }
    })
}
