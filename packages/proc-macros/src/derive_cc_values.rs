use quote::quote;
use syn::{parse::Error, spanned::Spanned};

pub(crate) fn impl_derive_cc_values(
    ast: &syn::DeriveInput,
) -> Result<proc_macro::TokenStream, Error> {
    // Check if we have a struct
    let data = match &ast.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) => {
            return Err(Error::new(
                ast.span(),
                "#[derive(CCValues)] is not supported for enums",
            ))
        }
        syn::Data::Union(_) => {
            return Err(Error::new(
                ast.span(),
                "#[derive(CCValues)] is not supported for unions",
            ))
        }
    };

    // Check if it has named fields
    let fields = match &data.fields {
        syn::Fields::Named(fields) => fields.named.iter(),
        syn::Fields::Unnamed(_) => {
            return Err(Error::new(
                ast.span(),
                "#[derive(CCValues)] is not supported for tuple structs",
            ))
        }
        syn::Fields::Unit => {
            return Err(Error::new(
                ast.span(),
                "#[derive(CCValues)] is not supported for unit structs",
            ))
        }
    };

    // Find the fields with a #[cc_value(value_name)] attribute
    let fields = fields.filter_map(|f| {
        let attr = f.attrs.iter().find(|a| a.path().is_ident("cc_value"));
        attr.map(|a| (f, a))
    });

    let values: Vec<_> = fields
        .map(|(f, a)| {
            let value_name = match a.parse_args::<syn::Path>() {
                Ok(path) => path,
                _ => {
                    return Err(Error::new(
                        a.span(),
                        "Expected a path to a CC value definition",
                    ))
                }
            };
            let field_name = f.ident.as_ref().unwrap();

            let ty = match &f.ty {
                syn::Type::Path(p) => p,
                _ => {
                    return Err(Error::new(
                        f.span(),
                        "Unsupported type for #[derive(CCValues)]",
                    ))
                }
            };

            // Depending on the type, we need to do different things
            match ty.path.segments.first() {
                Some(first) if first.ident == "Option" => {
                    // Only return Option-typed values if they are not None
                    Ok(quote! {
                        if let Some(#field_name) = self.#field_name {
                            ret.push((
                                #value_name().id,
                                CacheValue::from(#field_name)
                            ));
                        }
                    })
                }
                _ => {
                    // Return values with other types as-is
                    Ok(quote! {
                        ret.push((
                            #value_name().id,
                            CacheValue::from(self.#field_name)
                        ));
                    })
                }
            }
        })
        .collect::<Result<_, _>>()?;

    let name = &ast.ident;

    if values.is_empty() {
        // No values, return an empty impl
        return Ok(quote!(impl CCValues for #name {}).into());
    }

    Ok(quote! {
        impl CCValues for #name {
            fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
                let mut ret = Vec::new();

                #( #values )*

                ret
            }
        }
    }
    .into())
}
