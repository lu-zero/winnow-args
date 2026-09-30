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

#[proc_macro_derive(Args, attributes(arg, winnow_args))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Subcommand, attributes(arg, winnow_args))]
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
    let case = rename_all(input)?;
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let mut patterns = Vec::new();
    let mut subs = Vec::new();
    for variant in &data.variants {
        let ident = &variant.ident;
        let info = variant_names(variant, &mut names, case)?;
        let pattern = byte_patterns(&info.names);
        let primary = LitStr::new(&info.names[0], Span::call_site());
        let inner = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                Some(boxed(ty).unwrap_or(ty).clone())
            }
            _ => None,
        };
        let command = match &inner {
            Some(ty) => quote!(<#ty as ::winnow_args::Args>::HELP),
            None => {
                let about = &info.about;
                let about = text(about);
                quote!(&::winnow_args::help::Command {
                    name: "",
                    about: #about,
                    long_about: #about,
                    after_help: "",
                    after_long_help: "",
                    items: &[],
                    subcommands: &[],
                    subcommand_required: false,
                    help_flag: true,
                    help_short: true,
                    version: ::core::option::Option::None,
                })
            }
        };
        let about = match (&inner, info.about.is_empty()) {
            (Some(ty), true) => quote!(<#ty as ::winnow_args::Args>::HELP.about),
            _ => text(&info.about),
        };
        let shown = &info.names[1..=info.shown];
        let all = &info.names;
        let hide = info.hide;
        subs.push(quote! {
            ::winnow_args::help::Sub {
                name: #primary,
                aliases: &[#(#shown),*],
                names: &[#(#all),*],
                command: #command,
                about: #about,
                hide: #hide,
            }
        });
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
        arms.push(quote!(#pattern => (#parse).map_err(|e| e.within(#primary)),));
        patterns.push(pattern);
    }
    let (about, long_about) = docs(&input.attrs);
    let (about, long_about) = (text(&about), text(&long_about));

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
            const HELP: &'static ::winnow_args::help::Command = &::winnow_args::help::Command {
                name: "",
                about: #about,
                long_about: #long_about,
                after_help: "",
                after_long_help: "",
                items: &[],
                subcommands: &[#(#subs),*],
                subcommand_required: true,
                help_flag: true,
                help_short: true,
                version: ::core::option::Option::None,
            };

            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                let __help = <Self as ::winnow_args::Args>::HELP;
                if __input.is_empty() {
                    return ::core::result::Result::Err(__wa::Error::missing_subcommand(__input.offset()));
                }
                let __arg = __wa::arg(__input)?;
                match __arg {
                    __wa::Arg::Word(__word) => {
                        if <Self as __wa::Subcommand>::has(&**__word.value) {
                            return <Self as __wa::Subcommand>::parse_subcommand(
                                &**__word.value, __input, __globals,
                            );
                        }
                        if &**__word.value == b"help" {
                            return ::core::result::Result::Err(__wa::help_word(__help, __input));
                        }
                    }
                    __wa::Arg::Long(__flag) if __flag.name == b"help" => {
                        return ::core::result::Result::Err(__wa::Error::help(__help, true));
                    }
                    __wa::Arg::Short(__flag) if __flag.letter == 'h' => {
                        return ::core::result::Result::Err(__wa::Error::help(__help, false));
                    }
                    _ => {}
                }
                ::core::result::Result::Err(__arg.unexpected())
            }
        }
    })
}

/// Derive `Occurrence` for an enum: one variant per flag or positional, kept
/// in command-line order by an `#[arg(sequence)]` field of an `Args` struct.
#[proc_macro_derive(Occurrence, attributes(arg, winnow_args))]
pub fn derive_occurrence(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_occurrence(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ValueEnum, attributes(arg, winnow_args))]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_value_enum(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// `FromArg` as one `match` on the value's bytes.
/// `from_arg`: a `match` on the long name and one on the letter, each arm
/// building its variant; `from_word`: the positional variant, if any.
fn expand_occurrence(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Occurrence` can only be derived for enums",
        ));
    };
    let (mut long_arms, mut short_arms) = (Vec::new(), Vec::new());
    let (mut single_dash, mut prefixes) = (Vec::new(), Vec::new());
    let mut letters = Vec::new();
    let mut word = None;
    for variant in &data.variants {
        let ident = &variant.ident;
        let (mut short, mut longs, mut positional) = (None, Vec::new(), false);
        let (mut two_dashes, mut prefix, mut value_name) = (false, false, None);
        for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("short") {
                    short = Some(meta.value()?.parse::<LitChar>()?.value());
                } else if meta.path.is_ident("long") {
                    longs.push(if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<LitStr>()?.value()
                    } else {
                        kebab_case(&ident.to_string())
                    });
                } else if meta.path.is_ident("alias") {
                    longs.extend(aliases(&meta)?);
                } else if meta.path.is_ident("positional") {
                    positional = true;
                } else if meta.path.is_ident("two_dashes") {
                    two_dashes = true;
                } else if meta.path.is_ident("prefix") {
                    prefix = true;
                } else if meta.path.is_ident("value_name") {
                    value_name = Some(meta.value()?.parse::<LitStr>()?.value());
                } else {
                    return Err(meta.error(
                        "expected `short`, `long`, `alias`, `positional`, `two_dashes`, `prefix` or `value_name`",
                    ));
                }
                Ok(())
            })?;
        }
        let takes = match &variant.fields {
            Fields::Unit => None,
            Fields::Unnamed(f) if f.unnamed.len() == 1 => Some(&f.unnamed[0].ty),
            _ => {
                return Err(syn::Error::new(
                    variant.span(),
                    "an `Occurrence` variant is a unit (a switch) or holds one value",
                ));
            }
        };
        if positional {
            let Some(ty) = takes else {
                return Err(syn::Error::new(
                    variant.span(),
                    "a positional variant holds its value",
                ));
            };
            if word.is_some() {
                return Err(syn::Error::new(
                    variant.span(),
                    "at most one positional variant",
                ));
            }
            let name = value_name.unwrap_or_else(|| ident.to_string().to_uppercase());
            word = Some(quote! {
                ::core::result::Result::Ok(::core::option::Option::Some(
                    Self::#ident(__word.convert::<#ty>(#name)?),
                ))
            });
            continue;
        }
        if short.is_none() && longs.is_empty() {
            longs.push(kebab_case(&ident.to_string()));
        }
        let body = match takes {
            None => quote! {
                __arg.check_switch()?;
                ::core::result::Result::Ok(::core::option::Option::Some(Self::#ident))
            },
            Some(ty) => quote! {
                let __value = __arg.read_value(__input)?;
                ::core::result::Result::Ok(::core::option::Option::Some(
                    Self::#ident(__arg.convert::<#ty>(__value)?),
                ))
            },
        };
        if !longs.is_empty() {
            let pattern = byte_patterns(&longs);
            long_arms.push(quote!(#pattern => { #body }));
            if !two_dashes {
                single_dash.extend(longs.iter().cloned());
            }
        }
        if let Some(c) = short {
            if prefix {
                if !c.is_ascii() || takes.is_none() {
                    return Err(syn::Error::new(
                        variant.span(),
                        "`prefix` is for a value-taking variant with an ASCII `short` letter",
                    ));
                }
                prefixes.push(c as u8);
            }
            let letter = LitChar::new(c, Span::call_site());
            let takes_value = takes.is_some();
            letters.push(quote!(#letter => ::core::option::Option::Some(#takes_value),));
            short_arms.push(quote!(#letter => { #body }));
        }
    }
    let word =
        word.unwrap_or_else(|| quote!(::core::result::Result::Ok(::core::option::Option::None)));
    let is_long = if single_dash.is_empty() {
        quote!(false)
    } else {
        let names = byte_patterns(&single_dash);
        quote!(matches!(__name, #names))
    };
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::winnow_args::Occurrence for #name #ty_generics #where_clause {
            const PREFIXES: &'static [u8] = &[#(#prefixes),*];

            fn from_arg<'__i>(
                __arg: &::winnow_args::Arg<'__i>,
                __input: &mut ::winnow_args::Argv<'__i>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                let _ = &__input;
                match __arg {
                    ::winnow_args::Arg::Long(__flag) => match __flag.name {
                        #(#long_arms)*
                        _ => ::core::result::Result::Ok(::core::option::Option::None),
                    },
                    ::winnow_args::Arg::Short(__flag) if !__flag.plus => match __flag.letter {
                        #(#short_arms)*
                        _ => ::core::result::Result::Ok(::core::option::Option::None),
                    },
                    _ => ::core::result::Result::Ok(::core::option::Option::None),
                }
            }

            fn from_word(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                let _ = __word;
                #word
            }

            fn is_long(__name: &[u8]) -> bool {
                #is_long
            }

            fn short(__letter: char) -> ::core::option::Option<bool> {
                match __letter {
                    #(#letters)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    })
}

fn expand_value_enum(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`ValueEnum` can only be derived for enums",
        ));
    };
    let case = rename_all(input)?;
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let (mut choices, mut visible) = (Vec::new(), Vec::new());
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "a `ValueEnum` variant holds no data",
            ));
        }
        let Variant {
            names: spellings,
            hide,
            ..
        } = variant_names(variant, &mut names, case)?;
        let name = LitStr::new(&spellings[0], Span::call_site());
        if !hide {
            visible.push(name.clone());
        }
        choices.push(name);
        let pattern = byte_patterns(&spellings);
        let ident = &variant.ident;
        arms.push(quote!(#pattern => ::core::result::Result::Ok(Self::#ident),));
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::winnow_args::FromArg for #name #ty_generics #where_clause {
            const CHOICES: &'static [&'static str] = &[#(#visible),*];

            fn from_arg(
                __value: &::winnow_args::__private::BStr,
            ) -> ::core::result::Result<Self, ::winnow_args::__private::BoxError> {
                match &**__value {
                    #(#arms)*
                    _ => ::core::result::Result::Err(::std::boxed::Box::new(
                        ::winnow_args::ChoiceError { choices: &[#(#choices),*] },
                    )),
                }
            }
        }
    })
}

/// Help prose, left out of binaries built without winnow-args' `help-text`.
fn text(prose: &str) -> TokenStream2 {
    if prose.is_empty() {
        quote!("")
    } else {
        quote!(::winnow_args::__text!(#prose))
    }
}

/// A variant's spellings and help.
struct Variant {
    /// `name` (or kebab-case) first, then shown aliases, then hidden ones.
    names: Vec<String>,
    /// How many of `names` after the first are shown in help.
    shown: usize,
    hide: bool,
    about: String,
}

/// A variant's spellings, checked against `seen` for duplicates, and its help.
/// How a variant's name is spelled when it gives none: an enum's
/// `#[arg(rename_all = "…")]`.
#[derive(Clone, Copy)]
enum Case {
    /// `DryRun` → `dry-run` (the default).
    Kebab,
    /// `DryRun` → `dryrun`.
    Lower,
    /// `DryRun` → `DRYRUN`.
    Upper,
    /// `DryRun` → `dry_run`.
    Snake,
    /// `DryRun` → `DryRun`.
    Verbatim,
}

impl Case {
    fn apply(self, ident: &str) -> String {
        let ident = ident.trim_start_matches("r#");
        match self {
            Case::Kebab => kebab_case(ident),
            Case::Lower => ident.to_lowercase(),
            Case::Upper => ident.to_uppercase(),
            Case::Snake => kebab_case(ident).replace('-', "_"),
            Case::Verbatim => ident.to_owned(),
        }
    }
}

/// An enum's `#[arg(rename_all = "…")]`, kebab-case if absent.
fn rename_all(input: &DeriveInput) -> syn::Result<Case> {
    let mut case = Case::Kebab;
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename_all") {
                return Err(meta.error("expected `rename_all`"));
            }
            let lit = meta.value()?.parse::<LitStr>()?;
            case = match lit.value().as_str() {
                "kebab-case" => Case::Kebab,
                "lowercase" => Case::Lower,
                "UPPERCASE" => Case::Upper,
                "snake_case" => Case::Snake,
                "verbatim" => Case::Verbatim,
                _ => {
                    return Err(syn::Error::new(
                        lit.span(),
                        "expected \"kebab-case\", \"lowercase\", \"UPPERCASE\", \"snake_case\" or \"verbatim\"",
                    ));
                }
            };
            Ok(())
        })?;
    }
    Ok(case)
}

