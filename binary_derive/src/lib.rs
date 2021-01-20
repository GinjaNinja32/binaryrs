#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Fields, Generics, Ident, Index, IntSuffix, LitInt, Member, Path, Type,
    WherePredicate,
};

mod context;
use context::{Context, Environment, Level};
mod helpers;

struct SelfAttrs {
    tag_ty: Option<(Type, IntSuffix)>, // enum, based on repr()
    tag_le: Option<bool>,              // enum
    tag_default: bool,                 // enum field, which must be of type (tag_ty, Vec<u8>)

    nest_variants: bool,                // enum
    nest: bool,                         // enum variant, in enum with nest_variants true
    nest_ty: Option<helpers::SizeType>, // enum
    nest_le: Option<bool>,              // enum

    flags: bool,             // field
    flag_value: Option<u64>, // field
}

#[proc_macro_derive(BinSerialize, attributes(binary))]
pub fn derive_binserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    #[cfg(feature = "debug_prints")]
    println!("derive(BinSerialize) for {}", input.ident);
    let (context, attr_errors) = Context::from_input(&input);
    let ident = &input.ident;
    let (generics, fields) = encode_type(context, input.generics, input.data, &input.ident);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinSerialize for #ident#ty_generics #where_clause {
            fn encode_to(&self, buf: &mut dyn ::binary::BinWrite, attrs: ::binary::attr::Attrs) -> ::binary::Result<()> {
                #fields
                Ok(())
            }
        }
        #attr_errors
    };
    #[cfg(feature = "debug_prints")]
    println!("=====\n{}\n=====", s);
    s.into()
}

#[proc_macro_derive(BinDeserialize, attributes(binary))]
pub fn derive_bindeserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    #[cfg(feature = "debug_prints")]
    println!("derive(BinDeserialize) for {}", input.ident);
    let (context, attr_errors) = Context::from_input(&input);
    let ident = &input.ident;
    let (generics, fields) = decode_type(context, input.generics, input.data, &input.ident);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinDeserialize for #ident#ty_generics #where_clause {
            fn decode_from(buf: &mut dyn ::binary::BinRead, attrs: ::binary::attr::Attrs) -> ::binary::Result<Self> {
                Ok({
                    #fields
                })
            }
        }
        #attr_errors
    };
    #[cfg(feature = "debug_prints")]
    println!("=====\n{}\n=====", s);
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
            path: bound,
        })]
        .into_iter()
        .collect(),
    })
}

