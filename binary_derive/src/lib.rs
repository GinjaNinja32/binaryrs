#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Fields, Generics, Ident, Index, LitInt, Member, Path, Type, WherePredicate,
};

mod helpers;

struct SelfAttrs {
    tag_ty: Option<(syn::Type, syn::IntSuffix)>,
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

#[proc_macro_derive(BinSerialize, attributes(binary))]
pub fn derive_binserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (attrs, mut self_attrs, attr_errors) =
        helpers::parse_attrs(input.attrs, Context::from_data(&input.data), &quote! {});
    let ident = &input.ident;
    let (generics, fields) = encode_type(
        input.generics,
        input.data,
        &input.ident,
        &mut self_attrs,
        attrs,
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
        #attr_errors
    };
    s.into()
}

#[proc_macro_derive(BinDeserialize, attributes(binary))]
pub fn derive_bindeserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (attrs, mut self_attrs, attr_errors) =
        helpers::parse_attrs(input.attrs, Context::from_data(&input.data), &quote! {});
    let ident = &input.ident;
    let (generics, fields) = decode_type(
        input.generics,
        input.data,
        &input.ident,
        &mut self_attrs,
        attrs,
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
        #attr_errors
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
    struct_attrs: TokenStream2,
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
                        compile_error!("no span type defined; enums deriving BinSerialize or BinDeserialize must have repr(uN) or repr(iN)");
                    },
                );
            }

            let tag_ty = self_attrs.tag_ty.clone().unwrap();
            let mut tag = 0u64;

            for v in e.variants {
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
                    let attrs = helpers::build_tag_attrs(self_attrs.tag_le);
                    quote! {
                        ::binary::BinSerialize::encode_to(&#tag_lit, buf, #attrs)?;
                    }
                };
                tag += 1;

                let (variant_attrs, _variant_self_attrs, attr_errors) =
                    helpers::parse_attrs(v.attrs, Context::EnumVariant, &struct_attrs);

                let name = v.ident;
                let fields = pattern_fields(&v.fields);
                let (newgen, encodes) =
                    encode_fields(generics, v.fields, Context::EnumField, variant_attrs);
                generics = newgen;
                variants.push(quote! {
                    #ident::#name#fields => {
                        #header
                        #(#encodes)*
                    }
                    #attr_errors
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
    struct_attrs: TokenStream2,
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
                    compile_error!("no span type defined; enums deriving BinSerialize or BinDeserialize must have repr(uN) or repr(iN)");
                    let variant = 0;
                }
            } else {
                let tag_ty = self_attrs.tag_ty.clone().unwrap().0;
                let attrs = helpers::build_tag_attrs(self_attrs.tag_le);
                quote! {
                    let variant = <#tag_ty as ::binary::BinDeserialize>::decode_from(buf, #attrs)?;
                }
            };

            let tag_ty = self_attrs.tag_ty.clone().unwrap();
            let mut tag = 0u64;

            for v in e.variants {
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
                let (variant_attrs, _variant_self_attrs, attr_errors) =
                    helpers::parse_attrs(v.attrs, Context::EnumVariant, &struct_attrs);
                let name = v.ident;
                let (newgen, fields) =
                    decode_fields(generics, v.fields, Context::EnumField, variant_attrs);
                generics = newgen;
                variants.push(quote! {
                    #tag_lit => {
                        #ident::#name#fields
                    }
                    #attr_errors
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
    struct_attrs: TokenStream2,
) -> (Generics, Vec<TokenStream2>) {
    let mut encodes = vec![];
    let fields = match fields {
        Fields::Named(n) => n.named,
        Fields::Unnamed(u) => u.unnamed,
        Fields::Unit => return (generics, vec![]),
    };
    for (i, f) in fields.into_iter().enumerate() {
        let span = f.span();
        let (attrs, _field_attrs, attr_errors) = helpers::parse_attrs(f.attrs, ctx, &struct_attrs);

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
                #attrs
                attrs
            })?;
            #attr_errors
        });
    }
    (generics, encodes)
}

fn decode_fields(
    mut generics: Generics,
    fields: Fields,
    ctx: Context,
    struct_attrs: TokenStream2,
) -> (Generics, TokenStream2) {
    let mut decodes: Vec<TokenStream2> = vec![];
    let fields_list = match &fields {
        Fields::Named(n) => &n.named,
        Fields::Unnamed(u) => &u.unnamed,
        Fields::Unit => return (generics, quote! {}),
    };
    for f in fields_list {
        let ty = f.ty.clone();
        let (attrs, _self_attrs, attr_errors) =
            helpers::parse_attrs(f.attrs.clone(), ctx, &struct_attrs);

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
                #attrs
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