fn variant_names<'a>(
    variant: &'a syn::Variant,
    seen: &mut Vec<(String, &'a Ident)>,
    case: Case,
) -> syn::Result<Variant> {
    let (mut name, mut alias, mut hidden, mut hide) = (None, Vec::new(), Vec::new(), false);
    let mut help: Option<String> = None;
    for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("alias") {
                alias.extend(aliases(&meta)?);
            } else if meta.path.is_ident("alias_hidden") {
                hidden.extend(aliases(&meta)?);
            } else if meta.path.is_ident("hide") {
                hide = true;
            } else if meta.path.is_ident("help") {
                help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("long_help") {
                // The listing shows one line; the subcommand's own help has the rest.
                meta.value()?.parse::<LitStr>()?;
            } else {
                return Err(meta.error(
                    "expected `name`, `alias`, `alias_hidden`, `hide`, `help` or `long_help`",
                ));
            }
            Ok(())
        })?;
    }
    let ident = &variant.ident;
    let shown = alias.len();
    let names: Vec<String> =
        std::iter::once(name.unwrap_or_else(|| case.apply(&ident.to_string())))
            .chain(alias)
            .chain(hidden)
            .collect();
    for spelling in &names {
        if let Some((_, other)) = seen.iter().find(|(n, _)| n == spelling) {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{spelling}` is already used by `{other}`"),
            ));
        }
        seen.push((spelling.clone(), ident));
    }
    Ok(Variant {
        names,
        shown,
        hide,
        about: help
            .map(|h| {
                h.split("\n\n")
                    .next()
                    .unwrap_or_default()
                    .replace('\n', " ")
            })
            .unwrap_or_else(|| docs(&variant.attrs).0),
    })
}

/// A doc comment as help: its first paragraph, and the whole of it.
fn docs(attrs: &[syn::Attribute]) -> (String, String) {
    let lines: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        })
        .map(|line| {
            line.strip_prefix(' ')
                .unwrap_or(&line)
                .trim_end()
                .to_owned()
        })
        .collect();
    let long = lines.join("\n").trim().to_owned();
    let short = long
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .replace('\n', " ");
    (short, long)
}

/// `b"a" | b"b"`.
fn byte_patterns(names: &[String]) -> TokenStream2 {
    let literals = names
        .iter()
        .map(|n| LitByteStr::new(n.as_bytes(), Span::call_site()));
    quote!(#(#literals)|*)
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

/// A flag with neither a short nor a long spelling (only `plus`).
fn short_none_long_none(f: &Field) -> bool {
    f.short().is_none() && f.longs().is_empty()
}

/// An attribute for these derives: `#[arg(…)]`, or `#[winnow_args(…)]` on a
/// type that other derives (clap's claim `arg`) also read.
fn is_ours(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("arg") || attr.path().is_ident("winnow_args")
}

/// Whether `ty` is `bool`.
fn is_bool(ty: &Type) -> bool {
    last_segment(ty).is_some_and(|s| s.ident == "bool" && s.arguments.is_none())
}

/// Whether a field is `#[arg(sequence)]`: occurrences of an `Occurrence` enum.
fn is_sequence(f: &syn::Field) -> bool {
    has_flag(f, "sequence")
}

/// Whether a field is `#[arg(flatten)]`: another struct's flags, parsed as ours.
fn is_flatten(f: &syn::Field) -> bool {
    has_flag(f, "flatten")
}

/// Whether a field is `#[arg(unknown)]`: the flags no one declared, whole.
fn is_unknown(f: &syn::Field) -> bool {
    has_flag(f, "unknown")
}

/// Whether a field is `#[arg(skip)]`: left at its default, not parsed.
fn is_skipped(f: &syn::Field) -> bool {
    has_flag(f, "skip")
}

/// Whether one of the field's attributes is the bare word `name`.
fn has_flag(f: &syn::Field, name: &str) -> bool {
    f.attrs.iter().filter(|a| is_ours(a)).any(|a| {
        let mut skip = false;
        let _ = a.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) {
                skip = true;
            } else if meta.input.peek(syn::Token![=]) {
                meta.value()?.parse::<syn::Expr>()?;
            } else if meta.input.peek(syn::token::Paren) {
                let _content;
                syn::parenthesized!(_content in meta.input);
            }
            Ok(())
        });
        skip
    })
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
        double_dash: DoubleDash,
    },
    Subcommand,
}

/// A positional's relation to `--`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DoubleDash {
    /// Words fill it on either side of `--`.
    Optional,
    /// Only words after `--` fill it, and they all do.
    Required,
    /// Once it has a value, flags stop as if `--` had been typed.
    Automatic,
    /// A `--` reaching it is one of its values, and stops nothing.
    Preserve,
}

struct Field {
    ident: Ident,
    kind: Kind,
    role: Role,
    /// Splits each value of a `Vec` field.
    delimiter: Option<u8>,
    /// The only values accepted, checked before conversion.
    choices: Option<Vec<String>>,
    /// Environment variable consulted when the command line gave nothing.
    env: Option<String>,
    /// Value used when neither the command line nor the environment did.
    default: Option<String>,
    /// Selectors that may not be supplied together with this field.
    conflicts: Vec<String>,
    /// Selectors this field replaces, and that replace it: the last one given wins.
    overrides: Vec<String>,
    /// Selectors that must have a value when this field is supplied.
    requires: Vec<String>,
    /// Must end up with a value (for `Option` and `Vec` fields).
    required: bool,
    /// Required unless one of these selectors has a value.
    required_unless: Vec<String>,
    /// The struct-level group this field belongs to.
    group: Option<String>,
    /// The value of a flag given without one.
    default_missing: Option<String>,
    /// Help: first paragraph, whole text, section, hidden.
    help: String,
    long_help: String,
    heading: Option<String>,
    hide: bool,
    /// A flag's value placeholder.
    value_name: Option<String>,
    /// A negative number is a value: a flag's detached value, or a positional's word.
    negative_numbers: bool,
    /// A flag's detached value may be any word, flag-like or `--`.
    hyphen_values: bool,
    /// A flag's value must be attached.
    require_equals: bool,
    /// The long spelling that sets a `bool` false; the slot is then an
    /// `Option<bool>` until the end, so a default fills only what was not given.
    negate: Option<String>,
    /// Never spelled with one dash under `long_only`: `--omagic` (ld).
    two_dashes: bool,
    /// Words each occurrence takes (a `Vec` flag): `-platform_version macos 11.0 12.0`.
    values: usize,
    /// Once this positional has a value, flags stop (whatever its `double_dash`).
    stop_flags: bool,
    /// The letter of its `+c` spelling, in a `plus_options` struct.
    plus: Option<char>,
    /// An `Option<bool>` with `short` and the same `plus` letter: `-c` is
    /// `Some(true)`, `+c` is `Some(false)`.
    tristate: bool,
    /// Its values are words of a vocabulary of their own, the field's type
    /// (`Args`): `-z now -z max-page-size=4096` (ld).
    keywords: bool,
    /// More letters it answers to, from a second `short` (`kill -l`/`-L`).
    short_aliases: Vec<char>,
    /// A short letter that always takes the rest of its word (`-lfoo`), so
    /// under `long_only` no long name starting with it is tried with one dash.
    prefix: bool,
    /// The field's `#[cfg(…)]` attributes, repeated on everything generated
    /// for it.
    cfg: TokenStream2,
}

