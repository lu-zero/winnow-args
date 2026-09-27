//! `#[derive(Args)]` and `#[derive(Subcommand)]` for winnow-args.
//!
//! The generated `parse_argv` is one loop: lex an item with `arg`, `match` it
//! against every flag the struct declares, store into a local per field, and
//! build the struct once the line is exhausted. Long names are matched as byte
//! string patterns and shorts as `char` patterns, so the lookup is whatever
//! rustc makes of a `match`, not a walk over a list of parsers. Words fill the
//! positional fields in declaration order, tracked by one counter, unless the
//! first one names a subcommand: then the subcommand parses the rest.

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

#[proc_macro_derive(Subcommand, attributes(arg))]
pub fn derive_subcommand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_subcommand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// One `match` on the subcommand's name; the variant's `Args` parses the rest.
fn expand_subcommand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Subcommand` can only be derived for enums",
        ));
    };
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let mut patterns = Vec::new();
    for variant in &data.variants {
        let mut name = None;
        let mut alias = Vec::new();
        for attr in variant.attrs.iter().filter(|a| a.path().is_ident("arg")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if meta.path.is_ident("alias") {
                    alias.extend(aliases(&meta)?);
                } else {
                    return Err(meta.error("expected `name` or `alias`"));
                }
                Ok(())
            })?;
        }
        let ident = &variant.ident;
        let spellings: Vec<String> =
            std::iter::once(name.unwrap_or_else(|| kebab_case(&ident.to_string())))
                .chain(alias)
                .collect();
        for name in &spellings {
            if let Some((_, other)) = names.iter().find(|(n, _)| n == name) {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("`{name}` is already used by `{other}`"),
                ));
            }
            names.push((name.clone(), ident));
        }
        let literals = spellings
            .iter()
            .map(|n| LitByteStr::new(n.as_bytes(), Span::call_site()));
        let pattern = quote!(#(#literals)|*);
        let parse = match &variant.fields {
            Fields::Unit => quote!(__wa::finish_with(__input, __globals).map(|()| Self::#ident)),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                match boxed(ty) {
                    Some(inner) => quote! {
                        <#inner as ::winnow_args::Args>::parse_argv_with(__input, __globals)
                            .map(|v| Self::#ident(::std::boxed::Box::new(v)))
                    },
                    None => quote! {
                        <#ty as ::winnow_args::Args>::parse_argv_with(__input, __globals).map(Self::#ident)
                    },
                }
            }
            _ => {
                return Err(syn::Error::new(
                    variant.span(),
                    "a subcommand variant is a unit or holds one `Args` type",
                ));
            }
        };
        arms.push(quote!(#pattern => #parse,));
        patterns.push(pattern);
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::winnow_args::Subcommand for #name #ty_generics #where_clause {
            #[inline]
            fn has(__name: &[u8]) -> bool {
                matches!(__name, #(#patterns)|*)
            }

            fn parse_subcommand(
                __name: &[u8],
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                match __name {
                    #(#arms)*
                    _ => ::core::result::Result::Err(__wa::Error::unexpected_arg(
                        __input.offset(),
                        ::std::string::String::from_utf8_lossy(__name),
                    )),
                }
            }
        }

        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                if __input.is_empty() {
                    return ::core::result::Result::Err(__wa::Error::missing_subcommand(__input.offset()));
                }
                let __arg = __wa::arg(__input)?;
                if let __wa::Arg::Word(__word) = __arg {
                    if <Self as __wa::Subcommand>::has(&**__word.value) {
                        return <Self as __wa::Subcommand>::parse_subcommand(
                            &**__word.value, __input, __globals,
                        );
                    }
                }
                ::core::result::Result::Err(__arg.unexpected())
            }
        }
    })
}

/// `alias = "x"` or `alias("x", "y")`.
fn aliases(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Vec<String>> {
    if meta.input.peek(syn::Token![=]) {
        return Ok(vec![meta.value()?.parse::<LitStr>()?.value()]);
    }
    let content;
    syn::parenthesized!(content in meta.input);
    let names = syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
    Ok(names.iter().map(LitStr::value).collect())
}

/// `DryRun` → `dry-run`.
fn kebab_case(ident: &str) -> String {
    let mut out = String::new();
    for (i, c) in ident.trim_start_matches("r#").chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `T` in `Box<T>`.
fn boxed(ty: &Type) -> Option<&Type> {
    let last = last_segment(ty)?;
    if last.ident == "Box" {
        inner(last)
    } else {
        None
    }
}

enum Kind {
    /// `bool`: present or not.
    Switch,
    /// `#[arg(count)]` on an integer: how many times it was given.
    Count(Type),
    /// `Option<T>`: one value, may be absent.
    Optional(Type),
    /// `T`: one value, required.
    Required(Type),
    /// `Vec<T>`: a repeatable flag's values, or every remaining word.
    Many(Type),
}

