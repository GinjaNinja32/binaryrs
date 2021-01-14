#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Fields, Generics, Lit, Meta, NestedMeta, Path, Type,
    WherePredicate,
};

fn parse_attrs(
    input: Vec<Attribute>,
) -> (
    Vec<syn::export::TokenStream2>,
    Vec<syn::export::TokenStream2>,
) {
    let mut attrs = vec![];
    let mut errors = vec![];

    for attr in input {
        let data = match attr.parse_meta() {
            Ok(v) => v,
            Err(e) => {
                errors.push(e.to_compile_error());
                continue;
            }
        };
        match data {
            Meta::Word(_) => {
                let span = data.span();
                errors.push(quote_spanned! {span=>
                    compile_error!("illegal attribute form");
                });
                continue;
            }
            Meta::List(l) => {
                for elem in l.nested {
                    match elem {
                        NestedMeta::Meta(m) => match &m {
                            Meta::Word(w) => {
                                let span = w.span();
                                match w.to_string().as_str() {
                                    "little" => attrs.push(quote_spanned! {span=>
                                        attrs.endian = ::binary::attr::Endian::Little;
                                    }),
                                    "big" => attrs.push(quote_spanned! {span=>
                                        attrs.endian = ::binary::attr::Endian::Big;
                                    }),
                                    "len_little" => attrs.push(quote_spanned! {span=>
                                        attrs.len_endian = ::binary::attr::Endian::Little;
                                    }),
                                    "len_big" => attrs.push(quote_spanned! {span=>
                                        attrs.len_endian = ::binary::attr::Endian::Big;
                                    }),
                                    "reset" => attrs.push(quote_spanned! {span=>
                                        attrs = ::binary::attr::Attrs::zero();
                                    }),
                                    _ => {
                                        errors.push(quote_spanned! {span=>
                                            compile_error!("unknown attribute");
                                        });
                                    }
                                }
                            }
                            Meta::List(_) => {
                                let span = m.span();
                                errors.push(quote_spanned! {span=>
                                    compile_error!("illegal attribute form");
                                });
                            }
                            Meta::NameValue(nv) => match nv.ident.to_string().as_str() {
                                "len" => match &nv.lit {
                                    Lit::Int(i) => {
                                        let span = nv.span();
                                        match i.value() {
                                            0 => attrs.push(quote_spanned! {span=>
                                                attrs.len = ::binary::attr::Len::None;
                                            }),
                                            1 => attrs.push(quote_spanned! {span=>
                                                attrs.len = ::binary::attr::Len::U8;
                                            }),
                                            2 => attrs.push(quote_spanned! {span=>
                                                attrs.len = ::binary::attr::Len::U16;
                                            }),
                                            4 => attrs.push(quote_spanned! {span=>
                                                attrs.len = ::binary::attr::Len::U32;
                                            }),
                                            8 => attrs.push(quote_spanned! {span=>
                                                attrs.len = ::binary::attr::Len::U64;
                                            }),
                                            _ => {
                                                let span = nv.lit.span();
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal argument to 'len'");
                                                });
                                            }
                                        }
                                    }
                                    _ => {
                                        let span = nv.lit.span();
                                        errors.push(quote_spanned! {span=>
                                            compile_error!("illegal argument to 'len'");
                                        });
                                    }
                                },
                                _ => {
                                    let span = m.span();
                                    errors.push(quote_spanned! {span=>
                                        compile_error!("unknown attribute");
                                    });
                                }
                            },
                        },
                        NestedMeta::Literal(_) => {
                            let span = elem.span();
                            errors.push(quote_spanned! {span=>
                                compile_error!("illegal attribute form");
                            });
                        }
                    }
                }
            }
            Meta::NameValue(_) => {
                let span = data.span();
                errors.push(quote_spanned! {span=>
                    compile_error!("illegal attribute form");
                });
                continue;
            }
        }
    }

    (attrs, errors)
}

