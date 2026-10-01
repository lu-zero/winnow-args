//! The derives of winnow-args: `Args`, `Subcommand`, `ValueEnum` and `Occurrence`.
//!
//! Each is documented with every attribute it accepts. `#[winnow_args(...)]` is
//! `#[arg(...)]` under another name, for a type whose other derives (clap's)
//! claim `arg`.
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

/// Derive `Args` for a struct with named fields: the generated loop parses the whole command line.
///
/// Every attribute is `#[arg(...)]` (or `#[winnow_args(...)]`); a field with none is `--field-name`.
/// Doc comments are the help text.
///
/// # Struct options
///
/// Naming and help:
///
/// - `name = "…"`: the program name in usage and completion scripts; `argv[0]` when empty.
/// - `version`, `version = "…"`: supply `-V`/`--version`; bare, the crate's `CARGO_PKG_VERSION`.
/// - `about`, `long_about`, `after_help`, `after_long_help`: help text; the first two default to the doc comment.
/// - `disable_help_flag`, `disable_help_short` (`--help` only), `disable_version_flag`,
///   `disable_help_subcommand`: do not supply that item.
/// - `arg_required_else_help`: no arguments at all is a request for help.
///
/// Grammar:
///
/// - `unknown_flags = "value"`: a flag-like word naming no flag is a positional value; `"error"` is the default.
/// - `long_only`: a long name may be spelled with one dash (GNU `getopt_long_only`).
/// - `plus_options`: `+abc` is a bundle of `+` options, for fields with `plus`.
/// - `restart_token = "…"`: a word that starts a new run of positionals; flags resume and keep their values.
/// - `default_subcommand = "…"`: the subcommand for a line whose first word names none; needs a `subcommand` field.
/// - `group("name", required, multiple)`: declare a group; fields join it with `group = "name"`.
///
/// # Field roles
///
/// A field is a flag unless it is one of these:
///
/// - `positional`: a word. `T` is required, `Option<T>` optional, `Vec<T>` every word left; in that order.
/// - `subcommand`: a `Subcommand` enum, `E` or `Option<E>`; takes no other option.
/// - `flatten`: another flags-only `Args` struct, parsed as if declared here (nested too). A flag spelled in
///   both is a compile error. The flattened struct has no positionals, subcommand, `sequence`, `unknown`,
///   `keywords`, `global` flags or generics, nor `unknown_flags = "value"`; the parent no generics.
/// - `sequence`: a `Vec` of an `Occurrence` enum: its flags and words, kept in order, its flags in help.
///   One per struct; a spelling it shares with the struct is a compile error.
/// - `sequence, unknown`: the same, with unknown flags offered to the enum's `unknown` variant.
/// - `unknown`: a `Vec<T>` of the flag-like words no flag takes, whole. One per struct.
/// - `skip`: left at `Default`, never parsed.
///
/// # Flag names
///
/// - `short`, `short = 'x'`: `-x`; bare, the field name's first letter. A second `short` is another letter.
/// - `long`, `long = "name"`: `--name`; bare, the field name in kebab-case. A second `long` is another name.
/// - `alias = "…"`, `alias("…", …)`: more long names, not shown in help.
/// - `negate`, `negate = "no-name"`: `--no-name` sets a `bool` false; bare, `--no-<long>`.
/// - `two_dashes`: under `long_only`, the long name is never spelled with one dash (`--omagic`).
/// - `prefix`: an ASCII `short` that always takes the rest of its word (`-lfoo`); a value flag.
/// - `plus = 'c'`: also spelled `+c` in a `plus_options` struct; on an `Option<bool>` with `short = 'c'`,
///   `-c` is `Some(true)` and `+c` `Some(false)`.
/// - `global`: also accepted after a subcommand word, at any depth.
///
/// # Values
///
/// Field types are `bool` (a switch), an integer with `count`, `T`, `Option<T>` and `Vec<T>`, `T: FromArg`.
///
/// - `count`: an integer counting occurrences.
/// - `value_name = "…"`: the placeholder in help.
/// - `delimiter = ','`: split each value of a `Vec` field.
/// - `values = N`: each occurrence of a `Vec` flag takes `N` words, whatever they look like.
/// - `choices("a", "b")`: the only values accepted.
/// - `env = "VAR"`, `default = "…"`: fallbacks after the command line, in that order.
/// - `default_missing = "…"`: the value of a flag given without one; `value_optional` is accepted with it, and needs it.
/// - `require_equals`: the value only binds attached (`--name=v`, `-nv`).
/// - `keep_equals`: a short flag's attached value keeps a leading `=` (`-L=dir`).
/// - `allow_hyphen_values`: the next word is the value, flag-like or `--` included.
/// - `allow_negative_numbers`: a negative number is a value, for a flag or a positional.
/// - `keywords`: on a flag of a type deriving `Args`, each value is `--value` of that type (`-z now`).
///
/// # Positionals
///
/// - `double_dash = "…"`: how `--` treats it: `"optional"` (default), `"required"` (only words after it),
///   `"automatic"` (flags stop once it has a value), `"preserve"` (a `--` reaching it is a value).
/// - `stop_flags`: once it has a value, flags stop, whatever its `double_dash`.
///
/// # Rules
///
/// Selectors name another field: `"--long"`, `"-s"`, a positional's name or the field's name.
///
/// - `required`: must end with a value.
/// - `required_unless("…", …)`: required unless one of them has a value.
/// - `conflicts("…", …)`: not supplied together with these.
/// - `overrides("…", …)`: the last of this and these supplied wins.
/// - `requires("…", …)`: these must have a value when this is supplied.
/// - `group = "name"`: a member of a struct-level group.
///
/// # Help
///
/// - `help = "…"`, `long_help = "…"`: replace the doc comment's first paragraph and whole text.
/// - `help_heading = "…"`: list it under that heading instead of "Options".
/// - `hide`: leave it out of help and completion.
#[proc_macro_derive(Args, attributes(arg, winnow_args))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive `Subcommand` (and `Args`, so the enum can be the whole command line) for an enum of subcommands.
///
/// A variant is a unit, or holds one `Args` type, which may be `Box`ed; a parent struct holds the enum in
/// a `subcommand` field. The doc comments are the help text.
///
/// - Enum: `rename_all = "…"`: the case of variant names, one of `"kebab-case"` (default), `"lowercase"`,
///   `"UPPERCASE"`, `"snake_case"` and `"verbatim"`.
/// - Variant: `name = "…"`: the word that selects it.
/// - Variant: `alias = "…"`, `alias("…", …)`: other words, shown in help; `alias_hidden` is the same, not shown.
/// - Variant: `hide`: leave it out of the list in help and completion.
/// - Variant: `help = "…"`: the one-line description; `long_help = "…"` is accepted and unused in the list.
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
                // The whole doc comment, when the description is its first paragraph.
                let (doc, long_doc) = docs(&variant.attrs);
                let long_about = text(if doc == info.about {
                    &long_doc
                } else {
                    &info.about
                });
                let about = text(&info.about);
                quote!(&::winnow_args::help::Command {
                    name: "",
                    about: #about,
                    long_about: #long_about,
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
            Fields::Unit => {
                let index = subs.len() - 1;
                quote! {
                    __wa::finish_with(
                        __input,
                        __globals,
                        <Self as ::winnow_args::Args>::HELP.subcommands[#index].command,
                    )
                    .map(|()| Self::#ident)
                }
            }
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
                // Inherited global flags may come before the subcommand's name.
                loop {
                    if __input.is_empty() {
                        return ::core::result::Result::Err(
                            __wa::Error::missing_subcommand(__input.offset()),
                        );
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
                        __wa::Arg::Long(_) | __wa::Arg::Short(_)
                            if __globals.bind(&__arg, __input)? =>
                        {
                            continue;
                        }
                        __wa::Arg::Long(__flag) if __flag.name == b"help" => {
                            return ::core::result::Result::Err(__wa::Error::help(__help, true));
                        }
                        __wa::Arg::Short(__flag) if __flag.letter == 'h' => {
                            return ::core::result::Result::Err(__wa::Error::help(__help, false));
                        }
                        _ => {}
                    }
                    return ::core::result::Result::Err(__arg.unexpected());
                }
            }
        }
    })
}