enum Role {
    Flag {
        short: Option<char>,
        long: Option<String>,
        /// More long names, matched like `long`.
        aliases: Vec<String>,
        /// Also accepted after a subcommand word, at any depth.
        global: bool,
    },
    Positional {
        name: String,
    },
    Subcommand,
}

struct Field {
    ident: Ident,
    kind: Kind,
    role: Role,
    /// Splits each value of a `Vec` field.
    delimiter: Option<u8>,
}

impl Field {
    /// How errors name the field: its long flag, else its short one, else its value name.
    fn display(&self) -> String {
        match &self.role {
            Role::Flag { long: Some(l), .. } => format!("--{l}"),
            Role::Flag { short: Some(c), .. } => format!("-{c}"),
            Role::Flag { .. } => unreachable!("every flag has a name"),
            Role::Positional { name } => name.clone(),
            Role::Subcommand => "<COMMAND>".to_owned(),
        }
    }

    fn short(&self) -> Option<char> {
        match self.role {
            Role::Flag { short, .. } => short,
            Role::Positional { .. } | Role::Subcommand => None,
        }
    }

    /// Every long spelling: the name, then its aliases.
    fn longs(&self) -> Vec<&str> {
        match &self.role {
            Role::Flag { long, aliases, .. } => {
                long.iter().chain(aliases).map(String::as_str).collect()
            }
            Role::Positional { .. } | Role::Subcommand => Vec::new(),
        }
    }

    /// The pattern matching any long spelling, if there is one.
    fn long_pattern(&self) -> Option<TokenStream2> {
        let longs = self.longs();
        let literals = longs
            .iter()
            .map(|l| LitByteStr::new(l.as_bytes(), Span::call_site()));
        (!longs.is_empty()).then(|| quote!(#(#literals)|*))
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
            Kind::Count(ty) => quote!(let mut #ident: #ty = 0;),
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
                __arg.check_switch()?;
                #ident = true;
            },
            Kind::Count(_) => quote! {
                __arg.check_switch()?;
                #ident = #ident.saturating_add(1);
            },
            Kind::Optional(ty) | Kind::Required(ty) => quote! {
                #ident = ::core::option::Option::Some(
                    __arg.read_value_as::<#ty>(__input)?
                );
            },
            Kind::Many(ty) => match f.delimiter {
                None => quote! {
                    #ident.push(__arg.read_value_as::<#ty>(__input)?);
                },
                Some(d) => quote! {
                    for __piece in __wa::split(__arg.read_value(__input)?, #d) {
                        #ident.push(__arg.convert::<#ty>(__piece)?);
                    }
                },
            },
        }
    };

