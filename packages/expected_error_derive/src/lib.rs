use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, Variant, parenthesized, parse_macro_input};

#[proc_macro_derive(ExpectedError, attributes(ee))]
pub fn derive_expected_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the enum name
    let name = &input.ident;

    // Check if input is an enum
    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        panic!("ExpectedError can only be derived for enums");
    };

    // Generate match arms for the status and msg methods
    let status_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let status_code = extract_status_code(variant);

        match &variant.fields {
            Fields::Unit => {
                quote! { Self::#variant_name => #status_code }
            }
            Fields::Unnamed(_) => {
                quote! { Self::#variant_name(..) => #status_code }
            }
            Fields::Named(_) => {
                quote! { Self::#variant_name { .. } => #status_code }
            }
        }
    });

    // Generate implementation
    let expanded = quote! {
        impl expected_error::ExpectedError for #name {
            fn status(&self) -> expected_error::StatusCode {
                match self {
                    #(#status_arms),*
                }
            }

            fn msg(&self) -> std::borrow::Cow<str> {
                self.to_string().into()
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_status_code(variant: &Variant) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;

    for attr in &variant.attrs {
        let mut expr = None;
        if attr.path().is_ident("ee") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("status") {
                    let content;
                    parenthesized!(content in meta.input);
                    let code: Expr = content.parse()?;
                    expr = Some(quote! { #code });
                    return Ok(());
                }

                Err(meta.error("unrecognized ee"))
            });
        }

        if let Some(expr) = expr {
            return expr;
        }
    }

    // Error if status not specified
    panic!(
        "Missing #[ee(status = ...)] attribute for variant `{}`",
        variant_name
    );
}