fn encode_type(
    parent_context: Context,
    mut generics: Generics,
    data: Data,
    ident: &Ident,
) -> (Generics, TokenStream2) {
    match data {
        Data::Struct(s) => {
            let (generics, fields) = encode_fields(&parent_context, generics, s.fields);
            (
                generics,
                quote! {
                    #(#fields)*
                },
            )
        }
        Data::Enum(e) => {
            let mut variants: Vec<TokenStream2> = vec![];

            if parent_context.self_attrs.tag_ty.is_none() {
                let span = ident.span();
                return (
                    generics,
                    quote_spanned! {span=>
                        compile_error!("no tag type defined; enums deriving BinSerialize or BinDeserialize must have repr(uN) or repr(iN)");
                    },
                );
            }

            let tag_ty = parent_context.self_attrs.tag_ty.clone().unwrap();
            let mut tag = 0u64;
            let mut default_defined = false;

            for v in e.variants {
                let (context, attr_errors) = parent_context.recurse_into(Level::Variant, &v.attrs);
                let name = &v.ident;

                if context.self_attrs.tag_default {
                    let errors = if default_defined {
                        let span = name.span();
                        Some(quote_spanned! {span=>
                            compile_error!("multiple variants with a #[binary(default)] attribute found");
                        })
                    } else {
                        None
                    };
                    default_defined = true;
                    let tag_ty = &tag_ty.0;
                    let attrs = helpers::build_tag_attrs(parent_context.self_attrs.tag_le);
                    variants.push(quote! {
                        #ident::#name(tag, vec) => {
                            #attr_errors
                            #errors
                            <#tag_ty as ::binary::BinSerialize>::encode_to(tag, buf, #attrs)?;
                            <::std::vec::Vec<u8> as ::binary::BinSerialize>::encode_to(vec, buf, #attrs)?;
                        }
                    });
                } else {
                    tag = match helpers::find_discriminant(&v) {
                        Ok(Some(v)) => v,
                        Ok(None) => tag,
                        Err(e) => {
                            variants.push(e);
                            tag
                        }
                    };
                    let header = {
                        let tag_lit = LitInt::new(tag, tag_ty.1.clone(), v.span());
                        let attrs = helpers::build_tag_attrs(parent_context.self_attrs.tag_le);
                        quote! {
                            ::binary::BinSerialize::encode_to(&#tag_lit, buf, #attrs)?;
                        }
                    };
                    tag += 1;

                    let fields = pattern_fields(&v.fields);

                    let (newgen, encodes) = encode_fields(&context, generics, v.fields);
                    let mut encodes = quote! { #(#encodes)* };

                    if context.self_attrs.nest {
                        let attrs = helpers::build_nest_attrs(
                            context.self_attrs.nest_le,
                            context.self_attrs.nest_ty.unwrap(),
                        );
                        encodes = quote! {
                            let nested = {
                                let mut v = vec![];
                                let buf: &mut dyn ::binary::BinWrite = &mut v;
                                #encodes
                                v
                            };
                            ::binary::BinSerialize::encode_to(&nested, buf, #attrs)?;
                        }
                    }

                    generics = newgen;

                    variants.push(quote! {
                        #ident::#name#fields => {
                            #header
                            #(#encodes)*
                            #attr_errors
                        }
                    });
                }
            }
            let encode = quote! {
                match self {
                    #(#variants)*
                }
            };
            (generics, encode)
        }
        _ => unimplemented!(),
    }
}

fn pattern_fields(f: &Fields) -> TokenStream2 {
    match f {
        Fields::Named(f) => {
            let mut v = vec![];
            for f in &f.named {
                let base_ident = f.ident.clone().unwrap();
                let ident = Ident::new(
                    &format!("self_{}", f.ident.clone().unwrap()),
                    f.ident.span(),
                );
                v.push(quote! { #base_ident: #ident });
            }
            quote! { { #(#v),* } }
        }
        Fields::Unnamed(f) => {
            let mut v = vec![];
            for i in 0..f.unnamed.len() {
                let ident = Ident::new(&format!("self_{}", i), f.unnamed[i].span());
                v.push(quote! { #ident });
            }
            quote! { ( #(#v),* ) }
        }
        Fields::Unit => quote! {},
    }
}

fn decode_type(
    context: Context,
    mut generics: Generics,
    data: Data,
    ident: &Ident,
) -> (Generics, TokenStream2) {
    match data {
        Data::Struct(s) => {
            let (generics, decodes, transfers, errors) =
                decode_fields(&context, generics, s.fields);
            (
                generics,
                quote! {
                    #errors
                    #decodes
                    Self #transfers
                },
            )
        }
        Data::Enum(e) => {
            let mut variants: Vec<TokenStream2> = vec![];

            let header = if context.self_attrs.tag_ty.is_none() {
                let span = ident.span();
                quote_spanned! {span=>
                    compile_error!("no tag type defined; enums deriving BinSerialize or BinDeserialize must have repr(uN) or repr(iN)");
                    let variant = 0;
                }
            } else {
                let tag_ty = context.self_attrs.tag_ty.clone().unwrap().0;
                let attrs = helpers::build_tag_attrs(context.self_attrs.tag_le);
                quote! {
                    let variant = <#tag_ty as ::binary::BinDeserialize>::decode_from(buf, #attrs)?;
                }
            };

            let tag_ty = context.self_attrs.tag_ty.clone().unwrap();
            let mut tag = 0u64;
            let mut default_variant = None;

            for v in e.variants {
                let (context, attr_errors) = context.recurse_into(Level::Variant, &v.attrs);
                let name = &v.ident;
                if context.self_attrs.tag_default {
                    let errors = default_variant.as_ref().map(|_| {
                        let span = name.span();
                        quote_spanned!{span=>
                            compile_error!("multiple variants with a #[binary(default)] attribute found");
                        }
                    });
                    default_variant = Some(quote! {
                        _ => {
                            #errors
                            let vec = <::std::vec::Vec<u8> as ::binary::BinDeserialize>::decode_from(buf, ::binary::attr::Attrs::zero())?;
                            #ident::#name(variant, vec)
                        }
                    });
                } else {
                    tag = match helpers::find_discriminant(&v) {
                        Ok(Some(v)) => v,
                        Ok(None) => tag,
                        Err(e) => {
                            variants.push(e);
                            tag
                        }
                    };
                    let tag_lit = LitInt::new(tag, tag_ty.1.clone(), v.span());
                    tag += 1;
                    let (newgen, decodes, transfers, errors) =
                        decode_fields(&context, generics, v.fields);
                    generics = newgen;

                    let pre = if context.self_attrs.nest {
                        let attrs = helpers::build_nest_attrs(
                            context.self_attrs.nest_le,
                            context.self_attrs.nest_ty.unwrap(),
                        );
                        Some(quote! {
                            let v = <Vec<u8> as ::binary::BinDeserialize>::decode_from(buf, #attrs)?;
                            let buf: &mut dyn ::binary::BinRead = &mut v.as_slice();
                        })
                    } else {
                        None
                    };

                    variants.push(quote! {
                        #tag_lit => {
                            #attr_errors
                            #errors
                            #pre
                            #decodes
                            #ident::#name#transfers
                        }
                    });
                }
            }
            if default_variant.is_none() {
                default_variant = Some(quote! {
                    _ => return Err(::binary::BinError::VariantNotMatched(variant as u64))
                })
            }
            let decode = quote! {
                #header
                match variant {
                    #(#variants)*
                    #default_variant
                }
            };
            (generics, decode)
        }
        _ => unimplemented!(),
    }
}

fn encode_fields(
    context: &Context,
    mut generics: Generics,
    fields: Fields,
) -> (Generics, Vec<TokenStream2>) {
    let mut encodes = vec![];
    let mut tail = None;
    let fields = match fields {
        Fields::Named(n) => n.named,
        Fields::Unnamed(u) => u.unnamed,
        Fields::Unit => return (generics, vec![]),
    };
    let mut flags_ty = None;
    let mut warned_for_no_flags = false;
    for (i, f) in fields.into_iter().enumerate() {
        let span = f.span();
        let (context, attr_errors) = context.recurse_into(Level::Field, &f.attrs);

        generics
            .make_where_clause()
            .predicates
            .push(make_generic_bound(
                if context.self_attrs.flag_value.is_some() {
                    let ty = &f.ty;
                    parse_quote! {<#ty as ::binary::DeOption>::Assoc}
                } else {
                    f.ty.clone()
                },
                parse_quote! {::binary::BinSerialize},
            ));

        let ident: TokenStream2 = if context.env == Environment::Enum {
            let (name, span) = match &f.ident {
                Some(n) => (format!("self_{}", n), n.span()),
                None => (format!("self_{}", i), span),
            };
            let new_ident = Ident::new(&name, span);
            quote! { #new_ident }
        } else {
            let ident = match &f.ident {
                Some(i) => Member::Named(i.clone()),
                _ => Member::Unnamed(Index {
                    index: i as u32,
                    span,
                }),
            };
            quote! { self.#ident }
        };

        let attrs = context.build_attrs();

        if context.self_attrs.flags {
            if flags_ty.is_some() {
                let span = f.span();
                encodes.push(quote_spanned! {span=>
                    compile_error!("multiple #[binary(flags)] attributes in this struct");
                });
            }
            flags_ty = Some(f.ty.clone());
            let ty = &f.ty;
            encodes.push(quote! {
                let mut flags: #ty = <#ty as ::binary::BinFlags>::zero();
                let mut flagged = vec![];
                let unflagged_buf = buf;
                let buf: &mut dyn ::binary::BinWrite = &mut flagged;
            });
            tail = Some(quote! {
                <#ty as ::binary::BinSerialize>::encode_to(&flags, unflagged_buf, #attrs)?;
                <::std::vec::Vec<u8> as ::binary::BinSerialize>::encode_to(&flagged, unflagged_buf, ::binary::attr::Attrs::zero())?;
            });
        } else if let Some(v) = context.self_attrs.flag_value {
            let set = if flags_ty.is_none() {
                if !warned_for_no_flags {
                    warned_for_no_flags = true;
                    let span = f.span();
                    encodes.push(quote_spanned! {span=>
                        compile_error!("no #[binary(flags)] attribute before this #[binary(flags(...))]");
                    });
                }
                None
            } else {
                let v = LitInt::new(v, IntSuffix::None, span);
                Some(quote! { <#flags_ty as ::binary::BinFlags>::set(&mut flags, #v); })
            };
            encodes.push(quote! {
                if let Some(v) = &#ident {
                    #set
                    ::binary::BinSerialize::encode_to(v, buf, #attrs)?;
                }
            });
        } else {
            encodes.push(quote! {
                ::binary::BinSerialize::encode_to(&#ident, buf, #attrs)?;
                #attr_errors
            });
        }
    }
    if let Some(tail) = tail {
        encodes.push(tail);
    }
    (generics, encodes)
}

fn decode_fields(
    context: &Context,
    mut generics: Generics,
    fields: Fields,
) -> (Generics, TokenStream2, TokenStream2, TokenStream2) {
    let mut decodes: Vec<TokenStream2> = vec![];
    let mut transfers: Vec<TokenStream2> = vec![];
    let mut errors: Vec<TokenStream2> = vec![];
    let fields_list = match &fields {
        Fields::Named(n) => &n.named,
        Fields::Unnamed(u) => &u.unnamed,
        Fields::Unit => return (generics, quote! {}, quote! {}, quote! {}),
    };
    let mut flags_ty = None;
    let mut flags_field = None;
    let mut warned_for_no_flags = false;
    for (i, f) in fields_list.into_iter().enumerate() {
        let ty = f.ty.clone();
        let (context, attr_errors) = context.recurse_into(Level::Field, &f.attrs);

        generics
            .make_where_clause()
            .predicates
            .push(make_generic_bound(
                if context.self_attrs.flag_value.is_some() {
                    let ty = &f.ty;
                    parse_quote! {<#ty as ::binary::DeOption>::Assoc}
                } else {
                    f.ty.clone()
                },
                parse_quote! {::binary::BinDeserialize},
            ));

        let struct_ident = &f.ident;
        let colon = if struct_ident.is_some() {
            Some(quote! {:})
        } else {
            None
        };

        let ident: TokenStream2 = {
            let (name, span) = match &f.ident {
                Some(n) => (format!("self_{}", n), n.span()),
                None => (format!("self_{}", i), f.span()),
            };
            let new_ident = Ident::new(&name, span);
            quote! { #new_ident }
        };

        let attrs = context.build_attrs();
        if let Some(v) = context.self_attrs.flag_value {
            let span = ident.span();
            let has = if flags_ty.is_none() {
                if !warned_for_no_flags {
                    warned_for_no_flags = true;
                    let span = f.span();
                    decodes.push(quote_spanned! {span=>
                        compile_error!("no #[binary(flags)] attribute before this #[binary(flags(...))]");
                    });
                }
                quote! { false }
            } else {
                let v = LitInt::new(v, IntSuffix::None, span);
                quote! { <#flags_ty as ::binary::BinFlags>::has(&#flags_field, #v) }
            };
            decodes.push(quote! {
                let #ident = if #has {
                    Some(<<#ty as ::binary::DeOption>::Assoc as ::binary::BinDeserialize>::decode_from(buf, #attrs)?)
                } else {
                    None
                };
            });
        } else {
            decodes.push(quote! {
                let #ident = <#ty as ::binary::BinDeserialize>::decode_from(buf, #attrs)?;
            });
        }
        transfers.push(quote! {
            #struct_ident#colon #ident,
        });
        if context.self_attrs.flags {
            if flags_ty.is_some() {
                let span = f.span();
                decodes.push(quote_spanned! {span=>
                    compile_error!("multiple #[binary(flags)] attributes in this struct");
                });
            }
            flags_ty = Some(f.ty.clone());
            flags_field = Some(ident);
        }
        errors.push(attr_errors);
    }
    let errors = quote! { #(#errors)* };
    match fields {
        Fields::Named(_) => (
            generics,
            quote! { #(#decodes)* },
            quote! { { #(#transfers)* } },
            errors,
        ),
        Fields::Unnamed(_) => (
            generics,
            quote! { #(#decodes)* },
            quote! { ( #(#transfers)* ) },
            errors,
        ),
        _ => unreachable!(),
    }
}