    let long_arms = fields.iter().filter_map(|f| {
        let pattern = f.long_pattern()?;
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });
    let short_arms = fields.iter().filter_map(|f| {
        let pattern = LitChar::new(f.short()?, Span::call_site());
        let body = store(f);
        Some(quote!(#pattern => { #body }))
    });

    let unexpected = quote!(return ::core::result::Result::Err(__arg.unexpected()));
    let subcommands: Vec<&Field> = fields
        .iter()
        .filter(|f| matches!(f.role, Role::Subcommand))
        .collect();
    if let Some(extra) = subcommands.get(1) {
        return Err(syn::Error::new(
            extra.ident.span(),
            "a struct has at most one subcommand field",
        ));
    }
    let subcommand = subcommands.first();
    // A word routes to a subcommand only before any positional is filled.
    let track_filled = subcommand.is_some() && !positionals.is_empty();
    let filled = track_filled.then(|| quote!(__filled = true;));

    let positional_match = if positionals.is_empty() {
        quote!({ #unexpected; })
    } else {
        let arms = positionals.iter().enumerate().map(|(i, f)| {
            let ident = slot(&f.ident);
            let display = LitStr::new(&f.display(), Span::call_site());
            match &f.kind {
                Kind::Optional(ty) | Kind::Required(ty) => quote! {
                    #i => {
                        #ident = ::core::option::Option::Some(
                            __word.convert::<#ty>(#display)?
                        );
                        __position += 1;
                        #filled
                    }
                },
                Kind::Many(ty) => {
                    let push = match f.delimiter {
                        None => quote!(#ident.push(__word.convert::<#ty>(#display)?);),
                        Some(d) => quote! {
                            for __piece in __word.split(#d) {
                                #ident.push(__piece.convert::<#ty>(#display)?);
                            }
                        },
                    };
                    quote! {
                        #i => {
                            #push
                            #filled
                        }
                    }
                }
                Kind::Switch | Kind::Count(_) => {
                    unreachable!("rejected for positionals in `field`")
                }
            }
        });
        quote! {
            match __position {
                #(#arms)*
                _ => { #unexpected; }
            }
        }
    };
    let route = subcommand.map(|f| {
        let ident = slot(&f.ident);
        let ty = match &f.kind {
            Kind::Optional(ty) | Kind::Required(ty) => ty,
            _ => unreachable!("rejected for subcommands in `field`"),
        };
        let not_filled = track_filled.then(|| quote!(!__filled &&));
        let is_global = |f: &&Field| matches!(f.role, Role::Flag { global: true, .. });
        let global_longs = fields.iter().filter(is_global).filter_map(|f| {
            let pattern = f.long_pattern()?;
            let body = store(f);
            Some(quote!(#pattern => { #body return ::core::result::Result::Ok(true); }))
        });
        let global_shorts = fields.iter().filter(is_global).filter_map(|f| {
            let pattern = LitChar::new(f.short()?, Span::call_site());
            let body = store(f);
            Some(quote!(#pattern => { #body return ::core::result::Result::Ok(true); }))
        });
        // The subcommand sees this struct's globals first, then its ancestors'.
        let (inherit, handler) = if fields.iter().any(|f| is_global(&f)) {
            let setup = quote! {
                let mut __inherit = __wa::globals(|__arg, __input| {
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#global_longs)*
                            _ => {}
                        },
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#global_shorts)*
                            _ => {}
                        },
                        _ => {}
                    }
                    __globals.bind(__arg, __input)
                });
            };
            (setup, quote!(&mut __inherit))
        } else {
            (quote!(), quote!(&mut *__globals))
        };
        quote! {
            if #not_filled !__word.after_separator {
                if <#ty as __wa::Subcommand>::has(&**__word.value) {
                    let __sub = {
                        #inherit
                        <#ty as __wa::Subcommand>::parse_subcommand(
                            &**__word.value, __input, #handler,
                        )?
                    };
                    #ident = ::core::option::Option::Some(__sub);
                    break;
                }
            }
        }
    });
    let word_arm = quote! {
        __wa::Arg::Word(__word) => {
            #route
            #positional_match
        }
    };
    let position = (!positionals.is_empty()).then(|| quote!(let mut __position: usize = 0;));
    let filled_slot = track_filled.then(|| quote!(let mut __filled = false;));

    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        match &f.kind {
            Kind::Switch | Kind::Count(_) | Kind::Optional(_) | Kind::Many(_) => {
                quote!(#ident: #slot)
            }
            Kind::Required(_) => {
                let display = LitStr::new(&f.display(), Span::call_site());
                let error = match f.role {
                    Role::Positional { .. } => {
                        quote!(__wa::Error::missing_argument(__input.offset(), #display))
                    }
                    Role::Flag { .. } => {
                        quote!(__wa::Error::missing_required(__input.offset(), #display))
                    }
                    Role::Subcommand => quote!(__wa::Error::missing_subcommand(__input.offset())),
                };
                quote! {
                    #ident: match #slot {
                        ::core::option::Option::Some(v) => v,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(#error);
                        }
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                #(#slots)*
                #position
                #filled_slot
                while !__input.is_empty() {
                    let __arg = __wa::arg(__input)?;
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#long_arms)*
                            _ => if !__globals.bind(&__arg, __input)? { #unexpected; },
                        },
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#short_arms)*
                            _ => if !__globals.bind(&__arg, __input)? { #unexpected; },
                        },
                        __wa::Arg::Separator { .. } => {}
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
    let mut count = false;
    let mut subcommand = false;
    let mut global = false;
    let mut alias = Vec::new();
    let mut delimiter = None;
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
            } else if meta.path.is_ident("count") {
                count = true;
            } else if meta.path.is_ident("subcommand") {
                subcommand = true;
            } else if meta.path.is_ident("global") {
                global = true;
            } else if meta.path.is_ident("alias") {
                alias.extend(aliases(&meta)?);
            } else if meta.path.is_ident("delimiter") {
                let c = meta.value()?.parse::<LitChar>()?;
                if !c.value().is_ascii() {
                    return Err(syn::Error::new(c.span(), "a delimiter is one ASCII character"));
                }
                delimiter = Some(c.value() as u8);
            } else if meta.path.is_ident("value_name") {
                value_name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error(
                    "expected `short`, `long`, `alias`, `global`, `positional`, `subcommand`, `count`, `delimiter` or `value_name`",
                ));
            }
            Ok(())
        })?;
    }

    let error = |msg: String| Err(syn::Error::new(f.span(), msg));
    if delimiter.is_some() && (count || !matches!(kind(&f.ty), Kind::Many(_))) {
        return error("`delimiter` needs a `Vec<T>` field to put the pieces in".into());
    }
    let kind = if count {
        Kind::Count(f.ty.clone())
    } else {
        kind(&f.ty)
    };
    let role = if subcommand {
        if positional || count || global || short.is_some() || long.is_some() || !alias.is_empty() {
            return error("a subcommand field takes no other `arg` options".into());
        }
        if !matches!(kind, Kind::Optional(_) | Kind::Required(_)) {
            return error("a subcommand field is `E` or `Option<E>`".into());
        }
        Role::Subcommand
    } else if positional {
        if short.is_some() || long.is_some() || global || !alias.is_empty() {
            return error("a positional field has no `short`, `long`, `alias` or `global`".into());
        }
        if matches!(kind, Kind::Switch | Kind::Count(_)) {
            return error("a positional field cannot be `bool` or `count`".into());
        }
        Role::Positional {
            name: value_name.unwrap_or_else(|| bare.to_uppercase()),
        }
    } else {
        // Like bpaf: a flag with no names is `--field-name`.
        if short.is_none() && long.is_none() {
            long = Some(bare.replace('_', "-"));
        }
        for l in long.iter().chain(&alias) {
            if l.is_empty() || l.starts_with('-') || l.contains('=') {
                return error(format!("`{l}` is not a usable long name"));
            }
        }
        if let Some(c) = short {
            if c == '-' || c == '=' {
                return error(format!("`{c}` is not a usable short name"));
            }
        }
        Role::Flag {
            short,
            long,
            aliases: alias,
            global,
        }
    };

    Ok(Field {
        ident,
        kind,
        role,
        delimiter,
    })
}

fn check_duplicates(fields: &[Field]) -> syn::Result<()> {
    for (i, a) in fields.iter().enumerate() {
        for b in &fields[..i] {
            let clash = match (a.short(), b.short()) {
                (Some(x), Some(y)) if x == y => Some(format!("-{x}")),
                _ => None,
            }
            .or_else(|| {
                let longs = b.longs();
                a.longs()
                    .into_iter()
                    .find(|l| longs.contains(l))
                    .map(|l| format!("--{l}"))
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
        Kind::Many(_) | Kind::Switch | Kind::Count(_) => 2,
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

fn kind(ty: &Type) -> Kind {
    if let Some(last) = last_segment(ty) {
        if last.ident == "bool" && last.arguments.is_none() {
            return Kind::Switch;
        }
        if let Some(inner) = inner(last) {
            if last.ident == "Option" {
                return Kind::Optional(inner.clone());
            }
            if last.ident == "Vec" {
                return Kind::Many(inner.clone());
            }
        }
    }
    Kind::Required(ty.clone())
}
