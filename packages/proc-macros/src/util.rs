use proc_macro::TokenStream;
use std::path::Path;
use syn::{Expr, File};
use syn::{ImplItemFn, ItemImpl};
use walkdir::WalkDir;

pub(crate) fn try_get_impl_fn<'ast>(
    i: &'ast ItemImpl,
    method_name: &str,
) -> Option<&'ast ImplItemFn> {
    i.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(method) if method.sig.ident == method_name => Some(method),
        _ => None,
    })
}

pub(crate) fn try_get_single_value_from_fn(fun: &ImplItemFn) -> Option<&Expr> {
    let stmts = &fun.block.stmts;
    if stmts.len() != 1 {
        return None;
    }

    match stmts.first() {
        Some(syn::Stmt::Expr(e, _)) => Some(e),
        _ => None,
    }
}

pub(crate) fn parse_dirname_from_macro_input(input: TokenStream) -> String {
    let mut dirname = input.to_string();
    if !dirname.starts_with('"') || !dirname.ends_with('"') {
        panic!("Expected the directory for command implementations to be a string literal");
    }
    dirname.pop();
    dirname.remove(0);

    dirname
}

pub(crate) fn parse_files_in_dir(dirname: &str) -> Vec<File> {
    let root_dir = &std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = Path::new(root_dir).join(dirname).canonicalize().unwrap();

    let files = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().file_stem().map(|f| f != "mod").unwrap_or(false))
        .filter(|e| e.path().extension().map(|e| e == "rs").unwrap_or(false))
        .map(|e| e.path().to_owned())
        .map(|e| e.to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    let asts: Vec<File> = files
        .iter()
        .map(|file| {
            let file_content = std::fs::read_to_string(file).unwrap();
            syn::parse_file(&file_content).unwrap()
        })
        .collect();

    asts
}
