#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Fields, Generics, Ident, Index, IntSuffix, Lit, LitInt, Member,
    Meta, MetaNameValue, NestedMeta, Path, Type, WherePredicate,
};

struct SelfAttrs {
    tag: Option<u64>,
    tag_ty: Option<syn::IntSuffix>,
    tag_le: Option<bool>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Context {
    StructHeader,
    StructField,
    EnumHeader,
    EnumVariant,
    EnumField,
    UnionHeader,
    // UnionField,
}

impl Context {
    fn from_data(data: &Data) -> Self {
        match data {
            Data::Struct(_) => Self::StructHeader,
            Data::Enum(_) => Self::EnumHeader,
            Data::Union(_) => Self::UnionHeader,
        }
    }
}

fn parse_attrs(
    input: Vec<Attribute>,
    context: Context,
) -> (Vec<TokenStream2>, SelfAttrs, Vec<TokenStream2>) {
    let mut attrs = vec![];
    let mut self_attrs = SelfAttrs {
        tag: None,
        tag_ty: None,
        tag_le: None,
    };
    let mut errors = vec![];

    for attr in input {
        let data = match attr.parse_meta() {
            Ok(v) => v,
            Err(e) => {
                errors.push(e.to_compile_error());
                continue;
            }
        };
        match &data {
            Meta::Word(w) => {
                if w == "binary" {
                    let span = data.span();
                    errors.push(quote_spanned! {span=>
                        compile_error!("illegal attribute form");
                    });
                }
                continue;
            }
            Meta::List(l) => {
                if l.ident == "binary" {
                    for elem in &l.nested {
                        match elem {
                            NestedMeta::Meta(m) => match &m {
                                Meta::Word(w) => {
                                    let span = w.span();
                                    let s = w.to_string();
                                    match s.as_str() {
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
                                        "tag_little" | "tag_big" => {
                                            if context != Context::EnumHeader {
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal attribute target");
                                                });
                                            } else {
                                                self_attrs.tag_le = Some(s == "tag_little");
                                            }
                                        }
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
                                    "len" => match parse_size_attr_arg(nv) {
                                        Ok(v) => attrs.push(v),
                                        Err(v) => errors.push(v),
                                    },
                                    "tag_len" => match &nv.lit {
                                        Lit::Int(i) => {
                                            let ty = match i.value() {
                                                1 => IntSuffix::U8,
                                                2 => IntSuffix::U16,
                                                4 => IntSuffix::U32,
                                                8 => IntSuffix::U64,
                                                _ => {
                                                    let span = nv.lit.span();
                                                    errors.push(quote_spanned! {span=>
                                                        compile_error!("illegal attribute argument")
                                                    });
                                                    continue;
                                                }
                                            };
                                            if context != Context::EnumHeader {
                                                let span = nv.span();
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal attribute target");
                                                });
                                            } else {
                                                self_attrs.tag_ty = Some(ty);
                                            }
                                        }
                                        _ => {
                                            let span = nv.lit.span();
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute argument");
                                            })
                                        }
                                    },
                                    "tag" => match &nv.lit {
                                        Lit::Int(i) => {
                                            if context != Context::EnumVariant {
                                                let span = nv.span();
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal attribute target");
                                                });
                                            } else {
                                                let span = nv.lit.span();
                                                attrs.push(quote_spanned! {span=>
                                                    attrs.tag = #i;
                                                })
                                            }
                                        }
                                        _ => {
                                            let span = nv.lit.span();
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute argument");
                                            })
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
            }
            Meta::NameValue(nv) => {
                if nv.ident == "binary" {
                    let span = data.span();
                    errors.push(quote_spanned! {span=>
                        compile_error!("illegal attribute form");
                    });
                }
                continue;
            }
        }
    }

    (attrs, self_attrs, errors)
}

fn parse_size_attr_arg(nv: &MetaNameValue) -> Result<TokenStream2, TokenStream2> {
    match &nv.lit {
        Lit::Int(i) => {
            let span = nv.span();
            let name = &nv.ident;
            match i.value() {
                0 => Ok(quote_spanned! {span=>
                    attrs.#name = None;
                }),
                1 => Ok(quote_spanned! {span=>
                    attrs.#name = Some(::binary::attr::Len::U8);
                }),
                2 => Ok(quote_spanned! {span=>
                    attrs.#name = Some(::binary::attr::Len::U16);
                }),
                4 => Ok(quote_spanned! {span=>
                    attrs.#name = Some(::binary::attr::Len::U32);
                }),
                8 => Ok(quote_spanned! {span=>
                    attrs.#name = Some(::binary::attr::Len::U64);
                }),
                _ => {
                    let span = nv.lit.span();
                    Err(quote_spanned! {span=>
                        compile_error!("illegal attribute argument");
                    })
                }
            }
        }
        _ => {
            let span = nv.lit.span();
            Err(quote_spanned! {span=>
                compile_error!("illegal attribute argument");
            })
        }
    }
}

