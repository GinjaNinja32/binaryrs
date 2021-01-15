use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, IntSuffix, Lit, Meta, MetaNameValue, NestedMeta, Variant};

use crate::{Context, SelfAttrs};

pub(crate) fn parse_attrs(
    input: Vec<Attribute>,
    context: Context,
) -> (Vec<TokenStream2>, SelfAttrs, Vec<TokenStream2>) {
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
                    if context != Context::EnumHeader {
                        continue; // ignore, this isn't our attr to complain about
                    }
                    for elem in &l.nested {
                        match elem {
                            NestedMeta::Meta(m) => match &m {
                                Meta::Word(w) => {
                                    let span = w.span();
                                    let s = w.to_string();
                                    match s.as_str() {
                                        "u8" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {u8}, IntSuffix::U8))
                                        }
                                        "u16" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {u16}, IntSuffix::U16))
                                        }
                                        "u32" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {u32}, IntSuffix::U32))
                                        }
                                        "u64" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {u64}, IntSuffix::U64))
                                        }
                                        "usize" => {
                                            errors.push(quote_spanned!{span=>
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     compile_error!("Bin(De)Serialize requires enums to have repr(iN) or repr(uN), not repr(usize)");
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   });
                                        }
                                        "i8" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {i8}, IntSuffix::I8))
                                        }
                                        "i16" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {i16}, IntSuffix::I16))
                                        }
                                        "i32" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {i32}, IntSuffix::I32))
                                        }
                                        "i64" => {
                                            self_attrs.tag_ty =
                                                Some((parse_quote! {i64}, IntSuffix::I64))
                                        }
                                        "isize" => {
                                            errors.push(quote_spanned!{span=>
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       compile_error!("Bin(De)Serialize requires enums to have repr(iN) or repr(uN), not repr(isize)");
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     });
                                        }
                                        _ => continue, // ignore, this isn't our attr to complain about
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
