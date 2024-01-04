use syn::visit::Visit;
use syn::{Expr, Ident};

use crate::util::{try_get_impl_fn, try_get_single_value_from_fn};

pub(crate) struct CCAPIInfo<'ast> {
    pub api_name: &'ast Ident,
    pub cc_id: &'ast Expr,
    pub cc_version: &'ast Expr,
}

pub(crate) struct CCAPIInfoExtractor<'ast> {
    pub interview: Option<CCAPIInfo<'ast>>,
}

impl<'ast> Visit<'ast> for CCAPIInfoExtractor<'ast> {
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if i.trait_.is_none() {
            return;
        }
        let (_, trait_path, _) = &i.trait_.as_ref().expect("trait_ should be Some");
        let trait_name = trait_path
            .get_ident()
            .unwrap_or_else(|| &trait_path.segments.first().unwrap().ident);
        if trait_name != "CCAPI" {
            return;
        }

        let api_name = match i.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path
                .path
                .get_ident()
                .unwrap_or_else(|| &type_path.path.segments.first().unwrap().ident),
            _ => return,
        };

        let cc_id_fn = match try_get_impl_fn(i, "cc_id") {
            Some(f) => f,
            None => return,
        };

        let cc_id = match try_get_single_value_from_fn(cc_id_fn) {
            Some(p) => p,
            _ => return,
        };

        let cc_version_fn = match try_get_impl_fn(i, "cc_version") {
            Some(f) => f,
            None => return,
        };

        let cc_version = match try_get_single_value_from_fn(cc_version_fn) {
            Some(p) => p,
            _ => return,
        };

        self.interview = Some(CCAPIInfo {
            api_name,
            cc_id,
            cc_version,
        });
    }
}