#[proc_macro_derive(BinSerialize, attributes(binary))]
pub fn derive_binserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (attrs, mut self_attrs, attr_errors) =
        parse_attrs(input.attrs, Context::from_data(&input.data));
    let ident = &input.ident;
    let (generics, fields) = encode_type(
        input.generics,
        input.data,
        &input.ident,
        &mut self_attrs,
        &attrs,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinSerialize for #ident#ty_generics #where_clause {
            fn encode_to(&self, buf: &mut dyn ::binary::BufMut, attrs: ::binary::attr::Attrs) -> ::binary::Result<()> {
                #fields
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
    let (attrs, mut self_attrs, attr_errors) =
        parse_attrs(input.attrs, Context::from_data(&input.data));
    let ident = &input.ident;
    let (generics, fields) = decode_type(
        input.generics,
        input.data,
        &input.ident,
        &mut self_attrs,
        &attrs,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let s = quote! {
        #[automatically_derived]
        impl#impl_generics ::binary::BinDeserialize for #ident#ty_generics #where_clause {
            fn decode_from(buf: &mut dyn ::binary::Buf, attrs: ::binary::attr::Attrs) -> ::binary::Result<Self> {
                Ok({
                    #fields
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
            path: bound,
        })]
        .into_iter()
        .collect(),
    })
}

fn encode_type(
    mut generics: Generics,
    data: Data,
    ident: &Ident,
    self_attrs: &mut SelfAttrs,
    struct_attrs: &[TokenStream2],
) -> (Generics, TokenStream2) {
    match data {
        Data::Struct(s) => {
            let (generics, fields) =
                encode_fields(generics, s.fields, Context::StructField, struct_attrs);
            (
                generics,
                quote! {
                    #(#fields)*
                },
            )
        }
        Data::Enum(e) => {
            let mut variants: Vec<TokenStream2> = vec![];

            if self_attrs.tag_ty.is_none() {
                let span = ident.span();
                return (
                    generics,
                    quote_spanned! {span=>
                        compile_error!("no span type defined");
                    },
                );
            }

            for v in e.variants {
                let (mut variant_attrs, variant_self_attrs, attr_errors) =
                    parse_attrs(v.attrs, Context::EnumVariant);
                let struct_attrs = {
                    let mut attrs = vec![];
                    attrs.extend_from_slice(struct_attrs);
                    attrs.append(&mut variant_attrs);
                    attrs
                };

                let header = {
                    if let Some(v) = variant_self_attrs.tag {
                        self_attrs.tag = Some(v)
                    } else if self_attrs.tag.is_none() {
                        self_attrs.tag = Some(0);
                    } else {
                        *self_attrs.tag.iter_mut().next().unwrap() += 1;
                    }

                    let lit = LitInt::new(
                        self_attrs.tag.unwrap(),
                        self_attrs.tag_ty.clone().unwrap(),
                        ident.span(),
                    );

                    let v = if self_attrs.tag_le.unwrap_or(false) {
                        quote! { attrs.endian = ::binary::attr::Endian::Little; }
                    } else {
                        quote! { attrs.endian = ::binary::attr::Endian::Big; }
                    };
                    quote! {
                        ::binary::BinSerialize::encode_to(&#lit, buf, {
                            let mut attrs = ::binary::attr::Attrs::zero();
                            #v
                            attrs
                        })?;
                    }
                };

                let name = v.ident;
                let fields = pattern_fields(&v.fields);
                let (newgen, encodes) =
                    encode_fields(generics, v.fields, Context::EnumField, &struct_attrs);
                generics = newgen;
                variants.push(quote! {
                    #ident::#name#fields => {
                        #header
                        #(#encodes)*
                    }
                    #(#attr_errors)*
                });
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
    mut generics: Generics,
    data: Data,
    ident: &Ident,
    self_attrs: &mut SelfAttrs,
    struct_attrs: &[TokenStream2],
) -> (Generics, TokenStream2) {
    match data {
        Data::Struct(s) => {
            let (generics, fields) =
                decode_fields(generics, s.fields, Context::StructField, struct_attrs);
            (
                generics,
                quote! {
                    Self #(#fields)*
                },
            )
        }
        Data::Enum(e) => {
            let mut variants: Vec<TokenStream2> = vec![];

            let header = if self_attrs.tag_ty.is_none() {
                let span = ident.span();
                quote_spanned! {span=>
                    compile_error!("no span type defined");
                    let variant = 0;
                }
            } else {
                let dec_ty: Type = match self_attrs.tag_ty.clone().unwrap() {
                    IntSuffix::U8 => parse_quote! {u8},
                    IntSuffix::U16 => parse_quote! {u16},
                    IntSuffix::U32 => parse_quote! {u32},
                    IntSuffix::U64 => parse_quote! {u64},
                    _ => unreachable!(),
                };

                let v = if self_attrs.tag_le.unwrap_or(false) {
                    quote! { attrs.endian = ::binary::attr::Endian::Little; }
                } else {
                    quote! { attrs.endian = ::binary::attr::Endian::Big; }
                };
                quote! {
                    let variant = <#dec_ty as ::binary::BinDeserialize>::decode_from(buf, {
                        let mut attrs = ::binary::attr::Attrs::zero();
                        #v
                        attrs
                    })?;
                }
            };

            for v in e.variants {
                let (mut variant_attrs, variant_self_attrs, attr_errors) =
                    parse_attrs(v.attrs, Context::EnumVariant);
                let struct_attrs = {
                    let mut attrs = vec![];
                    attrs.extend_from_slice(struct_attrs);
                    attrs.append(&mut variant_attrs);
                    attrs
                };
                if let Some(tag) = variant_self_attrs.tag {
                    self_attrs.tag = Some(tag);
                } else if self_attrs.tag.is_none() {
                    self_attrs.tag = Some(0);
                } else {
                    *self_attrs.tag.iter_mut().next().unwrap() += 1;
                }
                let name = v.ident;
                let (newgen, fields) =
                    decode_fields(generics, v.fields, Context::EnumField, &struct_attrs);
                generics = newgen;
                let tag = self_attrs.tag.unwrap();
                let tag_lit = LitInt::new(tag, self_attrs.tag_ty.clone().unwrap(), name.span());
                variants.push(quote! {
                    #tag_lit => {
                        #ident::#name#fields
                    }
                    #(#attr_errors)*
                });
            }
            let decode = quote! {
                #header
                match variant {
                    #(#variants)*
                    _ => return Err(::binary::BinError::VariantNotMatched(variant as u64))
                }
            };
            (generics, decode)
        }
        _ => unimplemented!(),
    }
}

