//! `#[derive(Args)]` for winnow-args.
//!
//! The generated `parse_argv` is one loop: lex an item with `arg`, `match` it
//! against every flag the struct declares, store into a local per field, and
//! build the struct once the line is exhausted. Long names are matched as byte
//! string patterns and shorts as `char` patterns, so the lookup is whatever
//! rustc makes of a `match`, not a walk over a list of parsers.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Ident, LitByteStr, LitChar, LitStr, PathArguments,
    Type, parse_macro_input, spanned::Spanned as _,
};

#[proc_macro_derive(Args, attributes(arg))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

enum Kind {
    /// `bool`: present or not.
    Switch,
    /// `Option<T>`: one value, may be absent.
    Optional(Type),
    /// `T`: one value, required.
    Required(Type),
}

struct Field {
    ident: Ident,
    kind: Kind,
    short: Option<char>,
    long: Option<String>,
}

impl Field {
    fn display(&self) -> String {
        match (&self.long, self.short) {
            (Some(l), _) => format!("--{l}"),
            (None, Some(c)) => format!("-{c}"),
            (None, None) => unreachable!("every field has a name"),
        }
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Args` can only be derived for structs",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "`Args` needs a struct with named fields",
        ));
    };
    let fields = named
        .named
        .iter()
        .map(field)
        .collect::<syn::Result<Vec<_>>>()?;
    check_duplicates(&fields)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let slots = fields.iter().map(|f| {
        let ident = slot(&f.ident);
        match &f.kind {
            Kind::Switch => quote!(let mut #ident: bool = false;),
            Kind::Optional(ty) | Kind::Required(ty) => {
                quote!(let mut #ident: ::core::option::Option<#ty> = ::core::option::Option::None;)
            }
        }
    });

    // What storing one occurrence looks like; the same for both spellings.
    let store = |f: &Field| {
        let ident = slot(&f.ident);
        match &f.kind {
            Kind::Switch => quote! {
                __wa::no_value(&__arg, __offset)?;
                #ident = true;
            },
            Kind::Optional(ty) | Kind::Required(ty) => quote! {
                #ident = ::core::option::Option::Some(
                    __wa::value_as::<#ty>(__input, &__arg, __offset)?
                );
            },
        }
    };

    let long_arms = fields.iter().filter_map(|f| {
        let long = f.long.as_ref()?;
        let pattern = LitByteStr::new(long.as_bytes(), Span::call_site());
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });
    let short_arms = fields.iter().filter_map(|f| {
        let pattern = LitChar::new(f.short?, Span::call_site());
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });

    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        match &f.kind {
            Kind::Switch | Kind::Optional(_) => quote!(#ident: #slot),
            Kind::Required(_) => {
                let display = LitStr::new(&f.display(), Span::call_site());
                quote! {
                    #ident: match #slot {
                        ::core::option::Option::Some(v) => v,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(
                                __wa::Error::missing_required(__input.offset(), #display)
                            );
                        }
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            fn parse_argv(
                __input: &mut ::winnow_args::Argv<'_>,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                #(#slots)*
                while !__input.is_empty() {
                    let __offset = __input.offset();
                    let __arg = __wa::arg(__input)?;
                    match __arg {
                        __wa::Arg::Long { name: __name, .. } => match &**__name {
                            #(#long_arms)*
                            _ => return ::core::result::Result::Err(__wa::unexpected(&__arg, __offset)),
                        },
                        __wa::Arg::Short(__c) => match __c {
                            #(#short_arms)*
                            _ => return ::core::result::Result::Err(__wa::unexpected(&__arg, __offset)),
                        },
                        __wa::Arg::Separator => {}
                        __wa::Arg::Word(_) => {
                            return ::core::result::Result::Err(__wa::unexpected(&__arg, __offset));
                        }
                    }
                }
                ::core::result::Result::Ok(Self { #(#build),* })
            }
        }
    })
}

fn slot(ident: &Ident) -> Ident {
    format_ident!("__slot_{}", ident)
}

fn field(f: &syn::Field) -> syn::Result<Field> {
    let ident = f.ident.clone().expect("named field");
    let kind = kind(&f.ty);

    let mut short = None;
    let mut long = None;
    let mut named = false;
    for attr in f.attrs.iter().filter(|a| a.path().is_ident("arg")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("short") {
                named = true;
                short = Some(if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitChar>()?.value()
                } else {
                    let first = ident.to_string().trim_start_matches("r#").chars().next();
                    first.ok_or_else(|| meta.error("cannot infer a short name"))?
                });
                Ok(())
            } else if meta.path.is_ident("long") {
                named = true;
                long = Some(if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitStr>()?.value()
                } else {
                    kebab(&ident)
                });
                Ok(())
            } else {
                Err(meta.error("expected `short` or `long`"))
            }
        })?;
    }
    // Like bpaf: a field with no names is `--field-name`.
    if !named {
        long = Some(kebab(&ident));
    }
    if let Some(l) = &long {
        if l.is_empty() || l.starts_with('-') || l.contains('=') {
            return Err(syn::Error::new(
                f.span(),
                format!("`{l}` is not a usable long name"),
            ));
        }
    }
    if let Some(c) = short {
        if c == '-' || c == '=' {
            return Err(syn::Error::new(
                f.span(),
                format!("`{c}` is not a usable short name"),
            ));
        }
    }

    Ok(Field {
        ident,
        kind,
        short,
        long,
    })
}

fn check_duplicates(fields: &[Field]) -> syn::Result<()> {
    for (i, a) in fields.iter().enumerate() {
        for b in &fields[..i] {
            let clash = match (a.short, b.short) {
                (Some(x), Some(y)) if x == y => Some(format!("-{x}")),
                _ => None,
            }
            .or_else(|| match (&a.long, &b.long) {
                (Some(x), Some(y)) if x == y => Some(format!("--{x}")),
                _ => None,
            });
            if let Some(name) = clash {
                return Err(syn::Error::new(
                    a.ident.span(),
                    format!("`{name}` is already used by `{}`", b.ident),
                ));
            }
        }
    }
    Ok(())
}

fn kind(ty: &Type) -> Kind {
    if let Type::Path(path) = ty {
        if path.qself.is_none() {
            let last = path.path.segments.last().expect("non-empty path");
            if last.ident == "bool" && last.arguments.is_none() {
                return Kind::Switch;
            }
            if last.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &last.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return Kind::Optional(inner.clone());
                    }
                }
            }
        }
    }
    Kind::Required(ty.clone())
}

fn kebab(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").replace('_', "-")
}
