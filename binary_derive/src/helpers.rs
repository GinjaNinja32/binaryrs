use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Ident, IntSuffix, Lit, Meta, NestedMeta, Type, Variant};

use crate::context::{Environment, Level};
use crate::SelfAttrs;

enum SizeType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl SizeType {
    fn to_type_suffix(&self) -> (Type, IntSuffix) {
        match self {
            Self::U8 => (parse_quote! {u8}, IntSuffix::U8),
            Self::U16 => (parse_quote! {u16}, IntSuffix::U16),
            Self::U32 => (parse_quote! {u32}, IntSuffix::U32),
            Self::U64 => (parse_quote! {u64}, IntSuffix::U64),
            Self::I8 => (parse_quote! {i8}, IntSuffix::I8),
            Self::I16 => (parse_quote! {i16}, IntSuffix::I16),
            Self::I32 => (parse_quote! {i32}, IntSuffix::I32),
            Self::I64 => (parse_quote! {i64}, IntSuffix::I64),
        }
    }
}

trait SizeTypeExt {
    fn build_attr_form(&self) -> TokenStream2;
}
impl SizeTypeExt for Option<SizeType> {
    fn build_attr_form(&self) -> TokenStream2 {
        match self {
            None => quote! {None},
            Some(SizeType::U8) => quote! {Some(::binary::attr::Len::U8)},
            Some(SizeType::U16) => quote! {Some(::binary::attr::Len::U16)},
            Some(SizeType::U32) => quote! {Some(::binary::attr::Len::U32)},
            Some(SizeType::U64) => quote! {Some(::binary::attr::Len::U64)},
            Some(SizeType::I8) => quote! {Some(::binary::attr::Len::I8)},
            Some(SizeType::I16) => quote! {Some(::binary::attr::Len::I16)},
            Some(SizeType::I32) => quote! {Some(::binary::attr::Len::I32)},
            Some(SizeType::I64) => quote! {Some(::binary::attr::Len::I64)},
        }
    }
}

