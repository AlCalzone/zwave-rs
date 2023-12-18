use crate::util::{try_get_impl_fn, try_get_single_value_from_fn};
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprCast, ExprPath, Ident};

pub(crate) struct CCInfo<'ast> {
    pub cc_name: &'ast Ident,
    pub cc_id: &'ast Expr,
    pub cc_command: Option<&'ast Expr>,
}

pub(crate) struct CCInfoExtractor<'ast> {
    pub ccs: Vec<CCInfo<'ast>>,
}

impl<'ast> Visit<'ast> for CCInfoExtractor<'ast> {
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
                    // If the method returns `Some(Command::Variant as _)`, we return `Some(Command::Variant)`
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

                        Some(variant)
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
