use std::collections::HashMap;

use crate::util::{try_get_impl_fn, try_get_single_value_from_fn};
use quote::ToTokens;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprCast, ExprPath, Ident, LitInt};

pub(crate) struct CCInfo<'ast> {
    pub cc_name: &'ast Ident,
    pub cc_id: &'ast Expr,
    pub cc_command: Option<&'ast LitInt>,
}

pub(crate) struct CCInfoExtractor<'ast> {
    pub ccs: Vec<CCInfo<'ast>>,
    pub cc_command_enum_variants: HashMap<String, &'ast LitInt>,
}

impl<'ast> Visit<'ast> for CCInfoExtractor<'ast> {
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        // We're looking for enums with name "...CCCommand"
        let enum_name = i.ident.to_string();
        if !enum_name.ends_with("CCCommand") {
            return;
        }

        // ...that have a repr(u8) attribute
        let has_repr_u8 = i.attrs.iter().any(|a| {
            if a.path().get_ident().map(|i| i.to_string()) == Some("repr".to_string()) {
                if let Ok(list) = a.meta.require_list() {
                    return list.tokens.to_string() == "u8";
                }
            }
            false
        });
        if !has_repr_u8 {
            return;
        }

        // Filter enum variants that have a literal value
        let enum_variants = i.variants.iter().filter_map(|v| {
            let variant_name = &v.ident;
            let variant_value = v.discriminant.as_ref()?;
            let variant_value = match &variant_value.1 {
                syn::Expr::Lit(lit) => lit,
                _ => return None,
            };
            let variant_value = match variant_value.lit {
                syn::Lit::Int(ref lit_int) => lit_int,
                _ => return None,
            };
            Some((variant_name, variant_value))
        });

        // And create an entry in the enum variants map for each of them
        for (variant_name, variant_value) in enum_variants {
            self.cc_command_enum_variants
                .insert(format!("{}::{}", enum_name, variant_name), variant_value);
        }
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if i.trait_.is_none() {
            return;
        }
        let (_, trait_path, _) = &i.trait_.as_ref().unwrap();
        let trait_name = trait_path.get_ident().unwrap();
        if trait_name != "CCId" {
            return;
        }

        let cc_name = match i.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path.path.get_ident().unwrap(),
            _ => return,
        };

        let cc_id_fn = match try_get_impl_fn(i, "cc_id") {
            Some(f) => f,
            None => return,
        };

        let cc_command_fn = match try_get_impl_fn(i, "cc_command") {
            Some(f) => f,
            None => return,
        };

        let cc_id = match try_get_single_value_from_fn(cc_id_fn) {
            Some(p) => p,
            _ => return,
        };

        let cc_command = match try_get_single_value_from_fn(cc_command_fn) {
            // Map `Some(Command::Variant as _)` to `Some(Ok(Command::Variant))`
            Some(p) => {
                match p {
                    // If the method returns `None`, we return `None`
                    Expr::Path(ExprPath { path, .. }) => {
                        let ident = path.get_ident();
                        if ident.is_none() {
                            return;
                        }
                        let ident = ident.unwrap().to_string();
                        if ident != "None" {
                            return;
                        }
                        None
                    }
                    // If the method returns `Some(Command::Variant as _)` where Command::Variant == 0x01, we return `Some(0x01)`
                    Expr::Call(ExprCall { func, args, .. }) => {
                        let fn_name = match func.as_ref() {
                            Expr::Path(ExprPath { path, .. }) => {
                                path.get_ident().unwrap().to_string()
                            }
                            _ => return,
                        };
                        if fn_name != "Some" {
                            return;
                        }

                        let variant = match args.first().unwrap() {
                            Expr::Cast(ExprCast { expr, .. }) => expr.as_ref(),
                            _ => return,
                        };

                        let variant_val = self
                            .cc_command_enum_variants
                            // For some reason, this formats "Command::Variant" as "Command :: Variant"
                            // so we get rid of the spaces before lookup
                            .get(&variant.to_token_stream().to_string().replace(' ', ""));

                        variant_val.copied()
                    }
                    _ => return,
                }
            }
            // Map `None` to an actual `None`
            _ => return,
        };

        self.ccs.push(CCInfo {
            cc_name,
            cc_id,
            cc_command,
        })
    }
}