pub(crate) fn parse_attrs(
    input: &[Attribute],
    context: (Environment, Level),
) -> (TokenStream2, SelfAttrs, TokenStream2) {
    let mut attrs = vec![];
    let mut self_attrs = SelfAttrs {
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
                if l.ident == "repr" {
                    if context != (Environment::Enum, Level::Top) {
                        continue; // ignore, this isn't our attr to complain about
                    }
                    for elem in &l.nested {
                        match elem {
                            NestedMeta::Meta(m) => match &m {
                                Meta::Word(w) => {
                                    let span = w.span();
                                    match parse_size_attr_arg(w) {
                                        Ok(Some(v)) => self_attrs.tag_ty = Some(v.to_type_suffix()),
                                        Ok(None) => continue,
                                        Err(None) => errors.push(quote_spanned! {span=>
                                            compile_error!("unknown attribute");
                                        }),
                                        Err(Some(v)) => errors.push(v),
                                    }
                                }
                                _ => continue, // ignore, this isn't our attr to complain about
                            },
                            _ => continue, // ignore, this isn't our attr to complain about
                        }
                    }
                } else if l.ident == "binary" {
                    for elem in &l.nested {
                        match elem {
                            NestedMeta::Meta(meta) => match &meta {
                                Meta::Word(word) => {
                                    let span = word.span();
                                    let s = word.to_string();
                                    match s.as_str() {
                                        "little" => attrs.push(quote_spanned! {span=>
                                            attrs.endian = ::binary::attr::Endian::Little;
                                        }),
                                        "big" => attrs.push(quote_spanned! {span=>
                                            attrs.endian = ::binary::attr::Endian::Big;
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
                                Meta::List(list) => match list.ident.to_string().as_str() {
                                    "len" => {
                                        for elem in &list.nested {
                                            match elem {
                                                NestedMeta::Meta(Meta::Word(word)) => {
                                                    let span = word.span();
                                                    let s = word.to_string();
                                                    match s.as_str() {
                                                        "little" => attrs.push(quote_spanned! {span=>
                                                            attrs.len_endian = ::binary::attr::Endian::Little;
                                                        }),
                                                        "big" => attrs.push(quote_spanned! {span=>
                                                            attrs.len_endian = ::binary::attr::Endian::Big;
                                                        }),
                                                        _ => {
                                                            match parse_size_attr_arg(word) {
                                                                Ok(v) => {
                                                                    let ty = v.build_attr_form();
                                                                    attrs.push(quote!{
                                                                        attrs.len = #ty;
                                                                    });
                                                                }
                                                                Err(None) => {
                                                                    errors.push(quote_spanned! {span=>
                                                                        compile_error!("unknown attribute");
                                                                    })
                                                                }
                                                                Err(Some(v)) => errors.push(v),
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    let span = elem.span();
                                                    errors.push(quote_spanned! {span=>
                                                        compile_error!("illegal attribute form");
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    "tag" => {
                                        if context != (Environment::Enum, Level::Top) {
                                            let span = l.span();
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute target");
                                            });
                                        } else {
                                            for elem in &l.nested {
                                                match elem {
                                                    NestedMeta::Meta(Meta::Word(word)) => {
                                                        let span = word.span();
                                                        let s = word.to_string();
                                                        match s.as_str() {
                                                            "little" => {
                                                                self_attrs.tag_le = Some(true)
                                                            }
                                                            "big" => {
                                                                self_attrs.tag_le = Some(false)
                                                            }
                                                            _ => {
                                                                match parse_size_attr_arg(word) {
                                                                    Ok(Some(v)) => self_attrs.tag_ty = Some(v.to_type_suffix()),
                                                                    Err(None) | Ok(None) => {
                                                                        errors.push(quote_spanned! {span=>
                                                                            compile_error!("unknown attribute");
                                                                        })
                                                                    }
                                                                    Err(Some(v)) => errors.push(v),
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        let span = elem.span();
                                                        errors.push(quote_spanned! {span=>
                                                        compile_error!("illegal attribute form");
                                                    });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        let span = meta.span();
                                        errors.push(quote_spanned! {span=>
                                            compile_error!("illegal attribute form");
                                        });
                                    }
                                },
                                _ => {
                                    let span = meta.span();
                                    errors.push(quote_spanned! {span=>
                                        compile_error!("illegal attribute form");
                                    });
                                }
                            },
                            _ => {
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

    (
        quote! {
            #(#attrs)*
        },
        self_attrs,
        quote! {
            #(#errors)*
        },
    )
}

fn parse_size_attr_arg(ident: &Ident) -> Result<Option<SizeType>, Option<TokenStream2>> {
    let span = ident.span();
    match ident.to_string().as_str() {
        "none" => Ok(None),
        "u8" => Ok(Some(SizeType::U8)),
        "u16" => Ok(Some(SizeType::U16)),
        "u32" => Ok(Some(SizeType::U32)),
        "u64" => Ok(Some(SizeType::U64)),
        "usize" => Err(Some(quote_spanned! {span=>
            compile_error!("Bin(De)Serialize cannot (de)serialize usize correctly cross-platform; use an integer of specific size");
        })),
        "i8" => Ok(Some(SizeType::I8)),
        "i16" => Ok(Some(SizeType::I16)),
        "i32" => Ok(Some(SizeType::I32)),
        "i64" => Ok(Some(SizeType::I64)),
        "isize" => Err(Some(quote_spanned! {span=>
            compile_error!("Bin(De)Serialize cannot (de)serialize usize correctly cross-platform; use an integer of specific size");
        })),
        _ => Err(None),
    }
}

pub(crate) fn find_discriminant(v: &Variant) -> Result<Option<u64>, TokenStream2> {
    if let Some((_, expr)) = &v.discriminant {
        // explicit discriminant
        if let Expr::Lit(lit) = expr {
            if let Lit::Int(i) = &lit.lit {
                Ok(Some(i.value()))
            } else {
                let span = expr.span();
                Err(quote_spanned! {span=>
                    compile_error!("derive(Bin(De)serialize) expected a literal integer here");
                })
            }
        } else {
            let span = expr.span();
            Err(quote_spanned! {span=>
                compile_error!("derive(Bin(De)serialize) expected a literal here");
            })
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn build_tag_attrs(tag_le: Option<bool>) -> TokenStream2 {
    let tag_byteorder = if tag_le.unwrap_or(false) {
        quote! { attrs.endian = ::binary::attr::Endian::Little; }
    } else {
        quote! { attrs.endian = ::binary::attr::Endian::Big; }
    };

    quote! {
        {
            let mut attrs = ::binary::attr::Attrs::zero();
            #tag_byteorder
            attrs
        }
    }
}
