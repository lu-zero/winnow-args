//! `#[derive(Args)]` for winnow-args.
//!
//! The generated `parse_argv` is one loop: lex an item with `arg`, `match` it
//! against every flag the struct declares, store into a local per field, and
//! build the struct once the line is exhausted. Long names are matched as byte
//! string patterns and shorts as `char` patterns, so the lookup is whatever
//! rustc makes of a `match`, not a walk over a list of parsers. Words fill the
//! positional fields in declaration order, tracked by one counter.

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
    /// `Vec<T>`: every remaining word; positionals only.
    Many(Type),
}

enum Role {
    Flag {
        short: Option<char>,
        long: Option<String>,
    },
    Positional {
        name: String,
    },
}

struct Field {
    ident: Ident,
    kind: Kind,
    role: Role,
}

impl Field {
    /// How errors name the field: its long flag, else its short one, else its value name.
    fn display(&self) -> String {
        match &self.role {
            Role::Flag { long: Some(l), .. } => format!("--{l}"),
            Role::Flag { short: Some(c), .. } => format!("-{c}"),
            Role::Flag { .. } => unreachable!("every flag has a name"),
            Role::Positional { name } => name.clone(),
        }
    }

    fn short(&self) -> Option<char> {
        match self.role {
            Role::Flag { short, .. } => short,
            Role::Positional { .. } => None,
        }
    }

    fn long(&self) -> Option<&str> {
        match &self.role {
            Role::Flag { long, .. } => long.as_deref(),
            Role::Positional { .. } => None,
        }
    }