impl Field {
    /// This field's entry in `help::Command::items`.
    fn help_item(&self) -> TokenStream2 {
        let opt_str = |s: Option<&str>| match s {
            Some(s) => quote!(::core::option::Option::Some(#s)),
            None => quote!(::core::option::Option::None),
        };
        let short = match self.short() {
            Some(c) => quote!(::core::option::Option::Some(#c)),
            None => quote!(::core::option::Option::None),
        };
        let long = opt_str(self.longs().first().copied());
        let negate = opt_str(self.negate.as_deref());
        let (positional, trailing) = match &self.role {
            Role::Positional { double_dash, .. } => (true, *double_dash == DoubleDash::Required),
            _ => (false, false),
        };
        let display = self.display();
        let value_name = if positional {
            opt_str(Some(&display))
        } else {
            opt_str(self.value_name.as_deref())
        };
        let (help, long_help) = (text(&self.help), text(&self.long_help));
        let heading = opt_str(self.heading.as_deref());
        // A field spelled only `+c` has no `-`/`--` row to show.
        let hide =
            self.hide || matches!(self.role, Role::Flag { .. }) && short_none_long_none(self);
        let required = matches!(self.kind, Kind::Required(_))
            && self.default.is_none()
            && self.env.is_none()
            && !self.keywords
            || self.required;
        let multiple = matches!(self.kind, Kind::Many(_) | Kind::Count(_));
        let default = opt_str(self.default.as_deref());
        let env = opt_str(self.env.as_deref());
        // Declared `choices`, else whatever fixed set the value type has.
        let choices = match (&self.choices, &self.kind, &self.role) {
            (Some(choices), _, _) => quote!(&[#(#choices),*]),
            (None, _, Role::Subcommand) | (None, Kind::Switch | Kind::Count(_), _) => quote!(&[]),
            (None, _, _) if self.keywords || self.tristate => quote!(&[]),
            (None, Kind::Optional(ty) | Kind::Required(ty) | Kind::Many(ty), _) => {
                quote!(<#ty as ::winnow_args::FromArg>::CHOICES)
            }
        };
        quote! {
            ::winnow_args::help::Item {
                short: #short,
                long: #long,
                negate: #negate,
                value_name: #value_name,
                help: #help,
                long_help: #long_help,
                heading: #heading,
                hide: #hide,
                positional: #positional,
                required: #required,
                multiple: #multiple,
                trailing: #trailing,
                default: #default,
                env: #env,
                choices: #choices,
            }
        }
    }

    /// Reading this flag's value: `read_value`, or `read_value_or` its `default_missing`.
    fn read(&self) -> TokenStream2 {
        let (negative_numbers, hyphen_values, require_equals) = (
            self.negative_numbers,
            self.hyphen_values,
            self.require_equals,
        );
        let options = quote! {
            __wa::ValueOptions {
                negative_numbers: #negative_numbers,
                hyphen_values: #hyphen_values,
                require_equals: #require_equals,
            }
        };
        match &self.default_missing {
            None if !negative_numbers && !hyphen_values && !require_equals => {
                quote!(__arg.read_value(__input)?)
            }
            None => quote!(__arg.read_value_with(__input, #options)?),
            Some(missing) => {
                let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                quote!(__arg.read_value_or_with(__input, #options, __wa::BStr::new(#bytes)))
            }
        }
    }
}

impl Field {
    /// Reject `value` (a `&BStr`) outside `choices`, reported by `error(cause)`.
    fn check(
        &self,
        value: TokenStream2,
        error: impl FnOnce(TokenStream2) -> TokenStream2,
    ) -> TokenStream2 {
        let Some(choices) = &self.choices else {
            return quote!();
        };
        let pattern = byte_patterns(choices);
        let listed = choices.iter().map(|c| LitStr::new(c, Span::call_site()));
        let cause = quote!(::winnow_args::ChoiceError { choices: &[#(#listed),*] });
        let error = error(cause);
        quote! {
            if !matches!(&**#value, #pattern) {
                return ::core::result::Result::Err(#error);
            }
        }
    }

    /// A flag's value converted to `ty`, checked against `choices` first.
    fn flag_value(&self, ty: &Type, value: TokenStream2) -> TokenStream2 {
        let check = self.check(value.clone(), |cause| {
            quote!(__wa::Error::invalid_value(__arg.offset(), __arg.spelling(), #value, #cause))
        });
        quote!({ #check __arg.convert::<#ty>(#value)? })
    }

    /// `value` (a `&BStr` from the environment or a default) converted to `ty`;
    /// errors name it by `source`.
    fn source_value(&self, ty: &Type, value: TokenStream2, source: &str) -> TokenStream2 {
        let source = LitStr::new(source, Span::call_site());
        let check = self.check(
            value.clone(),
            |cause| quote!(__wa::Error::invalid_value(__input.offset(), #source, #value, #cause)),
        );
        quote!({
            #check
            <#ty as __wa::FromArg>::from_arg(#value).map_err(|cause| {
                __wa::Error::invalid_value(__input.offset(), #source, #value, cause)
            })?
        })
    }

    /// Whether the field holds a value. Read before defaults this is "supplied"
    /// (command line or environment); after them it is "has a value".
    fn has(&self) -> TokenStream2 {
        let slot = slot(&self.ident);
        if self.keywords {
            return quote!((!#slot.is_empty()));
        }
        match &self.kind {
            Kind::Switch if self.negate.is_some() => {
                quote!((#slot == ::core::option::Option::Some(true)))
            }
            Kind::Switch => quote!(#slot),
            Kind::Count(_) => quote!((#slot != 0)),
            Kind::Optional(_) | Kind::Required(_) => quote!(#slot.is_some()),
            Kind::Many(_) => quote!((!#slot.is_empty())),
        }
    }

    /// Fill the field from its environment variable (`env`) or its default when
    /// the command line left it unset and no `overrides` displaced it.
    fn fallback(&self, env: bool, displaced: bool) -> TokenStream2 {
        let slot = slot(&self.ident);
        if self.negate.is_some() {
            return self.negatable_fallback(env);
        }
        let has = self.has();
        let not_displaced = displaced.then(|| {
            let displaced = format_ident!("__displaced_{}", self.ident);
            quote!(&& !#displaced)
        });
        let unset = quote!(!#has #not_displaced);
        let assign = |value: TokenStream2, source: &str| match &self.kind {
            Kind::Optional(ty) | Kind::Required(ty) => {
                let value = self.source_value(ty, value, source);
                quote!(#slot = ::core::option::Option::Some(#value);)
            }
            Kind::Many(ty) => match self.delimiter {
                None => {
                    let value = self.source_value(ty, value, source);
                    quote!(#slot.push(#value);)
                }
                Some(d) => {
                    let piece = self.source_value(ty, quote!(__piece), source);
                    quote!(for __piece in __wa::split(#value, #d) { #slot.push(#piece); })
                }
            },
            Kind::Switch | Kind::Count(_) => unreachable!("handled below"),
        };
        if !env {
            return self
                .default
                .as_ref()
                .map(|value| {
                    let bytes = LitByteStr::new(value.as_bytes(), Span::call_site());
                    let assign = assign(quote!(__wa::BStr::new(#bytes)), &self.display());
                    quote!(if #unset { #assign })
                })
                .unwrap_or_default();
        }
        self.env
            .as_ref()
            .map(|var| {
                let name = LitStr::new(var, Span::call_site());
                let apply = match &self.kind {
                    Kind::Switch => quote!(#slot = __wa::env::truthy(&__raw);),
                    Kind::Count(ty) => quote! {
                        if let ::core::option::Option::Some(::core::result::Result::Ok(__n)) =
                            __raw.to_str().map(str::parse::<#ty>)
                        {
                            #slot = __n;
                        }
                    },
                    _ => {
                        let assign = assign(quote!(__value), &format!("${var}"));
                        quote! {
                            let __value = __wa::BStr::new(__raw.as_encoded_bytes());
                            #assign
                        }
                    }
                };
                quote! {
                    if #unset {
                        if let ::core::option::Option::Some(__raw) = __wa::env::var(#name) {
                            #apply
                        }
                    }
                }
            })
            .unwrap_or_default()
    }

    /// A negatable switch's fallback: only when neither spelling was given.
    fn negatable_fallback(&self, env: bool) -> TokenStream2 {
        let slot = slot(&self.ident);
        let value = if env {
            let Some(var) = &self.env else {
                return quote!();
            };
            let name = LitStr::new(var, Span::call_site());
            quote! {
                if let ::core::option::Option::Some(__raw) = __wa::env::var(#name) {
                    #slot = ::core::option::Option::Some(__wa::env::truthy(&__raw));
                }
            }
        } else {
            let Some(default) = &self.default else {
                return quote!();
            };
            let default = default == "true";
            quote!(#slot = ::core::option::Option::Some(#default);)
        };
        quote!(if #slot.is_none() { #value })
    }

    /// A positional word converted to `ty`, checked against `choices` first.
    fn word_value(&self, ty: &Type, word: TokenStream2, display: &LitStr) -> TokenStream2 {
        let check = self.check(
            quote!(#word.value),
            |cause| quote!(__wa::Error::invalid_value(#word.offset, #display, #word.value, #cause)),
        );
        quote!({ #check #word.convert::<#ty>(#display)? })
    }
}

impl Field {
    /// How errors name the field: its long flag, else its short one, else its value name.
    fn display(&self) -> String {
        match &self.role {
            Role::Flag { long: Some(l), .. } => format!("--{l}"),
            Role::Flag { short: Some(c), .. } => format!("-{c}"),
            Role::Flag { .. } => match self.plus {
                Some(c) => format!("+{c}"),
                None => unreachable!("every flag has a name"),
            },
            Role::Positional { name, .. } => name.clone(),
            Role::Subcommand => "<COMMAND>".to_owned(),
        }
    }

    /// The pattern of every letter it answers to: `'l' | 'L'`.
    fn short_pattern(&self) -> Option<TokenStream2> {
        let first = self.short()?;
        let letters = std::iter::once(first)
            .chain(self.short_aliases.iter().copied())
            .map(|c| LitChar::new(c, Span::call_site()));
        Some(quote!(#(#letters)|*))
    }

    /// Every letter it answers to.
    fn shorts(&self) -> Vec<char> {
        self.short()
            .into_iter()
            .chain(self.short_aliases.iter().copied())
            .collect()
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

    /// Whether a flag reads a value (rather than being a switch or a count).
    fn takes_value(&self) -> bool {
        !matches!(self.kind, Kind::Switch | Kind::Count(_))
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
        .filter(|f| !is_skipped(f) && !is_sequence(f) && !is_unknown(f) && !is_flatten(f))
        .map(field)
        .collect::<syn::Result<Vec<_>>>()?;
    let skipped: Vec<&Ident> = named
        .named
        .iter()
        .filter(|f| is_skipped(f) && !is_sequence(f) && !is_unknown(f) && !is_flatten(f))
        .filter_map(|f| f.ident.as_ref())
        .collect();
    // `#[arg(sequence)] items: Vec<T>`: occurrences of `T`'s flags and
    // positional, in order.
    let sequences: Vec<&syn::Field> = named.named.iter().filter(|f| is_sequence(f)).collect();
    if let Some(extra) = sequences.get(1) {
        return Err(syn::Error::new(
            extra.span(),
            "a struct has at most one `sequence` field",
        ));
    }
    let sequence = match sequences.first() {
        Some(f) => {
            let Some(ty) = last_segment(&f.ty)
                .filter(|s| s.ident == "Vec")
                .and_then(inner)
            else {
                return Err(syn::Error::new(
                    f.ty.span(),
                    "a `sequence` field is a `Vec<T>` of an `Occurrence` enum",
                ));
            };
            Some((f.ident.clone().expect("named field"), ty.clone()))
        }
        None => None,
    };
    // `#[arg(unknown)] unknown: Vec<T>`: flag-like words naming no flag.
    let unknowns: Vec<&syn::Field> = named.named.iter().filter(|f| is_unknown(f)).collect();
    if let Some(extra) = unknowns.get(1) {
        return Err(syn::Error::new(
            extra.span(),
            "a struct has at most one `unknown` field",
        ));
    }
    let unknown = match unknowns.first() {
        Some(f) => {
            let Some(ty) = last_segment(&f.ty)
                .filter(|s| s.ident == "Vec")
                .and_then(inner)
            else {
                return Err(syn::Error::new(
                    f.ty.span(),
                    "an `unknown` field is a `Vec<T>`",
                ));
            };
            Some((f.ident.clone().expect("named field"), ty.clone()))
        }
        None => None,
    };
    // `#[arg(flatten)] common: T`: `T`'s flags, parsed as if declared here.
    let flattens: Vec<(Ident, Type)> = named
        .named
        .iter()
        .filter(|f| is_flatten(f))
        .map(|f| (f.ident.clone().expect("named field"), f.ty.clone()))
        .collect();
    if !flattens.is_empty() && !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "`flatten` in a generic struct is not supported yet",
        ));
    }
    check_duplicates(&fields)?;
    let dd = |f: &Field| match f.role {
        Role::Positional { double_dash, .. } => double_dash,
        _ => DoubleDash::Optional,
    };
    // A `double_dash = "required"` positional is outside the ordinary sequence:
    // every word after `--` goes to it, and no word before.
    let positionals: Vec<&Field> = fields
        .iter()
        .filter(|f| f.is_positional() && dd(f) != DoubleDash::Required)
        .collect();
    check_positional_order(&positionals)?;
    let trailing: Vec<&Field> = fields
        .iter()
        .filter(|f| dd(f) == DoubleDash::Required)
        .collect();
    if let Some(extra) = trailing.get(1) {
        return Err(syn::Error::new(
            extra.ident.span(),
            "a struct has at most one `double_dash = \"required\"` positional",
        ));
    }
    let trailing = trailing.first().copied();
    // The positionals a `--` fills while they are next, by position.
    let preserving: Vec<usize> = positionals
        .iter()
        .enumerate()
        .filter(|(_, f)| dd(f) == DoubleDash::Preserve)
        .map(|(i, _)| i)
        .collect();
    if let (Some(t), false) = (trailing, preserving.is_empty()) {
        return Err(syn::Error::new(
            t.ident.span(),
            "a `double_dash = \"preserve\"` positional keeps the `--` this one waits for",
        ));
    }
    let StructOptions {
        groups,
        restart_token,
        default_subcommand,
        arg_required_else_help,
        name: program_name,
        version,
        about,
        long_about,
        after_help,
        after_long_help,
        disable_help_flag,
        disable_help_short,
        disable_version_flag,
        disable_help_subcommand,
        unknown_flags_value,
        long_only,
        plus_options,
    } = struct_options(input)?;
    let (doc_about, doc_long_about) = docs(&input.attrs);
    let about = about.unwrap_or(doc_about);
    let long_about = long_about.unwrap_or(doc_long_about);
    let (about, long_about) = (text(&about), text(&long_about));
    let (after_help, after_long_help) = (text(&after_help), text(&after_long_help));
    let rules = Rules::new(&fields, &groups)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Each field's slot: its `cfg`, name, type and starting value.
    let slot_parts: Vec<(&TokenStream2, Ident, TokenStream2, TokenStream2)> = fields
        .iter()
        .map(|f| {
            let none = quote!(::core::option::Option::None);
            let new = quote!(::std::vec::Vec::new());
            let (ty, init) = match &f.kind {
                // Borrows the command line, so never in a `Flatten` slot struct.
                _ if f.keywords => (quote!(::std::vec::Vec<&__wa::BStr>), new),
                Kind::Switch if f.negate.is_some() => (quote!(::core::option::Option<bool>), none),
                Kind::Switch => (quote!(bool), quote!(false)),
                Kind::Count(ty) => (quote!(#ty), quote!(0)),
                Kind::Optional(ty) | Kind::Required(ty) => {
                    (quote!(::core::option::Option<#ty>), none)
                }
                Kind::Many(ty) => (quote!(::std::vec::Vec<#ty>), new),
            };
            (&f.cfg, slot(&f.ident), ty, init)
        })
        .collect();
    let slots = slot_parts
        .iter()
        .map(|(cfg, ident, ty, init)| quote!(#cfg let mut #ident: #ty = #init;));

    // What storing one flag occurrence looks like; the same for both spellings.
    let store = |f: &Field| {
        let ident = slot(&f.ident);
        let displace = rules.displace(&fields, f);
        let stored = match &f.kind {
            _ if f.keywords => {
                let read = f.read();
                quote! {
                    let __value = #read;
                    #ident.push(__value);
                }
            }
            _ if f.tristate => quote! {
                __arg.check_switch()?;
                #ident = ::core::option::Option::Some(true);
            },
            Kind::Switch if f.negate.is_some() => quote! {
                __arg.check_switch()?;
                #ident = ::core::option::Option::Some(true);
            },
            Kind::Switch => quote! {
                __arg.check_switch()?;
                #ident = true;
            },
            Kind::Count(_) => quote! {
                __arg.check_switch()?;
                #ident = #ident.saturating_add(1);
            },
            Kind::Optional(ty) | Kind::Required(ty) => {
                let value = f.flag_value(ty, quote!(__value));
                let read = f.read();
                quote! {
                    let __value = #read;
                    #ident = ::core::option::Option::Some(#value);
                }
            }
            Kind::Many(ty) => match f.delimiter {
                None => {
                    let value = f.flag_value(ty, quote!(__value));
                    let read = f.read();
                    // `values = N`: the words after the first, whatever they look like.
                    let more = (f.values > 1).then(|| {
                        let more = f.values - 1;
                        quote! {
                            for _ in 0..#more {
                                let __value = __arg.read_next(__input)?;
                                #ident.push(#value);
                            }
                        }
                    });
                    quote! {
                        let __value = #read;
                        #ident.push(#value);
                        #more
                    }
                }
                Some(d) => {
                    let value = f.flag_value(ty, quote!(__piece));
                    let read = f.read();
                    quote! {
                        for __piece in __wa::split(#read, #d) {
                            #ident.push(#value);
                        }
                    }
                }
            },
        };
        quote!(#stored #displace)
    };

    let long_arms = fields.iter().filter_map(|f| {
        let pattern = f.long_pattern()?;
        let body = store(f);
        let cfg = &f.cfg;
        Some(quote!(#cfg #pattern => { #body }))
    });
    let negated = |f: &Field| {
        let no = LitByteStr::new(f.negate.as_ref()?.as_bytes(), Span::call_site());
        let ident = slot(&f.ident);
        let displace = rules.displace(&fields, f);
        Some((
            quote!(#no),
            quote! {
                __arg.check_switch()?;
                #ident = ::core::option::Option::Some(false);
                #displace
            },
        ))
    };
    let negated_arms = fields.iter().filter_map(|f| {
        let (pattern, body) = negated(f)?;
        let cfg = &f.cfg;
        Some(quote!(#cfg #pattern => { #body }))
    });
    let long_arms = long_arms.chain(negated_arms);
    let short_arms = fields.iter().filter_map(|f| {
        let pattern = f.short_pattern()?;
        let body = store(f);
        let cfg = &f.cfg;
        Some(quote!(#cfg #pattern => { #body }))
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

    // A word no ordinary positional takes: before `--`, it would have reached
    // the `double_dash = "required"` positional if there is one.
    let overflow = match trailing {
        Some(t) => {
            let display = LitStr::new(&t.display(), Span::call_site());
            quote! {
                return ::core::result::Result::Err(
                    __wa::Error::requires_double_dash(__word.offset, #display),
                )
            }
        }
        None => unexpected.clone(),
    };
    let positional_match = if positionals.is_empty() {
        quote!({ #overflow; })
    } else {
        let arms = positionals.iter().enumerate().map(|(i, f)| {
            let ident = slot(&f.ident);
            let display = LitStr::new(&f.display(), Span::call_site());
            let stop = (dd(f) == DoubleDash::Automatic || f.stop_flags)
                .then(|| quote!(__input.stop_flags();));
            match &f.kind {
                Kind::Optional(ty) | Kind::Required(ty) => {
                    let value = f.word_value(ty, quote!(__word), &display);
                    quote! {
                        #i => {
                            #ident = ::core::option::Option::Some(#value);
                            __position += 1;
                            #filled
                            #stop
                        }
                    }
                }
                Kind::Many(ty) => {
                    let push = match f.delimiter {
                        None => {
                            let value = f.word_value(ty, quote!(__word), &display);
                            quote!(#ident.push(#value);)
                        }
                        Some(d) => {
                            let value = f.word_value(ty, quote!(__piece), &display);
                            quote! {
                                for __piece in __word.split(#d) {
                                    #ident.push(#value);
                                }
                            }
                        }
                    };
                    quote! {
                        #i => {
                            #push
                            #filled
                            #stop
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
                _ => { #overflow; }
            }
        }
    };
    let positional_match = match trailing {
        None => positional_match,
        Some(t) => {
            let ident = slot(&t.ident);
            let display = LitStr::new(&t.display(), Span::call_site());
            let store = match &t.kind {
                Kind::Many(ty) => match t.delimiter {
                    None => {
                        let value = t.word_value(ty, quote!(__word), &display);
                        quote!(#ident.push(#value);)
                    }
                    Some(d) => {
                        let value = t.word_value(ty, quote!(__piece), &display);
                        quote!(for __piece in __word.split(#d) { #ident.push(#value); })
                    }
                },
                Kind::Optional(ty) | Kind::Required(ty) => {
                    let value = t.word_value(ty, quote!(__word), &display);
                    quote! {
                        if #ident.is_some() { #unexpected; }
                        #ident = ::core::option::Option::Some(#value);
                    }
                }
                Kind::Switch | Kind::Count(_) => {
                    unreachable!("rejected for positionals in `field`")
                }
            };
            quote! {
                if __word.after_separator {
                    #store
                } else {
                    #positional_match
                }
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
        let global_negated = fields.iter().filter(is_global).filter_map(|f| {
            let (pattern, body) = negated(f)?;
            Some(quote!(#pattern => { #body return ::core::result::Result::Ok(true); }))
        });
        let global_longs = global_longs.chain(global_negated);
        let global_shorts = fields.iter().filter(is_global).filter_map(|f| {
            let pattern = f.short_pattern()?;
            let body = store(f);
            Some(quote!(#pattern => { #body return ::core::result::Result::Ok(true); }))
        });
        // The subcommand sees this struct's globals first, then its ancestors'.
        let (inherit, handler) = if fields.iter().any(|f| is_global(&f)) {
            let shorts = fields.iter().filter(is_global).flat_map(|f| {
                let takes_value = f.takes_value();
                f.shorts().into_iter().map(move |c| {
                    let c = LitChar::new(c, Span::call_site());
                    quote!((#c, #takes_value))
                })
            });
            let setup = quote! {
                let mut __inherit = __wa::inherit(&[#(#shorts),*], __globals, |__arg, __input, __parent| {
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
                    __parent.bind(__arg, __input)
                });
            };
            (setup, quote!(&mut __inherit))
        } else {
            (quote!(), quote!(&mut *__globals))
        };
        // `help a b`: the help of a subcommand, unless one is actually named `help`.
        let help_word = (!disable_help_subcommand).then(|| {
            quote! {
                if &**__word.value == b"help" {
                    return ::core::result::Result::Err(
                        __wa::help_word(<Self as ::winnow_args::Args>::HELP, __input),
                    );
                }
            }
        });
        // Any other word selects the default subcommand, which reads it again as its own.
        let fallback = default_subcommand.as_ref().map(|name| {
            let name = LitByteStr::new(name.as_bytes(), Span::call_site());
            quote! {
                let __sub = {
                    #inherit
                    *__input = __start;
                    <#ty as __wa::Subcommand>::parse_subcommand(#name, __input, #handler)?
                };
                #ident = ::core::option::Option::Some(__sub);
                break;
            }
        });
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
                #help_word
                #fallback
            }
        }
    });
    // A new invocation of the same command: positionals start over, flags keep theirs.
    let restart = restart_token.map(|token| {
        let token = LitByteStr::new(token.as_bytes(), Span::call_site());
        let clears = fields.iter().filter(|f| f.is_positional()).map(|f| {
            let slot = slot(&f.ident);
            match f.kind {
                Kind::Many(_) => quote!(#slot.clear();),
                _ => quote!(#slot = ::core::option::Option::None;),
            }
        });
        let position = (!positionals.is_empty()).then(|| quote!(__position = 0;));
        let filled = track_filled.then(|| quote!(__filled = false;));
        quote! {
            if &**__word.value == #token {
                #(#clears)*
                #position
                #filled
                __input.resume_flags();
                continue;
            }
        }
    });
    let sequence_word = sequence.as_ref().map(|(_, ty)| {
        quote! {
            if let ::core::option::Option::Some(__item) =
                <#ty as ::winnow_args::Occurrence>::from_word(&__word)?
            {
                __sequence.push(__item);
                continue;
            }
        }
    });
    let word_arm = quote! {
        __wa::Arg::Word(__word) => {
            #restart
            #route
            #sequence_word
            #positional_match
        }
    };
    let separator_arm = if preserving.is_empty() {
        quote!(__wa::Arg::Separator { .. } => {})
    } else {
        quote! {
            __wa::Arg::Separator { offset: __offset } => {
                if matches!(__position, #(#preserving)|*) {
                    __input.resume_flags();
                    let __word = __wa::Word {
                        value: __wa::BStr::new(b"--"),
                        offset: __offset,
                        after_separator: false,
                    };
                    #positional_match
                }
            }
        }
    };
    let position = (!positionals.is_empty()).then(|| quote!(let mut __position: usize = 0;));
    let filled_slot = track_filled.then(|| quote!(let mut __filled = false;));

    // A negative number is a word where the positional next to fill opts in,
    // unless it spells a declared digit short exactly (`-0` for fd's `--print0`).
    let opted: Vec<usize> = positionals
        .iter()
        .enumerate()
        .filter(|(_, f)| f.negative_numbers)
        .map(|(i, _)| i)
        .collect();
    let (unknown_slot, unknown_build) = match &unknown {
        Some((ident, ty)) => (
            quote!(let mut __unknown: ::std::vec::Vec<#ty> = ::std::vec::Vec::new();),
            quote!(#ident: __unknown,),
        ),
        None => (quote!(), quote!()),
    };
    let (sequence_slot, sequence_arg, sequence_build) = match &sequence {
        Some((ident, ty)) => (
            quote!(let mut __sequence: ::std::vec::Vec<#ty> = ::std::vec::Vec::new();),
            quote! {
                if let ::core::option::Option::Some(__item) =
                    <#ty as ::winnow_args::Occurrence>::from_arg(&__arg, __input)?
                {
                    __sequence.push(__item);
                } else
            },
            quote!(#ident: __sequence,),
        ),
        None => (quote!(), quote!(), quote!()),
    };
    // `#[arg(flatten)]` fields: their slots, the flags offered to them after
    // ours, and their values built at the end.
    let flat_slot = |ident: &Ident| format_ident!("__flat_{}", ident);
    let flatten_slots = flattens.iter().map(|(ident, ty)| {
        let slot = flat_slot(ident);
        quote! {
            let mut #slot = <<#ty as __wa::Flatten>::Slots as ::core::default::Default>::default();
        }
    });
    let flatten_arg: TokenStream2 = flattens
        .iter()
        .map(|(ident, ty)| {
            let slot = flat_slot(ident);
            quote!(if <#ty as __wa::Flatten>::bind(&mut #slot, &__arg, __input)? {} else)
        })
        .collect();
    let flatten_build = flattens.iter().map(|(ident, ty)| {
        let slot = flat_slot(ident);
        quote!(#ident: <#ty as __wa::Flatten>::finish(#slot, __input)?,)
    });
    let flatten_shorts = flattens.iter().map(|(_, ty)| {
        quote! {
            __c if <#ty as __wa::Flatten>::short(__c).is_some() => {
                <#ty as __wa::Flatten>::short(__c)
            }
        }
    });
    let flatten_longs = flattens
        .iter()
        .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_long(__name)));
    let lexer = if plus_options {
        quote!(__wa::arg_plus)
    } else {
        quote!(__wa::arg)
    };
    if !plus_options {
        if let Some(f) = fields.iter().find(|f| f.plus.is_some()) {
            return Err(syn::Error::new(
                f.ident.span(),
                "`plus` needs `#[arg(plus_options)]` on the struct",
            ));
        }
    }
    let plus_arms = fields.iter().filter_map(|f| {
        let letter = LitChar::new(f.plus?, Span::call_site());
        let ident = slot(&f.ident);
        let displace = rules.displace(&fields, f);
        let body = if f.tristate {
            quote!(#ident = ::core::option::Option::Some(false); #displace)
        } else {
            store(f)
        };
        let cfg = &f.cfg;
        Some(quote!(#cfg #letter => { #body }))
    });
    let plus_match = plus_options.then(|| {
        quote! {
            __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                #(#plus_arms)*
                _ => #flatten_arg { #unexpected; }
            },
        }
    });
    let lex = if opted.is_empty() {
        quote!(#lexer(__input)?)
    } else {
        let digits: Vec<LitByteStr> = fields
            .iter()
            .filter_map(|f| f.short().filter(char::is_ascii_digit))
            .map(|c| LitByteStr::new(format!("-{c}").as_bytes(), Span::call_site()))
            .collect();
        let not_a_short =
            (!digits.is_empty()).then(|| quote!(&&!matches!(__input.front(), #(#digits)|*)));
        quote! {
            match matches!(__position, #(#opted)|*) #not_a_short {
                true => match __wa::number(__input) {
                    ::core::result::Result::Ok(__word) => __wa::Arg::Word(__word),
                    ::core::result::Result::Err(_) => #lexer(__input)?,
                },
                false => #lexer(__input)?,
            }
        }
    };
    let start = default_subcommand
        .is_some()
        .then(|| quote!(let __start = *__input;));
    let help_on_empty = arg_required_else_help.then(|| {
        quote! {
            if __input.is_empty() {
                return ::core::result::Result::Err(
                    __wa::Error::bare_help(<Self as ::winnow_args::Args>::HELP),
                );
            }
        }
    });
    if default_subcommand.is_some() && subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand` needs an `#[arg(subcommand)]` field",
        ));
    }
    // `-h`/`--help` and `-V`/`--version`, unless declared or disabled: arms after
    // the struct's own, so a CLI naming its own `--help` keeps it.
    let declares_long = |name: &str| fields.iter().any(|f| f.longs().contains(&name));
    let declares_short = |c: char| fields.iter().any(|f| f.shorts().contains(&c));
    let help = quote!(<Self as ::winnow_args::Args>::HELP);
    let help_long = (!disable_help_flag && !declares_long("help")).then(
        || quote!(b"help" => return ::core::result::Result::Err(__wa::Error::help(#help, true)),),
    );
    let help_short = (!disable_help_flag && !disable_help_short && !declares_short('h')).then(
        || quote!('h' => return ::core::result::Result::Err(__wa::Error::help(#help, false)),),
    );
    let with_version = version.is_some() && !disable_version_flag;
    let version_long = (with_version && !declares_long("version")).then(
        || quote!(b"version" => return ::core::result::Result::Err(__wa::Error::version(#help)),),
    );
    let version_short = (with_version && !declares_short('V'))
        .then(|| quote!('V' => return ::core::result::Result::Err(__wa::Error::version(#help)),));
    let help_flag = help_long.is_some() || help_short.is_some();
    let help_short_flag = help_short.is_some();

    // `long_only`: `-name` is `--name` for the long names that may take one
    // dash, tried before the word is read as short letters.
    // `unknown_flags = "value"`: a flag-like word naming no flag goes to the
    // positionals whole, never to a subcommand. A bundle of several letters is
    // checked before any binds; a long word or a single letter once no arm or
    // global took it.
    // An `unknown` field takes those words instead of the positionals.
    let (lenient_bundle, save_token, unknown_flag) = if unknown_flags_value || unknown.is_some() {
        // With nothing to take a word, `positional_match` always returns.
        let next = (!positionals.is_empty() || trailing.is_some()).then(|| quote!(continue;));
        let (sink, sink_next) = match &unknown {
            Some((_, ty)) => (
                quote!(__unknown.push(__word.convert::<#ty>("<UNKNOWN>")?);),
                quote!(continue;),
            ),
            None => (
                quote! {
                    let __arg = __wa::Arg::Word(__word);
                    #positional_match
                },
                quote!(#next),
            ),
        };
        let theirs = match sequence.as_ref().map(|(_, ty)| ty) {
            Some(ty) => quote! {
                __c => match <#ty as ::winnow_args::Occurrence>::short(__c) {
                    ::core::option::Option::None => __globals.short(__c),
                    __known => __known,
                },
            },
            None => quote!(_ => __globals.short(__c),),
        };
        let own = fields.iter().filter_map(|f| {
            let letters = f.short_pattern()?;
            let takes_value = f.takes_value();
            let cfg = &f.cfg;
            Some(quote!(#cfg #letters => ::core::option::Option::Some(#takes_value),))
        });
        let builtin = help_short
            .is_some()
            .then(|| quote!('h' => ::core::option::Option::Some(false),))
            .into_iter()
            .chain(
                version_short
                    .is_some()
                    .then(|| quote!('V' => ::core::option::Option::Some(false),)),
            );
        (
            quote! {
                if let ::core::option::Option::Some(__word) = __wa::unknown_bundle(__input, |__c| match __c {
                    #(#own)*
                    #(#builtin)*
                    #(#flatten_shorts)*
                    #theirs
                }) {
                    #sink
                    #sink_next
                }
            },
            quote!(let __token = __input.front();),
            quote! {
                let __word = __wa::Word {
                    value: __wa::BStr::new(__token),
                    offset: __flag.offset,
                    after_separator: false,
                };
                #sink
            },
        )
    } else {
        (quote!(), quote!(), quote!(#unexpected;))
    };
    let single_dash_longs = |fields: &[Field]| -> Vec<String> {
        let mut names: Vec<String> = fields
            .iter()
            .filter(|f| !f.two_dashes)
            .flat_map(|f| spellings(f).into_iter().map(str::to_owned))
            .collect();
        names.extend(help_long.is_some().then(|| "help".to_owned()));
        names.extend(version_long.is_some().then(|| "version".to_owned()));
        names
    };
    let names = if long_only {
        single_dash_longs(&fields)
    } else {
        Vec::new()
    };
    let sequence_ty = sequence.as_ref().map(|(_, ty)| ty);
    let long_only_lex =
        long_only && (!names.is_empty() || sequence_ty.is_some() || !flattens.is_empty());
    let lex = if long_only_lex {
        let prefixes: Vec<u8> = fields
            .iter()
            .filter(|f| f.prefix)
            .filter_map(|f| f.short().map(|c| c as u8))
            .collect();
        let own = if names.is_empty() {
            quote!(false)
        } else {
            let names = byte_patterns(&names);
            quote!(matches!(__name, #names))
        };
        let (their_names, their_prefixes) = match sequence_ty {
            Some(ty) => (
                quote!(|| <#ty as ::winnow_args::Occurrence>::is_long(__name)),
                quote! {
                    && !__name.first().is_some_and(|__c| {
                        <#ty as ::winnow_args::Occurrence>::PREFIXES.contains(__c)
                    })
                },
            ),
            None => (quote!(), quote!()),
        };
        let not_prefixed = (!prefixes.is_empty()).then(|| {
            let prefixes = prefixes.iter().map(|b| quote!(#b));
            quote!(!matches!(__name.first(), ::core::option::Option::Some(#(#prefixes)|*)) &&)
        });
        quote! {
            match __wa::long_only(__input, |__name| {
                #not_prefixed (#own #their_names #(#flatten_longs)*) #their_prefixes
            }) {
                ::core::option::Option::Some(__arg) => __arg,
                ::core::option::Option::None => {
                    #lenient_bundle
                    #lex
                }
            }
        }
    } else {
        lex
    };
    let lenient_bundle = if long_only_lex {
        quote!()
    } else {
        lenient_bundle
    };

    // Only the first letter of a word can be unknown here: the bundle check
    // vouched for the letters of a longer one.
    let unknown_short = unknown_flag.clone();
    let help_version = match &version {
        Some(v) if with_version => quote!(::core::option::Option::Some(#v)),
        _ => quote!(::core::option::Option::None),
    };
    let help_items = fields
        .iter()
        .filter(|f| !matches!(f.role, Role::Subcommand))
        .map(|f| {
            let (cfg, item) = (&f.cfg, f.help_item());
            quote!(#cfg #item)
        });
    let (help_subcommands, subcommand_required) = match subcommand {
        Some(f) => {
            let ty = match &f.kind {
                Kind::Optional(ty) | Kind::Required(ty) => ty,
                _ => unreachable!("rejected for subcommands in `field`"),
            };
            (
                quote!(<#ty as ::winnow_args::Args>::HELP.subcommands),
                matches!(f.kind, Kind::Required(_)),
            )
        }
        None => (quote!(&[]), false),
    };
    let displaced = rules.displaced_slots(&fields);
    let with_cfg = |f: &Field, code: TokenStream2| {
        if f.cfg.is_empty() || code.is_empty() {
            code
        } else {
            let cfg = &f.cfg;
            quote!(#cfg { #code })
        }
    };
    let env_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| with_cfg(f, f.fallback(true, rules.is_displaced(&fields, f))))
        .collect();
    let default_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| with_cfg(f, f.fallback(false, rules.is_displaced(&fields, f))))
        .collect();
    let exclusive = rules.exclusive(&fields, &groups);
    let supplied = rules.supplied(&fields);
    let required = rules.required(&fields, &groups);
    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        let cfg = &f.cfg;
        let value = match &f.kind {
            Kind::Required(ty) if f.keywords => {
                let display = LitStr::new(&f.display(), Span::call_site());
                quote!(#ident: __wa::keywords::<#ty>(&#slot, #display, __input.offset())?)
            }
            Kind::Switch if f.negate.is_some() => quote!(#ident: #slot.unwrap_or(false)),
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
        };
        quote!(#cfg #value)
    });
    let build: Vec<TokenStream2> = build.collect();
    let flatten_build: Vec<TokenStream2> = flatten_build.collect();
    // Flags only: no positionals, subcommand, `sequence`, `unknown`,
    // keywords, `global` flags or generics.
    let flattenable = positionals.is_empty()
        && trailing.is_none()
        && subcommand.is_none()
        && sequence.is_none()
        && unknown.is_none()
        && !unknown_flags_value
        && input.generics.params.is_empty()
        && !fields
            .iter()
            .any(|f| f.keywords || matches!(f.role, Role::Flag { global: true, .. }));
    let flatten_impl = flattenable.then(|| {
        let vis = &input.vis;
        let slots_name = format_ident!("__{}WinnowArgsSlots", name);
        let displaced_idents: Vec<Ident> = fields
            .iter()
            .filter(|f| rules.is_displaced(&fields, f))
            .map(|f| format_ident!("__displaced_{}", f.ident))
            .collect();
        let nested: Vec<(Ident, &Type)> =
            flattens.iter().map(|(ident, ty)| (flat_slot(ident), ty)).collect();
        let decls = slot_parts.iter().map(|(cfg, ident, ty, _)| quote!(#cfg #ident: #ty,));
        let inits = slot_parts.iter().map(|(cfg, ident, _, init)| quote!(#cfg #ident: #init,));
        let empty = TokenStream2::new();
        let all: Vec<(&TokenStream2, &Ident)> = slot_parts
            .iter()
            .map(|(cfg, ident, _, _)| (*cfg, ident))
            .chain(displaced_idents.iter().map(|d| (&empty, d)))
            .chain(nested.iter().map(|(n, _)| (&empty, n)))
            .collect();
        let names: Vec<TokenStream2> =
            all.iter().map(|(cfg, ident)| quote!(#cfg #ident,)).collect();
        let mutable: Vec<TokenStream2> =
            all.iter().map(|(cfg, ident)| quote!(#cfg mut #ident,)).collect();
        let displaced_decls = displaced_idents.iter().map(|d| quote!(#d: bool,));
        let displaced_inits = displaced_idents.iter().map(|d| quote!(#d: false,));
        let nested_decls = nested
            .iter()
            .map(|(n, ty)| quote!(#n: <#ty as ::winnow_args::__private::Flatten>::Slots,));
        let nested_inits = nested
            .iter()
            .map(|(n, _)| quote!(#n: ::core::default::Default::default(),));
        let arm = |cfg: &TokenStream2, pattern: TokenStream2, body: TokenStream2| {
            quote!(#cfg #pattern => { #body true })
        };
        let bind_long = fields.iter().filter_map(|f| {
            Some(arm(&f.cfg, f.long_pattern()?, store(f)))
        });
        let bind_negated = fields.iter().filter_map(|f| {
            let (pattern, body) = negated(f)?;
            Some(arm(&f.cfg, pattern, body))
        });
        let bind_short = fields.iter().filter_map(|f| {
            Some(arm(&f.cfg, f.short_pattern()?, store(f)))
        });
        let bind_plus = fields.iter().filter_map(|f| {
            let letter = LitChar::new(f.plus?, Span::call_site());
            let ident = slot(&f.ident);
            let displace = rules.displace(&fields, f);
            let body = if f.tristate {
                quote!(#ident = ::core::option::Option::Some(false); #displace)
            } else {
                store(f)
            };
            Some(arm(&f.cfg, quote!(#letter), body))
        });
        let bind_nested: TokenStream2 = nested
            .iter()
            .map(|(n, ty)| {
                quote!(if <#ty as __wa::Flatten>::bind(&mut #n, &__arg, __input)? { true } else)
            })
            .collect();
        let plus = plus_options.then(|| {
            quote! {
                __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                    #(#bind_plus)*
                    _ => #bind_nested { false }
                },
            }
        });
        let letters = fields.iter().filter_map(|f| {
            let letters = f.short_pattern()?;
            let takes_value = f.takes_value();
            let cfg = &f.cfg;
            Some(quote!(#cfg #letters => ::core::option::Option::Some(#takes_value),))
        });
        let nested_shorts = nested.iter().map(|(_, ty)| {
            quote! {
                __c if <#ty as __wa::Flatten>::short(__c).is_some() => {
                    <#ty as __wa::Flatten>::short(__c)
                }
            }
        });
        let longs: Vec<String> = fields
            .iter()
            .filter(|f| !f.two_dashes)
            .flat_map(|f| spellings(f).into_iter().map(str::to_owned))
            .collect();
        let own_longs = if longs.is_empty() {
            quote!(false)
        } else {
            let patterns = byte_patterns(&longs);
            quote!(matches!(__name, #patterns))
        };
        let nested_longs = nested
            .iter()
            .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_long(__name)));
        quote! {
            #[doc(hidden)]
            #vis struct #slots_name {
                #(#decls)*
                #(#displaced_decls)*
                #(#nested_decls)*
            }

            impl ::core::default::Default for #slots_name {
                fn default() -> Self {
                    Self {
                        #(#inits)*
                        #(#displaced_inits)*
                        #(#nested_inits)*
                    }
                }
            }

            #[automatically_derived]
            #[allow(unused_mut, unused_variables, unreachable_code, clippy::all)]
            impl ::winnow_args::__private::Flatten for #name {
                type Slots = #slots_name;

                fn bind<'__i>(
                    __slots: &mut #slots_name,
                    __arg: &::winnow_args::Arg<'__i>,
                    __input: &mut ::winnow_args::Argv<'__i>,
                ) -> ::core::result::Result<bool, ::winnow_args::Error> {
                    use ::winnow_args::__private as __wa;
                    let __arg = *__arg;
                    let #slots_name { #(#mutable)* } = ::core::mem::take(__slots);
                    let __bound = match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#bind_long)*
                            #(#bind_negated)*
                            _ => #bind_nested { false }
                        },
                        #plus
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#bind_short)*
                            _ => #bind_nested { false }
                        },
                        _ => false,
                    };
                    *__slots = #slots_name { #(#names)* };
                    ::core::result::Result::Ok(__bound)
                }

                fn short(__c: char) -> ::core::option::Option<bool> {
                    use ::winnow_args::__private as __wa;
                    match __c {
                        #(#letters)*
                        #(#nested_shorts)*
                        _ => ::core::option::Option::None,
                    }
                }

                fn is_long(__name: &[u8]) -> bool {
                    use ::winnow_args::__private as __wa;
                    #own_longs #(#nested_longs)*
                }

                fn finish(
                    __slots: #slots_name,
                    __input: &::winnow_args::Argv<'_>,
                ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                    use ::winnow_args::__private as __wa;
                    let #slots_name { #(#mutable)* } = __slots;
                    #(#env_fallbacks)*
                    #exclusive
                    #supplied
                    #(#default_fallbacks)*
                    #required
                    ::core::result::Result::Ok(Self {
                        #(#build,)*
                        #(#skipped: ::core::default::Default::default(),)*
                        #(#flatten_build)*
                    })
                }
            }
        }
    });
    let own_items = quote!(&[#(#help_items),*]);
    let items = if flattens.is_empty() {
        own_items
    } else {
        let tys: Vec<&Type> = flattens.iter().map(|(_, ty)| ty).collect();
        quote!({
            const __OWN: &[::winnow_args::help::Item] = #own_items;
            const __N: usize = __OWN.len() #(+ <#tys as ::winnow_args::Args>::HELP.items.len())*;
            const __ALL: [::winnow_args::help::Item; __N] = ::winnow_args::__private::concat_items::<__N>(
                &[__OWN, #(<#tys as ::winnow_args::Args>::HELP.items),*],
            );
            &__ALL
        })
    };

    Ok(quote! {
        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            const HELP: &'static ::winnow_args::help::Command = &::winnow_args::help::Command {
                name: #program_name,
                about: #about,
                long_about: #long_about,
                after_help: #after_help,
                after_long_help: #after_long_help,
                items: #items,
                subcommands: #help_subcommands,
                subcommand_required: #subcommand_required,
                help_flag: #help_flag,
                help_short: #help_short_flag,
                version: #help_version,
            };

            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                #(#slots)*
                #sequence_slot
                #unknown_slot
                #(#flatten_slots)*
                #(#displaced)*
                #position
                #filled_slot
                #help_on_empty
                while !__input.is_empty() {
                    #start
                    #lenient_bundle
                    #save_token
                    let __arg = #lex;
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#long_arms)*
                            #help_long
                            #version_long
                            _ => #sequence_arg #flatten_arg if !__globals.bind(&__arg, __input)? { #unknown_flag },
                        },
                        #plus_match
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#short_arms)*
                            #help_short
                            #version_short
                            _ => #sequence_arg #flatten_arg if !__globals.bind(&__arg, __input)? { #unknown_short },
                        },
                        #separator_arm
                        #word_arm
                    }
                }
                #(#env_fallbacks)*
                #exclusive
                #supplied
                #(#default_fallbacks)*
                #required
                ::core::result::Result::Ok(Self {
                    #(#build,)*
                    #(#skipped: ::core::default::Default::default(),)*
                    #sequence_build
                    #unknown_build
                    #(#flatten_build)*
                })
            }
        }

        #flatten_impl
    })
}

/// A struct-level `group("name", required, multiple)`.
struct Group {
    name: String,
    required: bool,
    multiple: bool,
}

/// `#[arg(...)]` on the struct: groups, and the word that restarts parsing.
/// Options declared on the struct rather than on a field.
#[derive(Default)]
struct StructOptions {
    groups: Vec<Group>,
    /// The word that starts a new invocation of this command.
    restart_token: Option<String>,
    /// The subcommand a word naming none selects.
    default_subcommand: Option<String>,
    /// A bare invocation asks for help.
    arg_required_else_help: bool,
    /// The program name for the usage line; argv[0] when empty.
    name: String,
    /// `-V`/`--version` text: a literal, or `CARGO_PKG_VERSION`.
    version: Option<TokenStream2>,
    /// Help text around the lists; `about`/`long_about` default to the doc comment.
    about: Option<String>,
    long_about: Option<String>,
    after_help: String,
    after_long_help: String,
    disable_help_flag: bool,
    /// Help is `--help` alone, `-h` left undeclared (bash's builtins).
    disable_help_short: bool,
    disable_version_flag: bool,
    disable_help_subcommand: bool,
    /// An unknown flag-like word is a positional value, as usage's default.
    unknown_flags_value: bool,
    /// GNU's `getopt_long_only`: a long name may be spelled with one dash.
    long_only: bool,
    /// `+abc` is a bundle of `+` options (`set +eu`), for fields with `plus`.
    plus_options: bool,
}

fn struct_options(input: &DeriveInput) -> syn::Result<StructOptions> {
    let mut options = StructOptions::default();
    let mut texts = StructOptions::default();
    let groups = &mut options.groups;
    let (restart, default_subcommand, help) = (
        &mut options.restart_token,
        &mut options.default_subcommand,
        &mut options.arg_required_else_help,
    );
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("restart_token") {
                *restart = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand") {
                *default_subcommand = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("arg_required_else_help") {
                *help = true;
                return Ok(());
            }
            if meta.path.is_ident("long_only") {
                texts.long_only = true;
                return Ok(());
            }
            if meta.path.is_ident("plus_options") {
                texts.plus_options = true;
                return Ok(());
            }
            if meta.path.is_ident("unknown_flags") {
                let mode = meta.value()?.parse::<LitStr>()?;
                texts.unknown_flags_value = match mode.value().as_str() {
                    "value" => true,
                    "error" => false,
                    _ => {
                        return Err(syn::Error::new(
                            mode.span(),
                            "expected \"value\" or \"error\"",
                        ));
                    }
                };
                return Ok(());
            }
            let text = |meta: &syn::meta::ParseNestedMeta<'_>| -> syn::Result<String> {
                Ok(meta.value()?.parse::<LitStr>()?.value())
            };
            if meta.path.is_ident("name") {
                texts.name = text(&meta)?;
                return Ok(());
            }
            if meta.path.is_ident("version") {
                texts.version = Some(if meta.input.peek(syn::Token![=]) {
                    let version = meta.value()?.parse::<LitStr>()?;
                    quote!(#version)
                } else {
                    quote!(::core::env!("CARGO_PKG_VERSION"))
                });
                return Ok(());
            }
            for (key, slot) in [
                ("about", &mut texts.about),
                ("long_about", &mut texts.long_about),
            ] {
                if meta.path.is_ident(key) {
                    *slot = Some(text(&meta)?);
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("after_help", &mut texts.after_help),
                ("after_long_help", &mut texts.after_long_help),
            ] {
                if meta.path.is_ident(key) {
                    *slot = text(&meta)?;
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("disable_help_flag", &mut texts.disable_help_flag),
                ("disable_help_short", &mut texts.disable_help_short),
                ("disable_version_flag", &mut texts.disable_version_flag),
                (
                    "disable_help_subcommand",
                    &mut texts.disable_help_subcommand,
                ),
            ] {
                if meta.path.is_ident(key) {
                    *slot = true;
                    return Ok(());
                }
            }
            if !meta.path.is_ident("group") {
                return Err(meta.error(
                    "expected `group(\"name\", ...)`, `restart_token`, `default_subcommand`, \
                     `arg_required_else_help`, `unknown_flags`, `long_only`, `plus_options`, `name`, `version`, `about`, \
                     `long_about`, `after_help`, `after_long_help` or a `disable_*` option",
                ));
            }
            let content;
            syn::parenthesized!(content in meta.input);
            let name = content.parse::<LitStr>()?.value();
            let (mut required, mut multiple) = (false, false);
            while !content.is_empty() {
                content.parse::<syn::Token![,]>()?;
                let flag = content.parse::<Ident>()?;
                match flag.to_string().as_str() {
                    "required" => required = true,
                    "multiple" => multiple = true,
                    _ => {
                        return Err(syn::Error::new(
                            flag.span(),
                            "expected `required` or `multiple`",
                        ));
                    }
                }
            }
            groups.push(Group {
                name,
                required,
                multiple,
            });
            Ok(())
        })?;
    }
    options.name = texts.name;
    options.version = texts.version;
    options.about = texts.about;
    options.long_about = texts.long_about;
    options.after_help = texts.after_help;
    options.after_long_help = texts.after_long_help;
    options.disable_help_flag = texts.disable_help_flag;
    options.disable_help_short = texts.disable_help_short;
    options.disable_version_flag = texts.disable_version_flag;
    options.disable_help_subcommand = texts.disable_help_subcommand;
    options.unknown_flags_value = texts.unknown_flags_value;
    options.long_only = texts.long_only;
    options.plus_options = texts.plus_options;
    Ok(options)
}

/// The relations between fields, resolved to field indices.
struct Rules {
    /// Pairs `(a, b)` that may not both be supplied; each unordered pair once.
    conflicts: Vec<(usize, usize)>,
    /// For each field, the fields it displaces when bound (symmetric).
    beats: Vec<Vec<usize>>,
    /// `(field, target)`: `field` supplied needs `target` to have a value.
    requires: Vec<(usize, usize)>,
    /// `(field, others)`: `field` needs a value unless one of `others` has one.
    required_unless: Vec<(usize, Vec<usize>)>,
    /// Members of each struct-level group, in `groups` order.
    members: Vec<Vec<usize>>,
}

impl Rules {
    fn new(fields: &[Field], groups: &[Group]) -> syn::Result<Self> {
        let resolve = |from: &Field, selector: &str| -> syn::Result<usize> {
            fields
                .iter()
                .position(
                    |f| match (selector.strip_prefix("--"), selector.strip_prefix('-')) {
                        (Some(long), _) => f.longs().contains(&long),
                        (None, Some(short)) => {
                            let mut chars = short.chars();
                            chars
                                .next()
                                .is_some_and(|c| chars.next().is_none() && f.short() == Some(c))
                        }
                        (None, None) => {
                            matches!(&f.role, Role::Positional { name, .. } if name == selector)
                                || f.ident == selector
                        }
                    },
                )
                .ok_or_else(|| {
                    syn::Error::new(
                        from.ident.span(),
                        format!("`{selector}` names no flag or positional of this struct"),
                    )
                })
        };
        let mut rules = Rules {
            conflicts: Vec::new(),
            beats: vec![Vec::new(); fields.len()],
            requires: Vec::new(),
            required_unless: Vec::new(),
            members: vec![Vec::new(); groups.len()],
        };
        for (i, f) in fields.iter().enumerate() {
            for selector in &f.conflicts {
                let j = resolve(f, selector)?;
                let pair = (i.min(j), i.max(j));
                if !rules.conflicts.contains(&pair) {
                    rules.conflicts.push(pair);
                }
            }
            for selector in &f.overrides {
                let j = resolve(f, selector)?;
                if !rules.beats[i].contains(&j) {
                    rules.beats[i].push(j);
                }
                if !rules.beats[j].contains(&i) {
                    rules.beats[j].push(i);
                }
            }
            for selector in &f.requires {
                rules.requires.push((i, resolve(f, selector)?));
            }
            if !f.required_unless.is_empty() {
                let others = f
                    .required_unless
                    .iter()
                    .map(|selector| resolve(f, selector))
                    .collect::<syn::Result<_>>()?;
                rules.required_unless.push((i, others));
            }
            if let Some(group) = &f.group {
                let g = groups
                    .iter()
                    .position(|g| &g.name == group)
                    .ok_or_else(|| {
                        syn::Error::new(
                            f.ident.span(),
                            format!("no `#[arg(group(\"{group}\"))]` on the struct"),
                        )
                    })?;
                rules.members[g].push(i);
            }
        }
        Ok(rules)
    }

    /// Whether an `overrides` can unset `f`, so its fallbacks must check.
    fn is_displaced(&self, fields: &[Field], f: &Field) -> bool {
        !self.beats[index_of(fields, f)].is_empty()
    }

    /// `let mut __displaced_x = false;` for every field an `overrides` can unset.
    fn displaced_slots(&self, fields: &[Field]) -> Vec<TokenStream2> {
        fields
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.beats[*i].is_empty())
            .map(|(_, f)| {
                let displaced = format_ident!("__displaced_{}", f.ident);
                quote!(let mut #displaced = false;)
            })
            .collect()
    }

    /// After binding `f`: unset what it beats, and mark `f` as standing.
    fn displace(&self, fields: &[Field], f: &Field) -> TokenStream2 {
        let i = index_of(fields, f);
        if self.beats[i].is_empty() {
            return quote!();
        }
        let standing = format_ident!("__displaced_{}", f.ident);
        let clears = self.beats[i].iter().map(|&j| {
            let other = &fields[j];
            let slot = slot(&other.ident);
            let displaced = format_ident!("__displaced_{}", other.ident);
            let clear = match &other.kind {
                _ if other.keywords => quote!(#slot.clear();),
                Kind::Switch if other.negate.is_some() => {
                    quote!(#slot = ::core::option::Option::None;)
                }
                Kind::Switch => quote!(#slot = false;),
                Kind::Count(_) => quote!(#slot = 0;),
                Kind::Optional(_) | Kind::Required(_) => {
                    quote!(#slot = ::core::option::Option::None;)
                }
                Kind::Many(_) => quote!(#slot.clear();),
            };
            quote!(#clear #displaced = true;)
        });
        quote!(#standing = false; #(#clears)*)
    }

    /// Conflicts and at-most-one groups, judged on what was supplied.
    fn exclusive(&self, fields: &[Field], groups: &[Group]) -> TokenStream2 {
        let pairs = self.conflicts.iter().map(|&(a, b)| {
            let (fa, fb) = (&fields[a], &fields[b]);
            let (has_a, has_b) = (fa.has(), fb.has());
            let (name_a, name_b) = (fa.display(), fb.display());
            quote! {
                if #has_a && #has_b {
                    return ::core::result::Result::Err(
                        __wa::Error::conflict(__input.offset(), #name_a, #name_b),
                    );
                }
            }
        });
        let at_most_one = groups.iter().zip(&self.members).filter(|(g, _)| !g.multiple).map(|(_, members)| {
            let checks = members.iter().map(|&m| {
                let has = fields[m].has();
                let name = fields[m].display();
                quote! {
                    if #has {
                        if let ::core::option::Option::Some(__first) = __first {
                            return ::core::result::Result::Err(
                                __wa::Error::conflict(__input.offset(), __first, #name),
                            );
                        }
                        __first = ::core::option::Option::Some(#name);
                    }
                }
            });
            quote! {{
                let mut __first: ::core::option::Option<&'static str> = ::core::option::Option::None;
                #(#checks)*
            }}
        });
        quote!(#(#pairs)* #(#at_most_one)*)
    }

    /// Whether each `requires`-declaring field was supplied, before defaults fill it.
    fn supplied(&self, fields: &[Field]) -> TokenStream2 {
        let mut seen = Vec::new();
        let snapshots = self.requires.iter().filter_map(|&(i, _)| {
            if seen.contains(&i) {
                return None;
            }
            seen.push(i);
            let supplied = format_ident!("__supplied_{}", fields[i].ident);
            let has = fields[i].has();
            Some(quote!(let #supplied = #has;))
        });
        quote!(#(#snapshots)*)
    }

    /// `required`, `required_unless`, required groups and `requires`, judged on
    /// what has a value, defaults included.
    fn required(&self, fields: &[Field], groups: &[Group]) -> TokenStream2 {
        let missing = |f: &Field| {
            let name = f.display();
            match f.role {
                Role::Positional { .. } => {
                    quote!(__wa::Error::missing_argument(__input.offset(), #name))
                }
                _ => quote!(__wa::Error::missing_required(__input.offset(), #name)),
            }
        };
        let plain = fields.iter().filter(|f| f.required).map(|f| {
            let has = f.has();
            let error = missing(f);
            quote!(if !#has { return ::core::result::Result::Err(#error); })
        });
        let unless = self.required_unless.iter().map(|(i, others)| {
            let has = fields[*i].has();
            let others = others.iter().map(|&j| fields[j].has());
            let error = missing(&fields[*i]);
            quote!(if !#has #(&& !#others)* { return ::core::result::Result::Err(#error); })
        });
        let one_of = groups
            .iter()
            .zip(&self.members)
            .filter(|(g, _)| g.required)
            .map(|(g, members)| {
                let name = &g.name;
                let has = members.iter().map(|&m| fields[m].has());
                let names = members.iter().map(|&m| fields[m].display());
                quote! {
                    if #(!#has)&&* {
                        return ::core::result::Result::Err(
                            __wa::Error::missing_one_of(__input.offset(), #name, &[#(#names),*]),
                        );
                    }
                }
            });
        let requires = self.requires.iter().map(|&(i, j)| {
            let supplied = format_ident!("__supplied_{}", fields[i].ident);
            let has = fields[j].has();
            let (by, target) = (fields[i].display(), fields[j].display());
            quote! {
                if #supplied && !#has {
                    return ::core::result::Result::Err(
                        __wa::Error::required_by(__input.offset(), #target, #by),
                    );
                }
            }
        });
        quote!(#(#plain)* #(#unless)* #(#one_of)* #(#requires)*)
    }
}

fn index_of(fields: &[Field], f: &Field) -> usize {
    fields
        .iter()
        .position(|g| g.ident == f.ident)
        .expect("a field of this struct")
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
    let mut choices = None;
    let mut env = None;
    let mut default = None;
    let mut double_dash = None;
    let (mut default_missing, mut value_optional) = (None, false);
    let mut negative_numbers = false;
    let mut hyphen_values = false;
    let mut require_equals = false;
    let (mut two_dashes, mut prefix) = (false, false);
    let mut values = 1;
    let mut stop_flags = false;
    let mut plus: Option<char> = None;
    let mut keywords = false;
    let mut short_aliases = Vec::new();
    let mut negate: Option<Option<String>> = None;
    let (doc_help, doc_long_help) = docs(&f.attrs);
    let (mut help, mut long_help, mut heading, mut hide) = (None, None, None, false);
    let (mut conflicts, mut overrides, mut requires) = (Vec::new(), Vec::new(), Vec::new());
    let (mut required, mut required_unless, mut group) = (false, Vec::new(), None);
    let mut value_name = None;
    for attr in f.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("short") {
                let c = if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitChar>()?.value()
                } else {
                    bare.chars()
                        .next()
                        .ok_or_else(|| meta.error("cannot infer a short name"))?
                };
                // A second `short` is another letter, as `long` is another name.
                match short {
                    None => short = Some(c),
                    Some(_) => short_aliases.push(c),
                }
            } else if meta.path.is_ident("long") {
                let name = if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitStr>()?.value()
                } else {
                    bare.replace('_', "-")
                };
                // A second `long` is another spelling, as in usage.
                match long {
                    None => long = Some(name),
                    Some(_) => alias.push(name),
                }
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
            } else if meta.path.is_ident("help") {
                help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("long_help") {
                long_help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("help_heading") {
                heading = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("hide") {
                hide = true;
            } else if meta.path.is_ident("negate") {
                negate = Some(if meta.input.peek(syn::Token![=]) {
                    let name = meta.value()?.parse::<LitStr>()?.value();
                    // usage spells it `"--no-color"`.
                    Some(name.strip_prefix("--").unwrap_or(&name).to_owned())
                } else {
                    None
                });
            } else if meta.path.is_ident("keywords") {
                keywords = true;
            } else if meta.path.is_ident("plus") {
                plus = Some(meta.value()?.parse::<LitChar>()?.value());
            } else if meta.path.is_ident("stop_flags") {
                stop_flags = true;
            } else if meta.path.is_ident("values") {
                let n = meta.value()?.parse::<syn::LitInt>()?;
                values = n.base10_parse::<usize>()?;
                if values == 0 {
                    return Err(syn::Error::new(n.span(), "a flag takes at least one value"));
                }
            } else if meta.path.is_ident("two_dashes") {
                two_dashes = true;
            } else if meta.path.is_ident("prefix") {
                prefix = true;
            } else if meta.path.is_ident("require_equals") {
                require_equals = true;
            } else if meta.path.is_ident("allow_hyphen_values") {
                hyphen_values = true;
            } else if meta.path.is_ident("allow_negative_numbers") {
                negative_numbers = true;
            } else if meta.path.is_ident("default_missing") {
                default_missing = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("value_optional") {
                value_optional = true;
            } else if meta.path.is_ident("double_dash") {
                let mode = meta.value()?.parse::<LitStr>()?;
                double_dash = Some(match mode.value().as_str() {
                    "required" => DoubleDash::Required,
                    "automatic" => DoubleDash::Automatic,
                    "optional" => DoubleDash::Optional,
                    "preserve" => DoubleDash::Preserve,
                    _ => {
                        return Err(syn::Error::new(
                            mode.span(),
                            "expected \"required\", \"automatic\", \"optional\" or \"preserve\"",
                        ));
                    }
                });
            } else if meta.path.is_ident("conflicts") {
                conflicts.extend(aliases(&meta)?);
            } else if meta.path.is_ident("overrides") {
                overrides.extend(aliases(&meta)?);
            } else if meta.path.is_ident("requires") {
                requires.extend(aliases(&meta)?);
            } else if meta.path.is_ident("required_unless") {
                required_unless.extend(aliases(&meta)?);
            } else if meta.path.is_ident("required") {
                required = true;
            } else if meta.path.is_ident("group") {
                group = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("env") {
                env = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("default") {
                default = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("choices") {
                let content;
                syn::parenthesized!(content in meta.input);
                let list =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
                choices = Some(list.iter().map(LitStr::value).collect());
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
                    "unknown `arg` option; expected one of `short`, `long`, `alias`, `global`, `count`, \
                     `positional`, `value_name`, `double_dash`, `subcommand`, `delimiter`, `choices`, `env`, `default`, \
                     `default_missing`, `value_optional`, `allow_negative_numbers`, `allow_hyphen_values`, `require_equals`, `negate`, `two_dashes`, `prefix`, `values`, `stop_flags`, `plus`, `skip`, `keywords`, \
                     `conflicts`, `overrides`, `requires`, `required`, `required_unless`, `group`",
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
    if value_optional && default_missing.is_none() {
        return error("`value_optional` needs `default_missing`: the value of a bare flag".into());
    }
    if default_missing.is_some()
        && (positional || subcommand || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error("`default_missing` is for flags that take a value".into());
    }
    if (hyphen_values || require_equals)
        && (positional || subcommand || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error(
            "`allow_hyphen_values` and `require_equals` are for flags that take a value".into(),
        );
    }
    if negative_numbers && (subcommand || matches!(kind, Kind::Switch | Kind::Count(_))) {
        return error("`allow_negative_numbers` is for fields that take a value".into());
    }
    let tristate =
        plus.is_some() && plus == short && matches!(&kind, Kind::Optional(ty) if is_bool(ty));
    if plus.is_some()
        && (positional || subcommand || !tristate && matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error(
            "`plus` is for a flag taking a value, or an `Option<bool>` with the same `short` letter".into(),
        );
    }
    if keywords
        && (positional
            || subcommand
            || !matches!(kind, Kind::Required(_))
            || env.is_some()
            || default.is_some()
            || default_missing.is_some()
            || choices.is_some()
            || delimiter.is_some())
    {
        return error(
            "`keywords` is for a flag whose type derives `Args`, without `env`, `default`, \
             `default_missing`, `choices` or `delimiter`"
                .into(),
        );
    }
    if stop_flags && !positional {
        return error("`stop_flags` is for positional fields".into());
    }
    if double_dash.is_some() && !positional {
        return error("`double_dash` is for positional fields".into());
    }
    if values > 1
        && (positional
            || subcommand
            || !matches!(kind, Kind::Many(_))
            || delimiter.is_some()
            || default_missing.is_some())
    {
        return error(
            "`values` is for a `Vec<T>` flag, without `delimiter` or `default_missing`".into(),
        );
    }
    if two_dashes && (long.is_none() && alias.is_empty() || positional || subcommand) {
        return error("`two_dashes` is for flags with a long name".into());
    }
    if prefix
        && (short.is_none_or(|c| !c.is_ascii())
            || positional
            || subcommand
            || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error("`prefix` is for a value-taking flag with an ASCII `short` letter".into());
    }
    let flag_value_name =
        (!positional && !subcommand && !matches!(kind, Kind::Switch | Kind::Count(_)))
            .then(|| value_name.clone().unwrap_or_else(|| bare.to_uppercase()));
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
            double_dash: double_dash.unwrap_or(DoubleDash::Optional),
            name: value_name.unwrap_or_else(|| bare.to_uppercase()),
        }
    } else {
        // Like bpaf: a flag with no names is `--field-name` (unless it is
        // spelled only `+c`).
        if short.is_none() && long.is_none() && plus.is_none() {
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

    if choices.is_some()
        && (matches!(kind, Kind::Switch | Kind::Count(_)) || matches!(role, Role::Subcommand))
    {
        return error("`choices` needs a field that takes a value".into());
    }
    if (env.is_some() || default.is_some()) && matches!(role, Role::Subcommand) {
        return error("a subcommand field takes no `env` or `default`".into());
    }
    let negate = match negate {
        None => None,
        Some(_) if !matches!(kind, Kind::Switch) || !matches!(role, Role::Flag { .. }) => {
            return error("`negate` is for `bool` flags".into());
        }
        Some(Some(name)) => Some(name),
        Some(None) => match &role {
            Role::Flag { long: Some(l), .. } => Some(format!("no-{l}")),
            _ => return error("`negate` without a name needs a `long` to prefix".into()),
        },
    };
    if let Some(no) = &negate {
        if no.is_empty() || no.starts_with('-') || no.contains('=') {
            return error(format!("`{no}` is not a usable long name"));
        }
    }
    match (&default, &negate) {
        (Some(d), Some(_)) if d != "true" && d != "false" => {
            return error("a negatable flag's `default` is \"true\" or \"false\"".into());
        }
        (Some(_), None) if matches!(kind, Kind::Switch | Kind::Count(_)) => {
            return error("`default` needs a field that takes a value, or `negate`".into());
        }
        _ => {}
    }
    let cfg: TokenStream2 = f
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .map(|a| quote!(#a))
        .collect();
    if !cfg.is_empty() {
        let plain_flag = matches!(role, Role::Flag { .. })
            && !global
            && group.is_none()
            && conflicts.is_empty()
            && overrides.is_empty()
            && requires.is_empty()
            && required_unless.is_empty()
            && !required;
        if !plain_flag {
            return error(
                "`#[cfg]` is supported on flags that no rule, group or `global` names".into(),
            );
        }
    }
    Ok(Field {
        cfg,
        ident,
        kind,
        role,
        delimiter,
        choices,
        env,
        default,
        conflicts,
        overrides,
        requires,
        required,
        required_unless,
        group,
        default_missing,
        long_help: long_help.or_else(|| help.clone()).unwrap_or(doc_long_help),
        help: help.unwrap_or(doc_help),
        heading,
        hide,
        value_name: if tristate { None } else { flag_value_name },
        negative_numbers,
        hyphen_values,
        require_equals,
        negate,
        two_dashes,
        prefix,
        values,
        stop_flags,
        plus,
        tristate,
        keywords,
        short_aliases,
    })
}

/// Every long spelling of `f`, its negation included.
fn spellings(f: &Field) -> Vec<&str> {
    let mut longs = f.longs();
    longs.extend(f.negate.as_deref());
    longs
}

fn check_duplicates(fields: &[Field]) -> syn::Result<()> {
    for (i, a) in fields.iter().enumerate() {
        for b in &fields[..i] {
            if !a.cfg.is_empty() && !b.cfg.is_empty() {
                // Alternatives for different configurations, as a rule.
                continue;
            }
            let theirs = b.shorts();
            let clash = a
                .shorts()
                .into_iter()
                .find(|c| theirs.contains(c))
                .map(|c| format!("-{c}"))
                .or_else(|| {
                    let longs = spellings(b);
                    spellings(a)
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
