use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Ident, IntSuffix, Lit, Meta, NestedMeta, Type, Variant};

use crate::context::{Environment, Level};
use crate::SelfAttrs;

#[derive(Copy, Clone)]
pub(crate) enum SizeType {
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
    fn build_attr_form(&self) -> TokenStream2 {
        match self {
            SizeType::U8 => quote! {::binary::attr::Len::U8},
            SizeType::U16 => quote! {::binary::attr::Len::U16},
            SizeType::U32 => quote! {::binary::attr::Len::U32},
            SizeType::U64 => quote! {::binary::attr::Len::U64},
            SizeType::I8 => quote! {::binary::attr::Len::I8},
            SizeType::I16 => quote! {::binary::attr::Len::I16},
            SizeType::I32 => quote! {::binary::attr::Len::I32},
            SizeType::I64 => quote! {::binary::attr::Len::I64},
        }
    }
}

pub(crate) trait SizeTypeExt {
    fn build_attr_form(&self) -> TokenStream2;
}
impl SizeTypeExt for Option<SizeType> {
    fn build_attr_form(&self) -> TokenStream2 {
        match self {
            None => quote! {None},
            Some(t) => {
                let t = t.build_attr_form();
                quote! {Some(#t)}
            }
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
        nest: false,
        nest_variants: false,
        nest_ty: None,
        nest_le: None,
        flags: false,
        flag_value: None,
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
                                        "nest" => {
                                            if context != (Environment::Enum, Level::Top) {
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal attribute target");
                                                });
                                            } else {
                                                self_attrs.nest_variants = true;
                                            }
                                        }
                                        "flags" => {
                                            if context.1 != Level::Field {
                                                errors.push(quote_spanned! {span=>
                                                    compile_error!("illegal attribute target");
                                                });
                                            } else {
                                                self_attrs.flags = true;
                                            }
                                        }
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
                                            let span = list.span();
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute target");
                                            });
                                        } else {
                                            for elem in &list.nested {
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
                                    "nest" => {
                                        if context != (Environment::Enum, Level::Top) {
                                            let span = list.span();
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute target");
                                            });
                                        } else {
                                            self_attrs.nest_variants = true;
                                            for elem in &list.nested {
                                                match elem {
                                                    NestedMeta::Meta(Meta::Word(word)) => {
                                                        let span = word.span();
                                                        let s = word.to_string();
                                                        match s.as_str() {
                                                            "little" => {
                                                                self_attrs.nest_le = Some(true)
                                                            }
                                                            "big" => {
                                                                self_attrs.nest_le = Some(false)
                                                            }
                                                            _ => {
                                                                match parse_size_attr_arg(word) {
                                                                    Ok(Some(v)) => self_attrs.nest_ty = Some(v),
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
                                    "flags" => {
                                        let span = list.span();
                                        if context.1 != Level::Field {
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute target");
                                            });
                                        } else if list.nested.len() != 1 {
                                            errors.push(quote_spanned! {span=>
                                                compile_error!("illegal attribute argument");
                                            });
                                        } else {
                                            match &list.nested[0] {
                                                NestedMeta::Literal(Lit::Int(i)) => {
                                                    self_attrs.flag_value = Some(i.value());
                                                }
                                                _ => {
                                                    errors.push(quote_spanned! {span=>
                                                        compile_error!("illegal attribute argument");
                                                    });
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
    let byteorder = if tag_le.unwrap_or(true) {
        quote! { ::binary::attr::Endian::Little; }
    } else {
        quote! { ::binary::attr::Endian::Big; }
    };

    quote! {
        {
            let mut attrs = ::binary::attr::Attrs::zero();
            attrs.endian = #byteorder;
            attrs
        }
    }
}

pub(crate) fn build_nest_attrs(nest_le: Option<bool>, nest_ty: SizeType) -> TokenStream2 {
    let byteorder = if nest_le.unwrap_or(true) {
        quote! { ::binary::attr::Endian::Little }
    } else {
        quote! { ::binary::attr::Endian::Big }
    };
    let len = nest_ty.build_attr_form();

    quote! {
        {
            let mut attrs = ::binary::attr::Attrs::zero();
            attrs.len = Some(#len);
            attrs.len_endian = #byteorder;
            attrs
        }
    }
}