/*
input
    attrs
        attributes on the whole struct/enum
    vis
        visibility (public, crate, restricted, inherited)
    ident
        name of struct/enum
    generics
        any generics that are involved
    data
        Struct
            struct_token
            fields
            semi_token
        Enum
            enum_token
            brace_token
            variants
        Union
            union_token
            fields
*/

#[proc_macro_derive(BinSerialize, attributes(binary))]
pub fn derive_binserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (attrs, attr_errors) = parse_attrs(input.attrs);
    let ident = input.ident;
    let (generics, fields) = encode_fields(input.generics, input.data, &attrs);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinSerialize for #ident#ty_generics #where_clause {
            fn encode_to(&self, buf: &mut dyn ::binary::BufMut, attrs: ::binary::attr::Attrs) -> ::binary::Result<()> {
                #(#fields)*
                Ok(())
            }
        }
        #(#attr_errors)*
    };
    s.into()
}

#[proc_macro_derive(BinDeserialize, attributes(binary))]
pub fn derive_bindeserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (attrs, attr_errors) = parse_attrs(input.attrs);
    let ident = input.ident;
    let (generics, fields) = decode_fields(input.generics, input.data, &attrs);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinDeserialize for #ident#ty_generics #where_clause {
            fn decode_from(buf: &mut dyn ::binary::Buf, attrs: ::binary::attr::Attrs) -> ::binary::Result<Self> {
                Ok(Self {
                    #(#fields)*
                })
            }
        }
        #(#attr_errors)*
    };
    s.into()
}

fn make_generic_bound(ty: Type, bound: Path) -> WherePredicate {
    syn::WherePredicate::Type(syn::PredicateType {
        lifetimes: None,
        bounded_ty: ty,
        colon_token: <Token![:]>::default(),
        bounds: vec![syn::TypeParamBound::Trait(syn::TraitBound {
            paren_token: None,
            modifier: syn::TraitBoundModifier::None,
            lifetimes: None,
            path: bound, // parse_quote! {::binary::BinSerialize},
        })]
        .into_iter()
        .collect(),
    })
}

fn encode_fields(
    mut generics: Generics,
    data: Data,
    struct_attrs: &[syn::export::TokenStream2],
) -> (Generics, Vec<syn::export::TokenStream2>) {
    let mut encodes: Vec<syn::export::TokenStream2> = vec![];
    match data {
        Data::Struct(s) => match s.fields {
            Fields::Named(n) => {
                for f in n.named {
                    let ident = f.ident.unwrap();
                    let (attrs, attr_errors) = parse_attrs(f.attrs);

                    generics
                        .make_where_clause()
                        .predicates
                        .push(make_generic_bound(
                            f.ty,
                            parse_quote! {::binary::BinSerialize},
                        ));
                    encodes.push(quote! {
                        ::binary::BinSerialize::encode_to(&self.#ident, buf, {
                            let mut attrs = attrs.clone();
                            #(#struct_attrs)*
                            #(#attrs)*
                            attrs
                        })?;
                        #(#attr_errors)*
                    });
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    (generics, encodes)
}

fn decode_fields(
    mut generics: Generics,
    data: Data,
    struct_attrs: &[syn::export::TokenStream2],
) -> (Generics, Vec<syn::export::TokenStream2>) {
    let mut decodes: Vec<syn::export::TokenStream2> = vec![];
    match data {
        Data::Struct(s) => match s.fields {
            Fields::Named(n) => {
                for f in n.named {
                    let ident = f.ident.unwrap();
                    let ty = f.ty.clone();
                    let (attrs, attr_errors) = parse_attrs(f.attrs);

                    generics
                        .make_where_clause()
                        .predicates
                        .push(make_generic_bound(
                            f.ty,
                            parse_quote! {::binary::BinDeserialize},
                        ));
                    decodes.push(quote! {
                        #ident: <#ty as ::binary::BinDeserialize>::decode_from(buf, {
                            let mut attrs = attrs.clone();
                            #(#struct_attrs)*
                            #(#attrs)*
                            attrs
                        })?,
                        #(#attr_errors)*
                    });
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    (generics, decodes)
}
