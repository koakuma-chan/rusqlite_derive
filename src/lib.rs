use proc_macro::TokenStream;

use quote::quote;

use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

struct RusqliteAttr {
    try_from: Option<Type>,
}

fn parse_rusqlite_attr(input: &DeriveInput) -> syn::Result<RusqliteAttr> {
    let mut attr = RusqliteAttr { try_from: None };

    for attr_item in &input.attrs {
        if attr_item.path().is_ident("rusqlite") {
            attr_item.parse_nested_meta(|meta| {
                if meta.path.is_ident("try_from") {
                    let value: Type = meta.value()?.parse()?;

                    attr.try_from = Some(value);

                    Ok(())
                } else {
                    Err(meta.error("unknown rusqlite attribute"))
                }
            })?;
        }
    }

    Ok(attr)
}

#[proc_macro_derive(FromSql, attributes(rusqlite))]
pub fn from_sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let rusqlite_attr = match parse_rusqlite_attr(&input) {
        Ok(attr) => attr,

        Err(e) => {
            return e
                //
                .to_compile_error()
                //
                .into();
        }
    };

    let struct_name = &input.ident;

    if let Some(try_from_type) = rusqlite_attr.try_from {
        return quote! {
            impl ::rusqlite::types::FromSql for #struct_name {
                fn column_result(value: ::rusqlite::types::ValueRef<'_>) -> ::rusqlite::types::FromSqlResult<Self> {
                    <#try_from_type as ::rusqlite::types::FromSql>::column_result(value)
                        .and_then(|v| {
                            <Self as ::core::convert::TryFrom<#try_from_type>>::try_from(v)
                                .map_err(|e| ::rusqlite::types::FromSqlError::Other(::std::boxed::Box::new(e)))
                        })
                }
            }
        }
        //
        .into();
    }

    let inner_type = match inner_type(&input) {
        Ok(inner_type) => inner_type,

        Err(e) => {
            return e
                //
                .to_compile_error()
                //
                .into();
        }
    };

    quote! {
        impl ::rusqlite::types::FromSql for #struct_name {
            fn column_result(value: ::rusqlite::types::ValueRef<'_>) -> ::rusqlite::types::FromSqlResult<Self> {
                <#inner_type as ::rusqlite::types::FromSql>::column_result(value).map(Self)
            }
        }
    }
    //
    .into()
}

#[proc_macro_derive(ToSql)]
pub fn to_sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let inner_type = match inner_type(&input) {
        Ok(inner_type) => inner_type,

        Err(e) => {
            return e
                //
                .to_compile_error()
                //
                .into();
        }
    };

    let struct_name = &input.ident;

    quote! {
        impl ::rusqlite::types::ToSql for #struct_name {
            fn to_sql(&self) -> ::rusqlite::Result<::rusqlite::types::ToSqlOutput<'_>> {
                <#inner_type as ::rusqlite::types::ToSql>::to_sql(&self.0)
            }
        }
    }
    //
    .into()
}

fn inner_type(input: &DeriveInput) -> syn::Result<&Type> {
    let inner_type = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                    &fields.unnamed.first().unwrap().ty
                }

                _ => {
                    return Err(
                        //
                        syn::Error::new_spanned(
                            //
                            &input.ident,
                            //
                            "Expected a tuple struct with exactly one field",
                        ),
                    );
                }
            }
        }
        _ => {
            return Err(
                //
                syn::Error::new_spanned(
                    //
                    &input.ident,
                    //
                    "Expected a tuple struct with exactly one field",
                ),
            );
        }
    };

    Ok(inner_type)
}
