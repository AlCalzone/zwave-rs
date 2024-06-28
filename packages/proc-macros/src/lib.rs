#![feature(proc_macro_diagnostic)]

use std::collections::HashMap;

use derive_cc_values::impl_derive_cc_values;
use derive_try_from_repr::try_from_repr_for_enum;
use impl_cc_apis::CCAPIInfoExtractor;
use impl_cc_enum::{CCInfo, CCInfoExtractor};
use impl_command_enum::{CommandInfo, CommandInfoExtractor};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, visit, DeriveInput};
use util::{parse_dirname_from_macro_input, parse_files_in_dir};

mod derive_cc_values;
mod derive_try_from_repr;
mod impl_cc_apis;
mod impl_cc_enum;
mod impl_command_enum;
mod util;

#[proc_macro]
pub fn impl_command_enum(input: TokenStream) -> TokenStream {
    // Figure out which files to look at
    let dirname = parse_dirname_from_macro_input(input);
    let files = parse_files_in_dir(&dirname);

    let commands: Vec<CommandInfo> = files
        .iter()
        .flat_map(|(_, ast)| {
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
            Self::#command_name(c) => c.serialize(output, ctx)
        }
    });

    let impl_try_from_command_raw_match_arms = commands.iter().map(|c| {
        let command_name = c.command_name;
        let command_type = c.command_type;
        let function_type = c.function_type;
        let origin = c.origin;
        quote! {
            (#command_type, #function_type, #origin) => {
                #command_name::parse(&mut payload, ctx).map(Self::#command_name)
            }
        }
    });

    let command_raw_serial_frame_conversions = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! {
            impl AsCommandRaw for #command_name {
                fn as_raw(&self, ctx: &CommandEncodingContext) -> CommandRaw {
                    CommandRaw {
                        command_type: self.command_type(),
                        function_type: self.function_type(),
                        payload: self.as_bytes(&ctx),
                        checksum: 0, // placeholder
                    }
                }
            }
        }
    });

    let impl_tologpayload_match_arms = commands.iter().map(|c| {
        let command_name = c.command_name;
        quote! {
            Self::#command_name(c) => c.to_log_payload(),
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
        impl SerializableWith<&CommandEncodingContext> for Command {
            fn serialize(&self, output: &mut bytes::BytesMut, ctx: &CommandEncodingContext) {
                use zwave_core::serialize::bytes::slice;
                match self {
                    Self::NotImplemented(c) => slice(&c.payload).serialize(output),
                    #( #serializable_match_arms ),*
                }
            }
        }

        // Implement shortcuts from each variant to CommandRaw / SerialFrame
        #( #command_raw_serial_frame_conversions )*

        impl Command {
            // Implement conversion from a raw command to the correct variant
            pub fn try_from_raw(raw: CommandRaw, ctx: &mut CommandParsingContext) -> zwave_core::parse::ParseResult<Self> {
                let command_type = raw.command_type;
                let function_type = raw.function_type;
                let mut payload = raw.payload;
                // We parse commands that are sent by the controller
                let expected_origin = MessageOrigin::Controller;

                // ...and hope that Rust optimizes the match arms with origin Host away
                let ret = match (command_type, function_type, expected_origin) {
                    #( #impl_try_from_command_raw_match_arms ),*
                    _ => Err(zwave_core::parse::ParseError::not_implemented("Unknown combination of command_type, function_type and origin")),
                };

                if let Err(ref e) = ret {
                    if let Some(zwave_core::parse::ErrorContext::NotImplemented(_)) = e.context() {
                        // If we don't know how to parse the command, we return the raw command
                        return Ok(Self::NotImplemented(NotImplemented {
                            command_type,
                            function_type,
                            payload,
                        }));
                    }
                }
                ret
            }
        }

        // We cannot use #[enum_dispatch] for ToLogPayload, since it is in another crate,
        // so we have to implement it here "manually"
        impl ToLogPayload for Command {
            fn to_log_payload(&self) -> LogPayload {
                match self {
                    Self::NotImplemented(c) => c.to_log_payload(),
                    #( #impl_tologpayload_match_arms )*
                }
            }
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro]
pub fn impl_cc_enum(input: TokenStream) -> TokenStream {
    // Figure out which files to look at
    let dirname = parse_dirname_from_macro_input(input);
    let files = parse_files_in_dir(&dirname);

    let ccs: Vec<CCInfo> = files
        .iter()
        .flat_map(|(_, ast)| {
            let mut extractor = CCInfoExtractor {
                ccs: Vec::new(),
                cc_command_enum_variants: HashMap::new(),
            };
            visit::visit_file(&mut extractor, ast);
            extractor.ccs
        })
        .collect();

    let submodule_imports = files.iter().map(|(file, _)| {
        let module = format_ident!("{}", file);
        quote! {
            // Expose each CC separately
            pub mod #module;
            // but also make the all available via the commandclass:: namespace
            pub use #module::*;
            // submodule!(#module);
        }
    });

    let enum_variants = ccs.iter().map(|c| {
        let cc_name = c.cc_name;
        quote! { #cc_name(#cc_name) }
    });

    let serializable_match_arms = ccs.iter().map(|c| {
        let cc_name = c.cc_name;
        quote! {
            Self::#cc_name(c) => c.serialize(output, ctx)
        }
    });

    let impl_tologpayload_match_arms = ccs.iter().map(|c| {
        let cc_name = c.cc_name;
        quote! {
            Self::#cc_name(c) => {
                LogPayloadText::new("")
                    .with_tag(stringify!(#cc_name))
                    .with_nested(c.to_log_payload())
                    .into()
            }
        }
    });

    let impl_try_from_cc_raw_match_arms = ccs.iter().map(|c| {
        let cc_name = c.cc_name;
        let cc_id = c.cc_id;
        let cc_command = c.cc_command;
        if let Some(cc_command) = cc_command {
            quote! {
                (#cc_id, Some(#cc_command)) => {
                    #cc_name::parse(&mut payload, ctx).map(Self::#cc_name)
                }
            }
        } else {
            quote! {
                (#cc_id, None) => {
                    #cc_name::parse(&mut payload, ctx).map(Self::#cc_name)
                }
            }
        }
    });

    let tokens = quote! {
        // Import all CC modules, so we don't have to do it manually
        #(#submodule_imports)*

        // Define the command enum with all possible variants.
        // Calls to the command enum will be dispatched to the corresponding variant.
        #[enum_dispatch]
        #[derive(Debug, Clone, PartialEq)]
        pub enum CC {
            NotImplemented(NotImplemented),
            #( #enum_variants ),*
        }

        // Delegate Serialization to the corresponding variant
        impl SerializableWith<&CCEncodingContext> for CC {
            fn serialize(&self, output: &mut bytes::BytesMut, ctx: &CCEncodingContext) {
                use zwave_core::serialize::bytes::slice;
                match self {
                    Self::NotImplemented(c) => slice(&c.payload).serialize(output),
                    #( #serializable_match_arms ),*
                }
            }
        }

        impl CC {
            // Implement conversion from a raw CC to the correct variant
            pub fn try_from_raw(raw: CCRaw, ctx: &mut CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
                let cc_id = raw.cc_id;
                let cc_command = raw.cc_command;
                let mut payload = raw.payload;

                let ret = match (cc_id, cc_command) {
                    #( #impl_try_from_cc_raw_match_arms ),*
                    _ => Err(zwave_core::parse::ParseError::not_implemented("Unknown combination of cc_id and cc_command")),
                };

                if let Err(ref e) = ret {
                    if let Some(zwave_core::parse::ErrorContext::NotImplemented(_)) = e.context() {
                        // If we don't know how to parse the CC, we return the raw CC
                        return Ok(Self::NotImplemented(NotImplemented {
                            cc_id,
                            cc_command,
                            payload,
                        }))
                    }
                }
                ret
            }

            pub fn as_raw(&self, ctx: &CCEncodingContext) -> CCRaw {
                CCRaw {
                    cc_id: self.cc_id(),
                    cc_command: self.cc_command(),
                    payload: self.as_bytes(ctx),
                }
            }
        }

        // We cannot use #[enum_dispatch] for ToLogPayload, since it is in another crate,
        // so we have to implement it here "manually"
        impl ToLogPayload for CC {
            fn to_log_payload(&self) -> LogPayload {
                match self {
                    Self::NotImplemented(c) => {
                        LogPayloadText::new("")
                            .with_tag("NotImplemented")
                            .with_nested(c.to_log_payload())
                            .into()
                    },
                    #( #impl_tologpayload_match_arms )*
                }
            }
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro]
pub fn impl_cc_apis(input: TokenStream) -> TokenStream {
    // Figure out which files to look at
    let dirname = parse_dirname_from_macro_input(input);
    let files = parse_files_in_dir(&dirname);

    let ccs: Vec<_> = files
        .iter()
        .filter_map(|(file, ast)| {
            let mut extractor = CCAPIInfoExtractor { interview: None };
            visit::visit_file(&mut extractor, ast);
            extractor.interview.map(|interview| (file, interview))
        })
        .collect();

    let submodules = files.iter().map(|(file, _)| {
        let module = format_ident!("{}", file);
        quote! {
            mod #module;
        }
    });

    let interview_depends_on_match_arms = ccs.iter().map(|(m, c)| {
        let module = format_ident!("{}", m);
        let cc_id = c.cc_id;
        quote! {
            #cc_id => Some(CCAPIs::new(endpoint).#module().interview_depends_on()),
        }
    });

    let interview_match_arms = ccs.iter().map(|(m, c)| {
        let module = format_ident!("{}", m);
        let cc_id = c.cc_id;
        quote! {
            #cc_id => CCAPIs::new(endpoint).#module().interview().await,
        }
    });

    let implemented_version_match_arms = ccs.iter().map(|(_, c)| {
        let cc_id = c.cc_id;
        let cc_version = c.cc_version;
        quote! {
            #cc_id => Some(#cc_version),
        }
    });

    let cc_apis_methods = ccs.iter().map(|(m, c)| {
        let module = format_ident!("{}", m);
        let api_name = c.api_name;
        quote! {
            pub fn #module(&self) -> #module::#api_name<'a> {
                #module::#api_name::new(self.endpoint)
            }
        }
    });

    let tokens = quote! {
        // Import all interview modules, so we don't have to do it manually
        #(#submodules)*

        pub fn interview_depends_on<'a>(endpoint: &'a dyn EndpointLike<'a>, cc: CommandClasses) -> Option<&'static [CommandClasses]> {
            match cc {
                #( #interview_depends_on_match_arms )*
                _ => None
            }
        }

        pub async fn interview_cc<'a>(endpoint: &'a dyn EndpointLike<'a>, cc: CommandClasses) -> CCAPIResult<()> {
            match cc {
                #( #interview_match_arms )*
                _ => {
                    // No interview procedure
                    Ok(())
                }
            }
        }

        /// Returns the version of the given CC this library implements
        pub fn get_implemented_version(cc: CommandClasses) -> Option<u8> {
            match cc {
                #( #implemented_version_match_arms )*
                _ => None
            }
        }

        pub struct CCAPIs<'a> {
            endpoint: &'a dyn EndpointLike<'a>,
        }
        impl<'a> CCAPIs<'a> {
            pub fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self {
                Self { endpoint }
            }

            #( #cc_apis_methods )*
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro_derive(TryFromRepr)]
pub fn derive_try_from_repr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = match &input.data {
        syn::Data::Enum(data) => {
            try_from_repr_for_enum(&input, data.variants.iter().cloned().collect())
        }
        syn::Data::Struct(_) => panic!("#[derive(TryFromRepr)] not supported for structs"),
        syn::Data::Union(_) => panic!("#[derive(TryFromRepr)] not supported for unions"),
    };
    result
}

#[proc_macro_derive(CCValues, attributes(cc_value))]
pub fn derive_cc_values(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_derive_cc_values(&input) {
        Ok(output) => output,
        Err(error) => error.to_compile_error().into(),
    }
}