/// Derive `Occurrence` for an enum: one variant per flag or positional, kept
/// in command-line order by an `#[arg(sequence)]` field of an `Args` struct.
///
/// A variant is a unit (a switch) or holds one value type; a `Spanned<T>` also records the offset and
/// whether the value was attached. A variant with no `short` or `long` is `--variant-name`. Doc comments
/// are the help text. A spelling used twice, here or in the parent, is a compile error.
///
/// Enum options, the default of every value variant:
///
/// - `allow_hyphen_values`, `keep_equals`: as for a field.
///
/// Variant names:
///
/// - `short = 'x'`, `long`, `long = "name"`: as for a field; a second `long` is another name.
/// - `alias = "…"`, `alias("…", …)`: more long names.
/// - `two_dashes`, `prefix`: as for a field.
///
/// Variant values, as for a field:
///
/// - `value_name = "…"`, `require_equals`, `keep_equals`, `allow_hyphen_values`, `allow_negative_numbers`,
///   `default_missing = "…"`.
///
/// Variant roles, at most one of each in an enum:
///
/// - `positional`: the variant takes every word; its `value_name` defaults to the variant's name.
/// - `unknown`: takes a flag no variant names, whole, for a parent with `sequence, unknown`.
/// - `bundle`: takes a word of several short flags (`-sS`), whole, ahead of its letters.
/// - `skip`: built by the program, never parsed.
#[proc_macro_derive(Occurrence, attributes(arg, winnow_args))]
pub fn derive_occurrence(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_occurrence(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive `FromArg` for an enum of unit variants, matched on the value's bytes; help lists them as possible values.
///
/// - Enum: `rename_all = "…"`: the case of variant names, one of `"kebab-case"` (default), `"lowercase"`,
///   `"UPPERCASE"`, `"snake_case"` and `"verbatim"`.
/// - Variant: `name = "…"`: the value that selects it.
/// - Variant: `alias = "…"`, `alias("…", …)`: other accepted values; `alias_hidden` is the same, not listed.
/// - Variant: `hide`: accepted, but not listed as possible.
/// - Variant: `help = "…"`, `long_help = "…"`: accepted and unused.
#[proc_macro_derive(ValueEnum, attributes(arg, winnow_args))]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_value_enum(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

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
    let (mut word, mut unknown, mut bundle) = (None, None, None);
    let mut seen = std::collections::HashMap::new();
    // What help lists: one row a variant, in declaration order.
    let mut help_items = Vec::new();
    // `allow_hyphen_values` and `keep_equals` on the enum: every value
    // variant's default.
    let (mut enum_hyphen_values, mut enum_keep_equals) = (false, false);
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("allow_hyphen_values") {
                enum_hyphen_values = true;
                Ok(())
            } else if meta.path.is_ident("keep_equals") {
                enum_keep_equals = true;
                Ok(())
            } else {
                Err(meta.error("expected `allow_hyphen_values` or `keep_equals`"))
            }
        })?;
    }
    for variant in &data.variants {
        let ident = &variant.ident;
        let (mut short, mut longs, mut positional) = (None, Vec::new(), false);
        let (mut two_dashes, mut prefix, mut value_name) = (false, false, None);
        let (mut hyphen_values, mut negative_numbers) = (enum_hyphen_values, false);
        let (mut require_equals, mut default_missing) = (false, None);
        let mut keep_equals = enum_keep_equals;
        let (mut skip, mut is_unknown, mut is_bundle) = (false, false, false);
        for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("allow_hyphen_values") {
                    hyphen_values = true;
                } else if meta.path.is_ident("allow_negative_numbers") {
                    negative_numbers = true;
                } else if meta.path.is_ident("require_equals") {
                    require_equals = true;
                } else if meta.path.is_ident("keep_equals") {
                    keep_equals = true;
                } else if meta.path.is_ident("default_missing") {
                    default_missing = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("bundle") {
                    is_bundle = true;
                } else if meta.path.is_ident("unknown") {
                    is_unknown = true;
                } else if meta.path.is_ident("short") {
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
                        "expected `short`, `long`, `alias`, `positional`, `two_dashes`, `prefix`, \
                         `value_name`, `allow_hyphen_values`, `allow_negative_numbers`, \
                         `require_equals`, `keep_equals`, `default_missing`, `skip`, `unknown` or `bundle`",
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
        if skip {
            // Built by the program itself (from a `-z` keyword, say), never parsed.
            continue;
        }
        // A `Spanned<T>` value also records where it was and whether it was attached.
        let spanned = takes.and_then(|ty| {
            last_segment(ty)
                .filter(|s| s.ident == "Spanned")
                .and_then(inner)
        });
        let wrap = |value: TokenStream2, offset: TokenStream2, attached: TokenStream2| {
            if spanned.is_some() {
                quote! {
                    ::winnow_args::value::Spanned {
                        value: #value,
                        offset: #offset,
                        attached: #attached,
                    }
                }
            } else {
                value
            }
        };
        if positional || is_unknown || is_bundle {
            let Some(ty) = takes else {
                return Err(syn::Error::new(
                    variant.span(),
                    "a positional, `unknown` or `bundle` variant holds its value",
                ));
            };
            let ty = spanned.unwrap_or(ty);
            let slot = if positional {
                &mut word
            } else if is_unknown {
                &mut unknown
            } else {
                &mut bundle
            };
            if slot.is_some() {
                return Err(syn::Error::new(
                    variant.span(),
                    "at most one positional, one `unknown` and one `bundle` variant",
                ));
            }
            let name = value_name.unwrap_or_else(|| ident.to_string().to_uppercase());
            if positional {
                help_items.push(occurrence_item(
                    variant,
                    None,
                    &[],
                    Some(&name),
                    quote!(<#ty as ::winnow_args::FromArg>::CHOICES),
                    false,
                ));
            }
            let value = wrap(
                quote!(__word.convert::<#ty>(#name)?),
                quote!(__word.offset),
                quote!(false),
            );
            *slot = Some(quote! {
                ::core::result::Result::Ok(::core::option::Option::Some(Self::#ident(#value)))
            });
            continue;
        }
        if short.is_none() && longs.is_empty() {
            longs.push(kebab_case(&ident.to_string()));
        }
        let spelled = short
            .map(|c| format!("-{c}"))
            .into_iter()
            .chain(longs.iter().map(|l| format!("--{l}")));
        for name in spelled {
            if let Some(other) = seen.insert(name.clone(), ident.clone()) {
                return Err(syn::Error::new(
                    variant.span(),
                    format!("`{name}` is already used by `{other}`"),
                ));
            }
        }
        let choices = match takes {
            Some(ty) => {
                let ty = spanned.unwrap_or(ty);
                quote!(<#ty as ::winnow_args::FromArg>::CHOICES)
            }
            None => quote!(&[]),
        };
        help_items.push(occurrence_item(
            variant,
            short,
            &longs,
            takes
                .is_some()
                .then(|| value_name.as_deref().unwrap_or("VALUE")),
            choices,
            require_equals,
        ));
        let body = match takes {
            None => quote! {
                __arg.check_switch()?;
                ::core::result::Result::Ok(::core::option::Option::Some(Self::#ident))
            },
            Some(ty) => {
                let ty = spanned.unwrap_or(ty);
                let options = quote! {
                    ::winnow_args::token::ValueOptions {
                        negative_numbers: #negative_numbers,
                        hyphen_values: #hyphen_values,
                        require_equals: #require_equals,
                        keep_equals: #keep_equals,
                    }
                };
                let read = match &default_missing {
                    Some(missing) => {
                        let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                        quote! {
                            __arg.read_value_or_with(
                                __input,
                                #options,
                                ::winnow_args::__private::BStr::new(#bytes),
                            )
                        }
                    }
                    None => quote!(__arg.read_value_with(__input, #options)?),
                };
                let value = wrap(
                    quote!(__arg.convert::<#ty>(__value)?),
                    quote!(__arg.offset()),
                    quote!(__attached),
                );
                let attached = spanned
                    .is_some()
                    .then(|| quote!(let __attached = __arg.has_attached_value(__input);));
                quote! {
                    #attached
                    let __value = #read;
                    ::core::result::Result::Ok(::core::option::Option::Some(Self::#ident(#value)))
                }
            }
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
    let has_positional = word.is_some();
    let word =
        word.unwrap_or_else(|| quote!(::core::result::Result::Ok(::core::option::Option::None)));
    let bundle = bundle.map(|body| {
        quote! {
            const BUNDLES: bool = true;

            fn from_bundle(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                #body
            }
        }
    });
    let unknown = unknown.map(|body| {
        quote! {
            fn from_unknown(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                #body
            }
        }
    });
    let is_long = matches_name(&single_dash);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::winnow_args::Occurrence for #name #ty_generics #where_clause {
            const PREFIXES: &'static [u8] = &[#(#prefixes),*];

            const POSITIONAL: bool = #has_positional;

            const ITEMS: &'static [::winnow_args::help::Item] = &[#(#help_items),*];

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

            #unknown

            #bundle

            fn short(__letter: char) -> ::core::option::Option<bool> {
                match __letter {
                    #(#letters)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    })
}

/// An `Occurrence` variant's row in help: a flag by `short` and `longs`, or
/// the positional when it has neither.
fn occurrence_item(
    variant: &syn::Variant,
    short: Option<char>,
    longs: &[String],
    value_name: Option<&str>,
    choices: TokenStream2,
    require_equals: bool,
) -> TokenStream2 {
    let (doc, long_doc) = docs(&variant.attrs);
    let (help, long_help) = (text(&doc), text(&long_doc));
    let positional = short.is_none() && longs.is_empty();
    let short = opt_char(short);
    let long = opt_str(longs.first().map(String::as_str));
    let aliases = longs.iter().skip(1);
    let value_name = opt_str(value_name);
    quote!(::winnow_args::help::Item {
        short: #short,
        long: #long,
        aliases: &[#(#aliases),*],
        negate: ::core::option::Option::None,
        value_name: #value_name,
        help: #help,
        long_help: #long_help,
        heading: ::core::option::Option::None,
        hide: false,
        positional: #positional,
        required: false,
        multiple: #positional,
        trailing: false,
        default: ::core::option::Option::None,
        env: ::core::option::Option::None,
        choices: #choices,
        require_equals: #require_equals,
        global: false,
    })
}

/// `FromArg` as one `match` on the value's bytes.
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

/// A variant's spellings, checked against `seen` for duplicates, and its help.
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

/// `Some("s")` or `None`, spelled out.
fn opt_str(s: Option<&str>) -> TokenStream2 {
    match s {
        Some(s) => quote!(::core::option::Option::Some(#s)),
        None => quote!(::core::option::Option::None),
    }
}

/// `Some('c')` or `None`, spelled out.
fn opt_char(c: Option<char>) -> TokenStream2 {
    match c {
        Some(c) => quote!(::core::option::Option::Some(#c)),
        None => quote!(::core::option::Option::None),
    }
}

/// `b"a" | b"b"`.
fn byte_patterns<S: AsRef<str>>(names: &[S]) -> TokenStream2 {
    let literals = names
        .iter()
        .map(|n| LitByteStr::new(n.as_ref().as_bytes(), Span::call_site()));
    quote!(#(#literals)|*)
}

/// Whether `__name` is one of `names`: `matches!(__name, b"a" | b"b")`.
fn matches_name(names: &[String]) -> TokenStream2 {
    if names.is_empty() {
        quote!(false)
    } else {
        let names = byte_patterns(names);
        quote!(matches!(__name, #names))
    }
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
    has_flag(f, "unknown") && !is_sequence(f)
}

/// Whether a field is `#[arg(skip)]`: left at its default, not parsed.
fn is_skipped(f: &syn::Field) -> bool {
    has_flag(f, "skip")
}

/// Whether one of the field's attributes is the bare word `name`.
fn has_flag(f: &syn::Field, name: &str) -> bool {
    f.attrs.iter().filter(|a| is_ours(a)).any(|a| {
        let mut found = false;
        let _ = a.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) {
                found = true;
            } else if meta.input.peek(syn::Token![=]) {
                meta.value()?.parse::<syn::Expr>()?;
            } else if meta.input.peek(syn::token::Paren) {
                let _content;
                syn::parenthesized!(_content in meta.input);
            }
            Ok(())
        });
        found
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
    /// A short option's attached value keeps a leading `=` (GNU ld's `-L=dir`).
    keep_equals: bool,
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
}

impl Field {
    /// This field's entry in `help::Command::items`.
    fn help_item(&self) -> TokenStream2 {
        let short = opt_char(self.short());
        let long = opt_str(self.longs().first().copied());
        let aliases = self.longs().into_iter().skip(1);
        let negate = opt_str(self.negate.as_deref());
        let require_equals = self.require_equals;
        let global = self.is_global();
        let positional = self.is_positional();
        let trailing = self.double_dash() == DoubleDash::Required;
        let display = self.display();
        let value_name = if positional {
            opt_str(Some(&display))
        } else {
            opt_str(self.value_name.as_deref())
        };
        let (help, long_help) = (text(&self.help), text(&self.long_help));
        let heading = opt_str(self.heading.as_deref());
        // A field spelled only `+c` has no `-`/`--` row to show.
        let hide = self.hide
            || matches!(self.role, Role::Flag { .. })
                && self.short().is_none()
                && self.longs().is_empty();
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
                aliases: &[#(#aliases),*],
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
                require_equals: #require_equals,
                global: #global,
            }
        }
    }

    /// Reading this flag's value: `read_value`, or `read_value_or` its `default_missing`.
    fn read(&self) -> TokenStream2 {
        let (negative_numbers, hyphen_values, require_equals, keep_equals) = (
            self.negative_numbers,
            self.hyphen_values,
            self.require_equals,
            self.keep_equals,
        );
        let options = quote! {
            __wa::ValueOptions {
                negative_numbers: #negative_numbers,
                hyphen_values: #hyphen_values,
                require_equals: #require_equals,
                keep_equals: #keep_equals,
            }
        };
        match &self.default_missing {
            None if !negative_numbers && !hyphen_values && !require_equals && !keep_equals => {
                quote!(__arg.read_value(__input)?)
            }
            None => quote!(__arg.read_value_with(__input, #options)?),
            Some(missing) => {
                let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                quote!(__arg.read_value_or_with(__input, #options, __wa::BStr::new(#bytes)))
            }
        }
    }

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
            let displaced = displaced_flag(&self.ident);
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
        (!longs.is_empty()).then(|| byte_patterns(&longs))
    }

    /// Whether a flag reads a value (rather than being a switch or a count).
    fn takes_value(&self) -> bool {
        !self.tristate && !matches!(self.kind, Kind::Switch | Kind::Count(_))
    }

    fn is_positional(&self) -> bool {
        matches!(self.role, Role::Positional { .. })
    }

    fn is_global(&self) -> bool {
        matches!(self.role, Role::Flag { global: true, .. })
    }

    /// A positional's relation to `--`; `Optional` for anything else.
    fn double_dash(&self) -> DoubleDash {
        match self.role {
            Role::Positional { double_dash, .. } => double_dash,
            _ => DoubleDash::Optional,
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
    // `#[arg(sequence, unknown)]`: unknown flags join the sequence, through
    // the enum's `#[arg(unknown)]` variant.
    let sequence_unknown = sequences.first().is_some_and(|f| has_flag(f, "unknown"));
    let sequence = vec_field(
        sequences.first().copied(),
        "a `sequence` field is a `Vec<T>` of an `Occurrence` enum",
    )?;
    let sequence_ty = sequence.as_ref().map(|(_, ty)| ty);
    // `#[arg(unknown)] unknown: Vec<T>`: flag-like words naming no flag.
    let unknowns: Vec<&syn::Field> = named.named.iter().filter(|f| is_unknown(f)).collect();
    if let Some(extra) = unknowns.get(1) {
        return Err(syn::Error::new(
            extra.span(),
            "a struct has at most one `unknown` field",
        ));
    }
    let unknown = vec_field(
        unknowns.first().copied(),
        "an `unknown` field is a `Vec<T>`",
    )?;
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
    // A `double_dash = "required"` positional is outside the ordinary sequence:
    // every word after `--` goes to it, and no word before.
    let positionals: Vec<&Field> = fields
        .iter()
        .filter(|f| f.is_positional() && f.double_dash() != DoubleDash::Required)
        .collect();
    check_positional_order(&positionals)?;
    let trailing: Vec<&Field> = fields
        .iter()
        .filter(|f| f.double_dash() == DoubleDash::Required)
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
        .filter(|(_, f)| f.double_dash() == DoubleDash::Preserve)
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

    // Each field's slot: its name, type and starting value.
    let slot_parts: Vec<(Ident, TokenStream2, TokenStream2)> = fields
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
            (slot(&f.ident), ty, init)
        })
        .collect();
    let slots = slot_parts
        .iter()
        .map(|(ident, ty, init)| quote!(let mut #ident: #ty = #init;));

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
            _ if f.tristate || f.negate.is_some() => quote! {
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
    // A `+c` occurrence: a tristate's `Some(false)`, else what `-c` stores.
    let plus = |f: &Field| {
        let letter = LitChar::new(f.plus?, Span::call_site());
        let body = if f.tristate {
            let ident = slot(&f.ident);
            let displace = rules.displace(&fields, f);
            quote!(#ident = ::core::option::Option::Some(false); #displace)
        } else {
            store(f)
        };
        Some((quote!(#letter), body))
    };
    // The long (negations last), short and `+` arms of the fields `keep`
    // selects, each body ending in `tail`.
    type Part<'a> = &'a dyn Fn(&Field) -> Option<(TokenStream2, TokenStream2)>;
    let arms = |keep: &dyn Fn(&Field) -> bool, tail: TokenStream2| {
        let of = |part: Part<'_>| -> Vec<TokenStream2> {
            fields
                .iter()
                .filter(|f| keep(f))
                .filter_map(part)
                .map(|(pattern, body)| quote!(#pattern => { #body #tail }))
                .collect()
        };
        let mut long = of(&|f| Some((f.long_pattern()?, store(f))));
        long.extend(of(&negated));
        let short = of(&|f| Some((f.short_pattern()?, store(f))));
        (long, short, of(&plus))
    };
    let (long_arms, short_arms, plus_arms) = arms(&|_| true, quote!());

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
            let stop = (f.double_dash() == DoubleDash::Automatic || f.stop_flags)
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
                            // Sized once, for at most every word left: the
                            // usual tail of operands then needs no regrowth.
                            quote! {
                                if #ident.capacity() == 0 {
                                    #ident.reserve_exact(__input.words_left() + 1);
                                }
                                #ident.push(#value);
                            }
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
        let ty = subcommand_type(f);
        let not_filled = track_filled.then(|| quote!(!__filled &&));
        let (global_longs, global_shorts, global_plus) = arms(
            &Field::is_global,
            quote!(return ::core::result::Result::Ok(true);),
        );
        // The subcommand sees this struct's globals first, then its ancestors'.
        let (inherit, handler) = if fields.iter().any(Field::is_global) {
            let shorts = fields.iter().filter(|f| f.is_global()).flat_map(|f| {
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
                        __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                            #(#global_plus)*
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
    let sequence_word = sequence_ty.map(|ty| {
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
            // Room for one item a word: a linker's line is mostly inputs.
            quote! {
                let mut __sequence: ::std::vec::Vec<#ty> =
                    ::std::vec::Vec::with_capacity(__input.words_left());
            },
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
    // A word of several short flags (`-sS`), where the enum wants to know:
    // once its first letter is read, whoever took it, the `bundle` item goes
    // before the items of its letters.
    let bundle_start = sequence_ty.map(|_| {
        quote! {
            let __bundle_word = __input.front();
            let __bundle_start = __input.mode() == ::winnow_args::stream::Mode::Word;
            let __bundle_at = __sequence.len();
        }
    });
    let bundle_end = sequence_ty.map(|ty| {
        quote! {
            if <#ty as ::winnow_args::Occurrence>::BUNDLES
                && __bundle_start
                && matches!(__arg, __wa::Arg::Short(__flag) if !__flag.plus)
                && __input.mode() == ::winnow_args::stream::Mode::Bundle
            {
                let __word = __wa::Word {
                    value: __wa::BStr::new(__bundle_word),
                    offset: __arg.offset(),
                    after_separator: false,
                };
                if let ::core::option::Option::Some(__bundle) =
                    <#ty as ::winnow_args::Occurrence>::from_bundle(&__word)?
                {
                    __sequence.insert(__bundle_at, __bundle);
                }
            }
        }
    });
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
    let flatten_shorts: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| {
            quote! {
                __c if <#ty as __wa::Flatten>::short(__c).is_some() => {
                    <#ty as __wa::Flatten>::short(__c)
                }
            }
        })
        .collect();
    let flatten_longs: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_long(__name)))
        .collect();
    // Whether each of our letters takes a value, as `short` answers.
    let own_shorts: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let letters = f.short_pattern()?;
            let takes_value = f.takes_value();
            Some(quote!(#letters => ::core::option::Option::Some(#takes_value),))
        })
        .collect();
    // The long names one dash may spell under `long_only`, and the letters
    // that never start one.
    let own_longs: Vec<String> = fields
        .iter()
        .filter(|f| !f.two_dashes)
        .flat_map(|f| spellings(f).into_iter().map(str::to_owned))
        .collect();
    let prefixes: Vec<u8> = fields
        .iter()
        .filter(|f| f.prefix)
        .filter_map(|f| f.short().map(|c| c as u8))
        .collect();
    let flatten_prefixes = flattens.iter().map(|(_, ty)| {
        quote!(&& !__name.first().is_some_and(|__c| <#ty as __wa::Flatten>::is_prefix(*__c)))
    });
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
    let plus_match = plus_options.then(|| {
        quote! {
            __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                #(#plus_arms)*
                _ => #flatten_arg if !__globals.bind(&__arg, __input)? { #unexpected; }
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
    // The row help prints is `--help`'s: a struct with its own has none.
    let help_flag = help_long.is_some();
    let help_short_flag = help_short.is_some();
    // A flattened struct or a sequence enum may declare `-h`, `--help`, `-V`
    // or `--version` itself (ld's `-h SONAME`): the built-in ones are then
    // tried after them, not as arms before.
    let late = !flattens.is_empty() || sequence.is_some();
    let late_if = |on: bool, test: TokenStream2, error: TokenStream2| {
        (late && on).then(|| quote!(if #test { return ::core::result::Result::Err(#error); } else))
    };
    let late_long: TokenStream2 = [
        late_if(
            help_long.is_some(),
            quote!(__flag.name == b"help"),
            quote!(__wa::Error::help(#help, true)),
        ),
        late_if(
            version_long.is_some(),
            quote!(__flag.name == b"version"),
            quote!(__wa::Error::version(#help)),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let late_short: TokenStream2 = [
        late_if(
            help_short.is_some(),
            quote!(__flag.letter == 'h'),
            quote!(__wa::Error::help(#help, false)),
        ),
        late_if(
            version_short.is_some(),
            quote!(__flag.letter == 'V'),
            quote!(__wa::Error::version(#help)),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let keep = |arm: Option<TokenStream2>| arm.filter(|_| !late);
    let (help_long, help_short) = (keep(help_long), keep(help_short));
    let (version_long, version_short) = (keep(version_long), keep(version_short));

    // `long_only`: `-name` is `--name` for the long names that may take one
    // dash, tried before the word is read as short letters.
    // `unknown_flags = "value"`: a flag-like word naming no flag goes to the
    // positionals whole, never to a subcommand. A bundle of several letters is
    // checked before any binds; a long word or a single letter once no arm or
    // global took it.
    // An `unknown` field takes those words instead of the positionals.
    // Only the first letter of a word reaches the short arm's sink: the bundle
    // check vouched for the letters of a longer one.
    let (lenient_bundle, save_token, unknown_flag) = if unknown_flags_value
        || unknown.is_some()
        || sequence_unknown
    {
        // With nothing to take a word, `positional_match` always returns.
        let next = (!positionals.is_empty() || trailing.is_some()).then(|| quote!(continue;));
        let (sink, sink_next) = match (&unknown, sequence_ty.filter(|_| sequence_unknown)) {
            (_, Some(ty)) => (
                quote! {
                    match <#ty as ::winnow_args::Occurrence>::from_unknown(&__word)? {
                        ::core::option::Option::Some(__item) => __sequence.push(__item),
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(__wa::Error::unknown_flag(
                                __word.offset,
                                ::std::string::String::from_utf8_lossy(&__word.value),
                            ));
                        }
                    }
                },
                quote!(continue;),
            ),
            (Some((_, ty)), None) => (
                quote!(__unknown.push(__word.convert::<#ty>("<UNKNOWN>")?);),
                quote!(continue;),
            ),
            (None, None) => (
                quote! {
                    let __arg = __wa::Arg::Word(__word);
                    #positional_match
                },
                quote!(#next),
            ),
        };
        let theirs = match sequence_ty {
            Some(ty) => quote! {
                __c => match <#ty as ::winnow_args::Occurrence>::short(__c) {
                    ::core::option::Option::None => __globals.short(__c),
                    __known => __known,
                },
            },
            None => quote!(_ => __globals.short(__c),),
        };
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
                    #(#own_shorts)*
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
    let mut names = Vec::new();
    if long_only {
        names.clone_from(&own_longs);
        names.extend(help_long.is_some().then(|| "help".to_owned()));
        names.extend(version_long.is_some().then(|| "version".to_owned()));
    }
    let long_only_lex =
        long_only && (!names.is_empty() || sequence_ty.is_some() || !flattens.is_empty());
    let lex = if long_only_lex {
        let own = matches_name(&names);
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
        let not_prefixed = (!prefixes.is_empty()).then(
            || quote!(!matches!(__name.first(), ::core::option::Option::Some(#(#prefixes)|*)) &&),
        );
        quote! {
            match __wa::long_only(__input, |__name| {
                #not_prefixed (#own #their_names #(#flatten_longs)*) #their_prefixes #(#flatten_prefixes)*
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

    let help_version = match &version {
        Some(v) if with_version => quote!(::core::option::Option::Some(#v)),
        _ => quote!(::core::option::Option::None),
    };
    let help_items = fields
        .iter()
        .filter(|f| !matches!(f.role, Role::Subcommand))
        .map(|f| f.help_item());
    let (help_subcommands, subcommand_required) = match subcommand {
        Some(f) => {
            let ty = subcommand_type(f);
            (
                quote!(<#ty as ::winnow_args::Args>::HELP.subcommands),
                matches!(f.kind, Kind::Required(_)),
            )
        }
        None => (quote!(&[]), false),
    };
    let displaced = rules.displaced_slots(&fields);
    let env_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| f.fallback(true, rules.is_displaced(&fields, f)))
        .collect();
    let default_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| f.fallback(false, rules.is_displaced(&fields, f)))
        .collect();
    let exclusive = rules.exclusive(&fields, &groups);
    let supplied = rules.supplied(&fields);
    let required = rules.required(&fields, &groups);
    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        match &f.kind {
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
        }
    });
    let build: Vec<TokenStream2> = build.collect();
    // Every field read once after building, as clap's and usage's derives do:
    // a flag accepted and ignored on purpose (`pwd -L`) is the derive's field,
    // not dead code in the user's crate. No runtime work after optimization.
    let field_reads: Vec<TokenStream2> = fields
        .iter()
        .map(|f| &f.ident)
        .chain(skipped.iter().copied())
        .chain(sequence.iter().chain(&unknown).map(|(ident, _)| ident))
        .chain(flattens.iter().map(|(ident, _)| ident))
        .map(|ident| quote!(let _ = &__built.#ident;))
        .collect();
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
        && !fields.iter().any(|f| f.keywords || f.is_global());
    let flatten_impl = flattenable.then(|| {
        let vis = &input.vis;
        let slots_name = format_ident!("__{}WinnowArgsSlots", name);
        let displaced_idents: Vec<Ident> = fields
            .iter()
            .filter(|f| rules.is_displaced(&fields, f))
            .map(|f| displaced_flag(&f.ident))
            .collect();
        let nested: Vec<(Ident, &Type)> = flattens
            .iter()
            .map(|(ident, ty)| (flat_slot(ident), ty))
            .collect();
        let decls = slot_parts.iter().map(|(ident, ty, _)| quote!(#ident: #ty,));
        let inits = slot_parts
            .iter()
            .map(|(ident, _, init)| quote!(#ident: #init,));
        let all: Vec<&Ident> = slot_parts
            .iter()
            .map(|(ident, _, _)| ident)
            .chain(&displaced_idents)
            .chain(nested.iter().map(|(n, _)| n))
            .collect();
        let names: Vec<TokenStream2> = all.iter().map(|ident| quote!(#ident,)).collect();
        let mutable: Vec<TokenStream2> = all.iter().map(|ident| quote!(mut #ident,)).collect();
        let displaced_decls = displaced_idents.iter().map(|d| quote!(#d: bool,));
        let displaced_inits = displaced_idents.iter().map(|d| quote!(#d: false,));
        let nested_decls = nested
            .iter()
            .map(|(n, ty)| quote!(#n: <#ty as ::winnow_args::__private::Flatten>::Slots,));
        let nested_inits = nested
            .iter()
            .map(|(n, _)| quote!(#n: ::core::default::Default::default(),));
        let (bind_long, bind_short, bind_plus) = arms(&|_| true, quote!(true));
        let bind_nested: TokenStream2 = nested
            .iter()
            .map(|(n, ty)| {
                quote!(if <#ty as __wa::Flatten>::bind(&mut #n, &__arg, __input)? { true } else)
            })
            .collect();
        // A `+x` word is offered to the `+` arms only, here and below.
        let plus = if plus_options {
            quote! {
                __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                    #(#bind_plus)*
                    _ => #bind_nested { false }
                },
            }
        } else {
            quote!(__wa::Arg::Short(__flag) if __flag.plus => #bind_nested { false },)
        };
        let own_longs = matches_name(&own_longs);
        let own_prefixes = if prefixes.is_empty() {
            quote!(false)
        } else {
            quote!(matches!(__c, #(#prefixes)|*))
        };
        let nested_prefixes = nested
            .iter()
            .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_prefix(__c)));
        quote! {
            #[doc(hidden)]
            #[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
            #vis struct #slots_name {
                #(#decls)*
                #(#displaced_decls)*
                #(#nested_decls)*
            }

            #[automatically_derived]
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
            #[allow(
                unused_mut,
                unused_variables,
                unreachable_code,
                clippy::all,
                clippy::pedantic,
                clippy::nursery,
                clippy::restriction
            )]
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
                        #(#own_shorts)*
                        #(#flatten_shorts)*
                        _ => ::core::option::Option::None,
                    }
                }

                fn is_prefix(__c: u8) -> bool {
                    use ::winnow_args::__private as __wa;
                    #own_prefixes #(#nested_prefixes)*
                }

                fn is_long(__name: &[u8]) -> bool {
                    use ::winnow_args::__private as __wa;
                    #own_longs #(#flatten_longs)*
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
                    let __built = Self {
                        #(#build,)*
                        #(#skipped: ::core::default::Default::default(),)*
                        #(#flatten_build)*
                    };
                    #(#field_reads)*
                    ::core::result::Result::Ok(__built)
                }
            }
        }
    });
    let own_items = quote!(&[#(#help_items),*]);
    // A sequence enum with a positional variant takes every word.
    let positional_clash = sequence_ty
        .filter(|_| !positionals.is_empty() || trailing.is_some())
        .map(|ty| {
            quote! {
                const _: () = assert!(
                    !<#ty as ::winnow_args::Occurrence>::POSITIONAL,
                    "the sequence enum has a positional variant, which takes every word: this struct's positionals would never be filled",
                );
            }
        });
    // Ours, then the flattened structs', then the sequence enum's.
    let more: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| quote!(<#ty as ::winnow_args::Args>::HELP.items))
        .chain(
            sequence_ty
                .iter()
                .map(|ty| quote!(<#ty as ::winnow_args::Occurrence>::ITEMS)),
        )
        .collect();
    let items = if more.is_empty() {
        own_items
    } else {
        quote!({
            const __OWN: &[::winnow_args::help::Item] = #own_items;
            const _: () = assert!(
                !::winnow_args::help::items_clash(&[__OWN, #(#more),*]),
                "a flattened struct or the sequence enum declares a flag that this struct, or another of them, also declares",
            );
            #positional_clash
            const __N: usize = __OWN.len() #(+ #more.len())*;
            const __ALL: [::winnow_args::help::Item; __N] = ::winnow_args::__private::concat_items::<__N>(
                &[__OWN, #(#more),*],
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
                    #bundle_start
                    let __arg = #lex;
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#long_arms)*
                            #help_long
                            #version_long
                            _ => #sequence_arg #flatten_arg #late_long if !__globals.bind(&__arg, __input)? { #unknown_flag },
                        },
                        #plus_match
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#short_arms)*
                            #help_short
                            #version_short
                            _ => #sequence_arg #flatten_arg #late_short if !__globals.bind(&__arg, __input)? { #unknown_flag },
                        },
                        #separator_arm
                        #word_arm
                    }
                    #bundle_end
                }
                #(#env_fallbacks)*
                #exclusive
                #supplied
                #(#default_fallbacks)*
                #required
                let __built = Self {
                    #(#build,)*
                    #(#skipped: ::core::default::Default::default(),)*
                    #sequence_build
                    #unknown_build
                    #(#flatten_build)*
                };
                #(#field_reads)*
                ::core::result::Result::Ok(__built)
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

/// `#[arg(...)]` on the struct: options declared there rather than on a field.
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
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("restart_token") {
                options.restart_token = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand") {
                options.default_subcommand = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("arg_required_else_help") {
                options.arg_required_else_help = true;
                return Ok(());
            }
            if meta.path.is_ident("long_only") {
                options.long_only = true;
                return Ok(());
            }
            if meta.path.is_ident("plus_options") {
                options.plus_options = true;
                return Ok(());
            }
            if meta.path.is_ident("unknown_flags") {
                let mode = meta.value()?.parse::<LitStr>()?;
                options.unknown_flags_value = match mode.value().as_str() {
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
                options.name = text(&meta)?;
                return Ok(());
            }
            if meta.path.is_ident("version") {
                options.version = Some(if meta.input.peek(syn::Token![=]) {
                    let version = meta.value()?.parse::<LitStr>()?;
                    quote!(#version)
                } else {
                    quote!(::core::env!("CARGO_PKG_VERSION"))
                });
                return Ok(());
            }
            for (key, slot) in [
                ("about", &mut options.about),
                ("long_about", &mut options.long_about),
            ] {
                if meta.path.is_ident(key) {
                    *slot = Some(text(&meta)?);
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("after_help", &mut options.after_help),
                ("after_long_help", &mut options.after_long_help),
            ] {
                if meta.path.is_ident(key) {
                    *slot = text(&meta)?;
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("disable_help_flag", &mut options.disable_help_flag),
                ("disable_help_short", &mut options.disable_help_short),
                ("disable_version_flag", &mut options.disable_version_flag),
                (
                    "disable_help_subcommand",
                    &mut options.disable_help_subcommand,
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
            options.groups.push(Group {
                name,
                required,
                multiple,
            });
            Ok(())
        })?;
    }
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
                let displaced = displaced_flag(&f.ident);
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
        let standing = displaced_flag(&f.ident);
        let clears = self.beats[i].iter().map(|&j| {
            let other = &fields[j];
            let slot = slot(&other.ident);
            let displaced = displaced_flag(&other.ident);
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

/// A `Vec<T>` field's name and `T`; `shape` is the error for any other type.
fn vec_field(f: Option<&syn::Field>, shape: &str) -> syn::Result<Option<(Ident, Type)>> {
    let Some(f) = f else {
        return Ok(None);
    };
    let ty = last_segment(&f.ty)
        .filter(|s| s.ident == "Vec")
        .and_then(inner)
        .ok_or_else(|| syn::Error::new(f.ty.span(), shape))?;
    Ok(Some((f.ident.clone().expect("named field"), ty.clone())))
}

/// The enum of a subcommand field.
fn subcommand_type(f: &Field) -> &Type {
    match &f.kind {
        Kind::Optional(ty) | Kind::Required(ty) => ty,
        _ => unreachable!("rejected for subcommands in `field`"),
    }
}

fn displaced_flag(ident: &Ident) -> Ident {
    format_ident!("__displaced_{}", ident)
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
    let mut keep_equals = false;
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
            } else if meta.path.is_ident("keep_equals") {
                keep_equals = true;
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
                     `default_missing`, `value_optional`, `allow_negative_numbers`, `allow_hyphen_values`, `require_equals`, `keep_equals`, `negate`, `two_dashes`, `prefix`, `values`, `stop_flags`, `plus`, `skip`, `keywords`, \
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
    Ok(Field {
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
        keep_equals,
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
    if is_bool(ty) {
        return Kind::Switch;
    }
    if let Some(last) = last_segment(ty) {
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
