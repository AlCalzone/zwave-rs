use crate::util::{try_get_impl_fn, try_get_single_value_from_fn};
use syn::visit::Visit;
use syn::{Expr, Ident};

pub(crate) struct CommandInfo<'ast> {
    pub command_name: &'ast Ident,
    pub command_type: &'ast Expr,
    pub function_type: &'ast Expr,
    pub origin: &'ast Expr,
}

pub(crate) struct CommandInfoExtractor<'ast> {
    pub commands: Vec<CommandInfo<'ast>>,
}

impl<'ast> Visit<'ast> for CommandInfoExtractor<'ast> {
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let Some((_, trait_path, _)) = i.trait_.as_ref() else {
            return;
        };
        let Some(trait_name) = trait_path.get_ident() else {
            return;
        };
        if trait_name != "CommandId" {
            return;
        }

        let command_name = match i.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path.path.get_ident().unwrap(),
            _ => return,
        };

        let command_type_fn = match try_get_impl_fn(i, "command_type") {
            Some(f) => f,
            None => return,
        };
        let function_type_fn = match try_get_impl_fn(i, "function_type") {
            Some(f) => f,
            None => return,
        };
        let origin_fn = match try_get_impl_fn(i, "origin") {
            Some(f) => f,
            None => return,
        };

        let command_type = match try_get_single_value_from_fn(command_type_fn) {
            Some(p) => p,
            _ => return,
        };
        let function_type = match try_get_single_value_from_fn(function_type_fn) {
            Some(p) => p,
            _ => return,
        };
        let origin = match try_get_single_value_from_fn(origin_fn) {
            Some(p) => p,
            _ => return,
        };

        self.commands.push(CommandInfo {
            command_name,
            command_type,
            function_type,
            origin,
        })
    }
}
