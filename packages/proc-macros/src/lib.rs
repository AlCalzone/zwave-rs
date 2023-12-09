#![feature(proc_macro_diagnostic)]

use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;
use syn::visit::{self, Visit};
use syn::{Expr, ExprPath, File, Ident};
use syn::{ImplItemFn, ItemImpl};
use walkdir::WalkDir;

#[proc_macro]
pub fn impl_command_enum(input: TokenStream) -> TokenStream {
    let _ = input;

    // Figure out which files to look at
    let mut dirname = input.to_string();
    if !dirname.starts_with('"') || !dirname.ends_with('"') {
        panic!("Expected the directory for command implementations to be a string literal");
    }
    dirname.pop();
    dirname.remove(0);

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
    let commands: Vec<CommandInfo> = asts
        .iter()
        .flat_map(|ast| {
            let mut extractor = CommandInfoExtractor {
                commands: Vec::new(),
            };
            visit::visit_file(&mut extractor, ast);
            extractor.commands
        })
        .collect();

    let enum_variants = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! { #command_name(#command_name) }
    });

    let serializable_match_arms = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! {
            Self::#command_name(c) => c.serialize()(out)
        }
    });

    let vec_conversion_impls = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! {
            impl_vec_serializing_for!(#command_name);
            impl_vec_parsing_with_context_for!(#command_name, CommandParseContext);
        }
    });

    let impl_try_from_command_raw_match_arms = commands.iter().map(|c| {
        let command_name = c.command_name;
        let command_type = c.command_type;
        let function_type = c.function_type;
        let origin = c.origin;
        quote! {
            (#command_type, #function_type, #origin) => {
                Ok(Self::#command_name(#command_name::try_from((raw.payload.as_slice(), ctx))?))
            }
        }
    });

    let command_raw_serial_frame_conversions = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! {
            impl TryInto<CommandRaw> for #command_name {
                type Error = EncodingError;

                fn try_into(self) -> std::result::Result<CommandRaw, Self::Error> {
                    let cmd: Command = self.into();
                    cmd.try_into()
                }
            }

            impl From<#command_name> for SerialFrame {
                fn from(val: #command_name) -> Self {
                    SerialFrame::Command(val.into())
                }
            }
        }
    });

    let tokens = quote! {
        // Define the command enum with all possible variants.
        // Calls to the command enum will be dispatched to the corresponding variant.
        #[enum_dispatch]
        #[derive(Debug, Clone, PartialEq)]
        pub enum Command {
            NotImplemented(NotImplemented),
            #( #enum_variants ),*
        }

        // Delegate Serialization to the corresponding variant
        impl Serializable for Command {
            fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
                move |out| match self {
                    Self::NotImplemented(c) => cookie_factory::combinator::slice(&c.payload)(out),
                    #( #serializable_match_arms ),*
                }
            }
        }

        // Implement the default TryFrom<&[u8]>/TryInto<Vec<u8>> conversions for each variant
        #( #vec_conversion_impls )*

        // Implement shortcuts from each variant to CommandRaw / SerialFrame
        #( #command_raw_serial_frame_conversions )*

        impl Command {
            // Implement conversion from a raw command to the correct variant
            pub fn try_from_raw(raw: CommandRaw, ctx: CommandParseContext) -> std::result::Result<Self, EncodingError> {
                let command_type = raw.command_type;
                let function_type = raw.function_type;
                // We parse commands that are sent by the controller
                let expected_origin = MessageOrigin::Controller;

                // ...and hope that Rust optimizes the match arms with origin Host away
                match (command_type, function_type, expected_origin) {
                    #( #impl_try_from_command_raw_match_arms ),*
                    _ => Ok(Self::NotImplemented(NotImplemented {
                        command_type,
                        function_type,
                        payload: raw.payload,
                    })),
                }
            }
        }
    };

    TokenStream::from(tokens)
}

struct CommandInfo<'ast> {
    pub command_name: &'ast Ident,
    pub command_type: &'ast ExprPath,
    pub function_type: &'ast ExprPath,
    pub origin: &'ast ExprPath,
}

struct CommandInfoExtractor<'ast> {
    commands: Vec<CommandInfo<'ast>>,
}

impl<'ast> Visit<'ast> for CommandInfoExtractor<'ast> {
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if i.trait_.is_none() {
            return;
        }
        let (_, trait_path, _) = &i.trait_.as_ref().unwrap();
        let trait_name = trait_path.get_ident().unwrap();
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

fn try_get_impl_fn<'ast>(i: &'ast ItemImpl, method_name: &str) -> Option<&'ast ImplItemFn> {
    i.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(method) if method.sig.ident == method_name => Some(method),
        _ => None,
    })
}

fn try_get_single_value_from_fn(fun: &ImplItemFn) -> Option<&ExprPath> {
    let stmts = &fun.block.stmts;
    if stmts.len() != 1 {
        return None;
    }

    match stmts.first() {
        Some(syn::Stmt::Expr(Expr::Path(p), _)) => Some(p),
        _ => None,
    }
}
