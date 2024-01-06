use core::panic;
use quote::quote;

pub(crate) fn try_from_repr_for_enum(
    ast: &syn::DeriveInput,
    variants: Vec<syn::Variant>,
) -> proc_macro::TokenStream {
    if variants.is_empty() {
        panic!("#[derive(TryFromRepr)] cannot be implemented for enums with zero variants");
    }

    try_from_repr(ast, variants)
}

pub(crate) fn try_from_repr(
    ast: &syn::DeriveInput,
    variants: Vec<syn::Variant>,
) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let doc = format!(
        "Generated impl [TryFrom](std::convert::TryFrom) for `{}`.",
        name
    );
    let lint_attrs = collect_parent_lint_attrs(&ast.attrs);
    let lint_attrs = quote![#(#lint_attrs),*];
    let repr = find_repr_attr(&ast.attrs);

    let mut discr = None;
    let match_arms = variants.iter().map(|v| {
        let v_name = &v.ident;

        if let Some((_, syn::Expr::Lit(l))) = &v.discriminant {
            match &l.lit {
                syn::Lit::Int(int_lit) => match int_lit.base10_parse::<u64>() {
                    Ok(d) => discr = Some(d),
                    Err(e) => panic!(
                        "Could not parse Enum variant in #[derive(TryFromRepr)]. Reason: {}",
                        e
                    ),
                },
                _ => panic!("Enum discriminant must be an integer literal"),
            }
        } else {
            discr = Some(discr.map(|d| d + 1).unwrap_or(0));
        }

        let repr_string = repr
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or("usize".to_string());
        let discr_tokens = match repr_string.as_str() {
            "u8" => {
                let discr = discr.unwrap() as u8;
                quote!(#discr)
            }
            "u16" => {
                let discr = discr.unwrap() as u16;
                quote!(#discr)
            }
            "u32" => {
                let discr = discr.unwrap() as u32;
                quote!(#discr)
            }
            "u64" => {
                let discr = discr.unwrap();
                quote!(#discr)
            }
            "usize" => {
                let discr = discr.unwrap() as usize;
                quote!(#discr)
            }
            "i8" => {
                let discr = discr.unwrap() as i8;
                quote!(#discr)
            }
            "i16" => {
                let discr = discr.unwrap() as i16;
                quote!(#discr)
            }
            "i32" => {
                let discr = discr.unwrap() as i32;
                quote!(#discr)
            }
            "i64" => {
                let discr = discr.unwrap() as i64;
                quote!(#discr)
            }
            "isize" => {
                let discr = discr.unwrap() as isize;
                quote!(#discr)
            }
            ty => {
                panic!(
                    "#[derive(TryFromRepr)] does not support enum repr type {:?}",
                    ty
                );
            }
        };

        let result_tokens = if v.fields.is_empty() {
            quote!(Ok(#name::#v_name))
        } else {
            quote!(Err(TryFromReprError::NonPrimitive(n)))
        };

        quote!(#discr_tokens => #result_tokens)
    });
    let match_arms = quote![#(#match_arms),*];

    quote! {
        impl #impl_generics core::convert::TryFrom<#repr> for #name #ty_generics #where_clause {
            type Error = TryFromReprError<#repr>;

            #[doc = #doc]
            #lint_attrs
            fn try_from(n: #repr) -> Result<Self, Self::Error> {
                match n {
                    #match_arms,
                    _ => Err(TryFromReprError::Invalid(n))
                }
            }
        }
    }
    .into()
}

pub(crate) fn collect_parent_lint_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    fn is_lint_ident(path: &syn::Path) -> bool {
        path.is_ident("allow")
            || path.is_ident("deny")
            || path.is_ident("forbid")
            || path.is_ident("warn")
            || path.is_ident("must_use")
    }

    fn is_lint(item: &syn::Meta) -> bool {
        if let syn::Meta::List(ref l) = *item {
            is_lint_ident(&l.path)
        } else {
            false
        }
    }

    fn is_cfg_attr_lint(item: &syn::Meta) -> bool {
        if let syn::Meta::List(ref l) = *item {
            if l.path.is_ident("cfg_attr") {
                let mut has_lint = false;
                let _ = l.parse_nested_meta(|meta| {
                    if is_lint_ident(&meta.path) {
                        has_lint = true;
                    }
                    Ok(())
                });
                return has_lint;
            }
        }
        false
    }

    attrs
        .iter()
        .filter(|&a| is_lint(&a.meta) || is_cfg_attr_lint(&a.meta))
        .cloned()
        .collect()
}

pub(crate) fn find_repr_attr(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    let last_repr: Option<syn::Meta> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("repr") {
                Some(attr.meta.clone())
            } else {
                None
            }
        })
        .last();

    if let Some(repr) = last_repr {
        if let Ok(list) = repr.require_list() {
            let mut result: Option<syn::Ident> = None;
            let _ = list.parse_nested_meta(|meta| {
                result = Some(meta.path.get_ident().cloned().unwrap());
                Ok(())
            });
            return result;
        }
        panic!("failed to parse #[repr(...)] attribute");
    } else {
        None
    }
}
