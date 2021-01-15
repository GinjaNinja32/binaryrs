use syn::export::TokenStream2;
use syn::{Attribute, Data, DeriveInput};

use crate::SelfAttrs;

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum Environment {
    Struct,
    Enum,
    Union,
}

impl Environment {
    fn from_data(data: &Data) -> Self {
        match data {
            Data::Struct(_) => Self::Struct,
            Data::Enum(_) => Self::Enum,
            Data::Union(_) => Self::Union,
        }
    }
}

/*
 * #[top]
 * enum ExampleEnum {
 *     #[variant]
 *     SomeVariant(#[field] u64),
 * }
 *
 * #[top]
 * struct ExampleStruct(#[field] u64)
 *
 */
#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum Level {
    Top,     // Enum or struct
    Variant, // Enum variant
    Field,   // Variant field or struct field
}

pub(crate) struct Context {
    pub(crate) env: Environment,
    pub(crate) attrs: TokenStream2,
    pub(crate) self_attrs: SelfAttrs,
}

impl Context {
    pub(crate) fn from_input(input: &DeriveInput) -> (Self, TokenStream2) {
        let env = Environment::from_data(&input.data);
        let initial = Self {
            env,
            attrs: quote! {},
            self_attrs: SelfAttrs {
                tag_ty: None,
                tag_le: None,
                nest: false,
                nest_variants: false,
                nest_ty: None,
                nest_le: None,
            },
        };

        initial.recurse_into(Level::Top, &input.attrs)
    }
    pub(crate) fn recurse_into(&self, level: Level, attrs: &[Attribute]) -> (Self, TokenStream2) {
        let old_attrs = &self.attrs;

        let (attrs, mut self_attrs, attr_errors) =
            crate::helpers::parse_attrs(attrs, (self.env, level));

        if self.self_attrs.nest_variants {
            self_attrs.nest = true;
            self_attrs.nest_ty = self.self_attrs.nest_ty;
            self_attrs.nest_le = self.self_attrs.nest_le;
        }

        (
            Self {
                env: self.env,
                attrs: quote! { #old_attrs #attrs },
                self_attrs,
            },
            attr_errors,
        )
    }

    pub(crate) fn build_attrs(&self) -> TokenStream2 {
        let attrs = &self.attrs;
        quote! {
            {
                let mut attrs = attrs.clone();
                #attrs
                attrs
            }
        }
    }
}