    fn is_positional(&self) -> bool {
        matches!(self.role, Role::Positional { .. })
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
    let positionals: Vec<&Field> = fields.iter().filter(|f| f.is_positional()).collect();
    check_positional_order(&positionals)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let slots = fields.iter().map(|f| {
        let ident = slot(&f.ident);
        match &f.kind {
            Kind::Switch => quote!(let mut #ident: bool = false;),
            Kind::Optional(ty) | Kind::Required(ty) => {
                quote!(let mut #ident: ::core::option::Option<#ty> = ::core::option::Option::None;)
            }
            Kind::Many(ty) => {
                quote!(let mut #ident: ::std::vec::Vec<#ty> = ::std::vec::Vec::new();)
            }
        }
    });

    // What storing one flag occurrence looks like; the same for both spellings.
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
            Kind::Many(_) => unreachable!("rejected for flags in `field`"),
        }
    };

    let long_arms = fields.iter().filter_map(|f| {
        let pattern = LitByteStr::new(f.long()?.as_bytes(), Span::call_site());
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });
    let short_arms = fields.iter().filter_map(|f| {
        let pattern = LitChar::new(f.short()?, Span::call_site());
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });

    let unexpected = quote!(return ::core::result::Result::Err(__wa::unexpected(&__arg, __offset)));
    let word_arm = if positionals.is_empty() {
        quote!(__wa::Arg::Word(_) => { #unexpected; })
    } else {
        let arms = positionals.iter().enumerate().map(|(i, f)| {
            let ident = slot(&f.ident);
            let display = LitStr::new(&f.display(), Span::call_site());
            match &f.kind {
                Kind::Optional(ty) | Kind::Required(ty) => quote! {
                    #i => {
                        #ident = ::core::option::Option::Some(
                            __wa::positional_as::<#ty>(__word, __offset, #display)?
                        );
                        __position += 1;
                    }
                },
                Kind::Many(ty) => quote! {
                    #i => #ident.push(__wa::positional_as::<#ty>(__word, __offset, #display)?),
                },
                Kind::Switch => unreachable!("rejected for positionals in `field`"),
            }
        });
        quote! {
            __wa::Arg::Word(__word) => match __position {
                #(#arms)*
                _ => { #unexpected; }
            },
        }
    };
    let position = (!positionals.is_empty()).then(|| quote!(let mut __position: usize = 0;));

    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        match &f.kind {
            Kind::Switch | Kind::Optional(_) | Kind::Many(_) => quote!(#ident: #slot),
            Kind::Required(_) => {
                let display = LitStr::new(&f.display(), Span::call_site());
                let error = if f.is_positional() {
                    quote!(missing_argument)
                } else {
                    quote!(missing_required)
                };
                quote! {
                    #ident: match #slot {
                        ::core::option::Option::Some(v) => v,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(
                                __wa::Error::#error(__input.offset(), #display)
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
                #position
                while !__input.is_empty() {
                    let __offset = __input.offset();
                    let __arg = __wa::arg(__input)?;
                    match __arg {
                        __wa::Arg::Long { name: __name, .. } => match &**__name {
                            #(#long_arms)*
                            _ => { #unexpected; }
                        },
                        __wa::Arg::Short(__c) => match __c {
                            #(#short_arms)*
                            _ => { #unexpected; }
                        },
                        __wa::Arg::Separator => {}
                        #word_arm
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
    let bare = ident.to_string().trim_start_matches("r#").to_owned();

    let mut short = None;
    let mut long = None;
    let mut positional = false;
    let mut value_name = None;
    for attr in f.attrs.iter().filter(|a| a.path().is_ident("arg")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("short") {
                short = Some(if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitChar>()?.value()
                } else {
                    bare.chars()
                        .next()
                        .ok_or_else(|| meta.error("cannot infer a short name"))?
                });
            } else if meta.path.is_ident("long") {
                long = Some(if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitStr>()?.value()
                } else {
                    bare.replace('_', "-")
                });
            } else if meta.path.is_ident("positional") {
                positional = true;
            } else if meta.path.is_ident("value_name") {
                value_name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error("expected `short`, `long`, `positional` or `value_name`"));
            }
            Ok(())
        })?;
    }

    let error = |msg: String| Err(syn::Error::new(f.span(), msg));
    let kind = kind(&f.ty, positional);
    let role = if positional {
        if short.is_some() || long.is_some() {
            return error("a positional field has no `short` or `long` name".into());
        }
        if matches!(kind, Kind::Switch) {
            return error("a positional field cannot be `bool`".into());
        }
        Role::Positional {
            name: value_name.unwrap_or_else(|| bare.to_uppercase()),
        }
    } else {
        if matches!(kind, Kind::Required(_)) && is_vec(&f.ty) {
            return error("repeatable flags (`Vec<T>`) are not supported yet".into());
        }
        // Like bpaf: a flag with no names is `--field-name`.
        if short.is_none() && long.is_none() {
            long = Some(bare.replace('_', "-"));
        }
        if let Some(l) = &long {
            if l.is_empty() || l.starts_with('-') || l.contains('=') {
                return error(format!("`{l}` is not a usable long name"));
            }
        }
        if let Some(c) = short {
            if c == '-' || c == '=' {
                return error(format!("`{c}` is not a usable short name"));
            }
        }
        Role::Flag { short, long }
    };

    Ok(Field { ident, kind, role })
}

fn check_duplicates(fields: &[Field]) -> syn::Result<()> {
    for (i, a) in fields.iter().enumerate() {
        for b in &fields[..i] {
            let clash = match (a.short(), b.short()) {
                (Some(x), Some(y)) if x == y => Some(format!("-{x}")),
                _ => None,
            }
            .or_else(|| match (a.long(), b.long()) {
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

/// Required positionals come first, then optional ones, then at most one `Vec`,
/// last: otherwise which word fills which field would depend on lookahead.
fn check_positional_order(positionals: &[&Field]) -> syn::Result<()> {
    let rank = |f: &Field| match f.kind {
        Kind::Required(_) => 0,
        Kind::Optional(_) => 1,
        Kind::Many(_) | Kind::Switch => 2,
    };
    for pair in positionals.windows(2) {
        if rank(pair[1]) < rank(pair[0]) || rank(pair[0]) == 2 {
            return Err(syn::Error::new(
                pair[1].ident.span(),
                "positionals must be required, then optional, then one `Vec` last",
            ));
        }
    }
    Ok(())
}

fn last_segment(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last(),
        _ => None,
    }
}

fn inner(segment: &syn::PathSegment) -> Option<&Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => match args.args.first()? {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        },
        _ => None,
    }
}

fn is_vec(ty: &Type) -> bool {
    last_segment(ty).is_some_and(|s| s.ident == "Vec")
}

fn kind(ty: &Type, positional: bool) -> Kind {
    if let Some(last) = last_segment(ty) {
        if last.ident == "bool" && last.arguments.is_none() {
            return Kind::Switch;
        }
        if let Some(inner) = inner(last) {
            if last.ident == "Option" {
                return Kind::Optional(inner.clone());
            }
            if last.ident == "Vec" && positional {
                return Kind::Many(inner.clone());
            }
        }
    }
    Kind::Required(ty.clone())
}