fn encode_fields(
    mut generics: Generics,
    fields: Fields,
    ctx: Context,
    struct_attrs: &[TokenStream2],
) -> (Generics, Vec<TokenStream2>) {
    let mut encodes = vec![];
    let fields = match fields {
        Fields::Named(n) => n.named,
        Fields::Unnamed(u) => u.unnamed,
        Fields::Unit => return (generics, vec![]),
    };
    for (i, f) in fields.into_iter().enumerate() {
        let span = f.span();
        let (attrs, _field_attrs, attr_errors) = parse_attrs(f.attrs, ctx);

        generics
            .make_where_clause()
            .predicates
            .push(make_generic_bound(
                f.ty,
                parse_quote! {::binary::BinSerialize},
            ));

        let ident: TokenStream2 = if ctx == Context::EnumField {
            let (name, span) = match f.ident {
                Some(n) => (format!("self_{}", n), n.span()),
                None => (format!("self_{}", i), span),
            };
            let new_ident = Ident::new(&name, span);
            quote! { #new_ident }
        } else {
            let ident = match f.ident {
                Some(i) => Member::Named(i),
                _ => Member::Unnamed(Index {
                    index: i as u32,
                    span,
                }),
            };
            quote! { self.#ident }
        };

        encodes.push(quote! {
            ::binary::BinSerialize::encode_to(&#ident, buf, {
                let mut attrs = attrs.clone();
                #(#struct_attrs)*
                #(#attrs)*
                attrs
            })?;
            #(#attr_errors)*
        });
    }
    (generics, encodes)
}

fn decode_fields(
    mut generics: Generics,
    fields: Fields,
    ctx: Context,
    struct_attrs: &[TokenStream2],
) -> (Generics, TokenStream2) {
    let mut decodes: Vec<TokenStream2> = vec![];
    let fields_list = match &fields {
        Fields::Named(n) => &n.named,
        Fields::Unnamed(u) => &u.unnamed,
        Fields::Unit => return (generics, quote! {}),
    };
    for f in fields_list {
        let ty = f.ty.clone();
        let (attrs, _self_attrs, attr_errors) = parse_attrs(f.attrs.clone(), ctx);

        generics
            .make_where_clause()
            .predicates
            .push(make_generic_bound(
                f.ty.clone(),
                parse_quote! {::binary::BinDeserialize},
            ));

        let ident = &f.ident;
        let colon = if ident.is_some() {
            Some(quote! {:})
        } else {
            None
        };

        decodes.push(quote! {
            #ident#colon <#ty as ::binary::BinDeserialize>::decode_from(buf, {
                let mut attrs = attrs.clone();
                #(#struct_attrs)*
                #(#attrs)*
                attrs
            })?,
            #(#attr_errors)*
        });
    }
    match fields {
        Fields::Named(_) => (generics, quote! { { #(#decodes)* } }),
        Fields::Unnamed(_) => (generics, quote! { ( #(#decodes)* ) }),
        _ => unreachable!(),
    }
}
