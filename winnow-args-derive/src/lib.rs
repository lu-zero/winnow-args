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
    let mut subs = Vec::new();
    for variant in &data.variants {
        let ident = &variant.ident;
        let info = variant_names(variant, &mut names)?;
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
                    version: ::core::option::Option::None,
                })
            }
        };
        let about = match (&inner, info.about.is_empty()) {
            (Some(ty), true) => quote!(<#ty as ::winnow_args::Args>::HELP.about),
            _ => {
                let about = &info.about;
                quote!(#about)
            }
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

#[proc_macro_derive(ValueEnum, attributes(arg))]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_value_enum(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// `FromArg` as one `match` on the value's bytes.
fn expand_value_enum(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`ValueEnum` can only be derived for enums",
        ));
    };
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let mut choices = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "a `ValueEnum` variant holds no data",
            ));
        }
        let spellings = variant_names(variant, &mut names)?.names;
        choices.push(LitStr::new(&spellings[0], Span::call_site()));
        let pattern = byte_patterns(&spellings);
        let ident = &variant.ident;
        arms.push(quote!(#pattern => ::core::result::Result::Ok(Self::#ident),));
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::winnow_args::FromArg for #name #ty_generics #where_clause {
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
fn variant_names<'a>(
    variant: &'a syn::Variant,
    seen: &mut Vec<(String, &'a Ident)>,
) -> syn::Result<Variant> {
    let (mut name, mut alias, mut hidden, mut hide) = (None, Vec::new(), Vec::new(), false);
    for attr in variant.attrs.iter().filter(|a| a.path().is_ident("arg")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("alias") {
                alias.extend(aliases(&meta)?);
            } else if meta.path.is_ident("alias_hidden") {
                hidden.extend(aliases(&meta)?);
            } else if meta.path.is_ident("hide") {
                hide = true;
            } else {
                return Err(meta.error("expected `name`, `alias`, `alias_hidden` or `hide`"));
            }
            Ok(())
        })?;
    }
    let ident = &variant.ident;
    let shown = alias.len();
    let names: Vec<String> =
        std::iter::once(name.unwrap_or_else(|| kebab_case(&ident.to_string())))
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
        about: docs(&variant.attrs).0,
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
        let (help, long_help) = (&self.help, &self.long_help);
        let heading = opt_str(self.heading.as_deref());
        let hide = self.hide;
        let required =
            matches!(self.kind, Kind::Required(_)) && self.default.is_none() && self.env.is_none()
                || self.required;
        let multiple = matches!(self.kind, Kind::Many(_) | Kind::Count(_));
        let default = opt_str(self.default.as_deref());
        let env = opt_str(self.env.as_deref());
        let choices = self.choices.iter().flatten();
        quote! {
            ::winnow_args::help::Item {
                short: #short,
                long: #long,
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
                choices: &[#(#choices),*],
            }
        }
    }

    /// Reading this flag's value: `read_value`, or `read_value_or` its `default_missing`.
    fn read(&self) -> TokenStream2 {
        match &self.default_missing {
            None => quote!(__arg.read_value(__input)?),
            Some(missing) => {
                let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                quote!(__arg.read_value_or(__input, __wa::BStr::new(#bytes)))
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
        match &self.kind {
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
            Role::Flag { .. } => unreachable!("every flag has a name"),
            Role::Positional { name, .. } => name.clone(),
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
    if let (Some(t), Some(_)) = (
        trailing,
        fields.iter().find(|f| dd(f) == DoubleDash::Automatic),
    ) {
        return Err(syn::Error::new(
            t.ident.span(),
            "`double_dash = \"required\"` and `\"automatic\"` cannot share a struct",
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
        disable_version_flag,
        disable_help_subcommand,
    } = struct_options(input)?;
    let (doc_about, doc_long_about) = docs(&input.attrs);
    let about = about.unwrap_or(doc_about);
    let long_about = long_about.unwrap_or(doc_long_about);
    let rules = Rules::new(&fields, &groups)?;

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
        let displace = rules.displace(&fields, f);
        let stored = match &f.kind {
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
                    quote! {
                        let __value = #read;
                        #ident.push(#value);
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
            let stop = (dd(f) == DoubleDash::Automatic).then(|| quote!(__input.stop_flags();));
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
    let word_arm = quote! {
        __wa::Arg::Word(__word) => {
            #restart
            #route
            #positional_match
        }
    };
    let position = (!positionals.is_empty()).then(|| quote!(let mut __position: usize = 0;));
    let filled_slot = track_filled.then(|| quote!(let mut __filled = false;));

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
    let declares_short = |c: char| fields.iter().any(|f| f.short() == Some(c));
    let help = quote!(<Self as ::winnow_args::Args>::HELP);
    let help_long = (!disable_help_flag && !declares_long("help")).then(
        || quote!(b"help" => return ::core::result::Result::Err(__wa::Error::help(#help, true)),),
    );
    let help_short = (!disable_help_flag && !declares_short('h')).then(
        || quote!('h' => return ::core::result::Result::Err(__wa::Error::help(#help, false)),),
    );
    let with_version = version.is_some() && !disable_version_flag;
    let version_long = (with_version && !declares_long("version")).then(
        || quote!(b"version" => return ::core::result::Result::Err(__wa::Error::version(#help)),),
    );
    let version_short = (with_version && !declares_short('V'))
        .then(|| quote!('V' => return ::core::result::Result::Err(__wa::Error::version(#help)),));
    let help_flag = help_long.is_some() || help_short.is_some();
    let help_version = match &version {
        Some(v) if with_version => quote!(::core::option::Option::Some(#v)),
        _ => quote!(::core::option::Option::None),
    };
    let help_items = fields
        .iter()
        .filter(|f| !matches!(f.role, Role::Subcommand))
        .map(Field::help_item);
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
    let env_fallbacks = fields
        .iter()
        .map(|f| f.fallback(true, rules.is_displaced(&fields, f)));
    let default_fallbacks = fields
        .iter()
        .map(|f| f.fallback(false, rules.is_displaced(&fields, f)));
    let exclusive = rules.exclusive(&fields, &groups);
    let supplied = rules.supplied(&fields);
    let required = rules.required(&fields, &groups);
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
            const HELP: &'static ::winnow_args::help::Command = &::winnow_args::help::Command {
                name: #program_name,
                about: #about,
                long_about: #long_about,
                after_help: #after_help,
                after_long_help: #after_long_help,
                items: &[#(#help_items),*],
                subcommands: #help_subcommands,
                subcommand_required: #subcommand_required,
                help_flag: #help_flag,
                version: #help_version,
            };

            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                #(#slots)*
                #(#displaced)*
                #position
                #filled_slot
                #help_on_empty
                while !__input.is_empty() {
                    #start
                    let __arg = __wa::arg(__input)?;
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#long_arms)*
                            #help_long
                            #version_long
                            _ => if !__globals.bind(&__arg, __input)? { #unexpected; },
                        },
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#short_arms)*
                            #help_short
                            #version_short
                            _ => if !__globals.bind(&__arg, __input)? { #unexpected; },
                        },
                        __wa::Arg::Separator { .. } => {}
                        #word_arm
                    }
                }
                #(#env_fallbacks)*
                #exclusive
                #supplied
                #(#default_fallbacks)*
                #required
                ::core::result::Result::Ok(Self { #(#build),* })
            }
        }
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
    disable_version_flag: bool,
    disable_help_subcommand: bool,
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
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("arg")) {
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
                     `arg_required_else_help`, `name`, `version`, `about`, `long_about`, \
                     `after_help`, `after_long_help` or a `disable_*` option",
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
    options.disable_version_flag = texts.disable_version_flag;
    options.disable_help_subcommand = texts.disable_help_subcommand;
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
    let (doc_help, doc_long_help) = docs(&f.attrs);
    let (mut help, mut long_help, mut heading, mut hide) = (None, None, None, false);
    let (mut conflicts, mut overrides, mut requires) = (Vec::new(), Vec::new(), Vec::new());
    let (mut required, mut required_unless, mut group) = (false, Vec::new(), None);
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
            } else if meta.path.is_ident("help") {
                help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("long_help") {
                long_help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("help_heading") {
                heading = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("hide") {
                hide = true;
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
                    _ => {
                        return Err(syn::Error::new(
                            mode.span(),
                            "expected \"required\", \"automatic\" or \"optional\"",
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
                     `default_missing`, `value_optional`, \
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
    if double_dash.is_some() && !positional {
        return error("`double_dash` is for positional fields".into());
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

    if choices.is_some()
        && (matches!(kind, Kind::Switch | Kind::Count(_)) || matches!(role, Role::Subcommand))
    {
        return error("`choices` needs a field that takes a value".into());
    }
    if (env.is_some() || default.is_some()) && matches!(role, Role::Subcommand) {
        return error("a subcommand field takes no `env` or `default`".into());
    }
    if default.is_some() && matches!(kind, Kind::Switch | Kind::Count(_)) {
        return error("`default` needs a field that takes a value".into());
    }
    if required && !matches!(kind, Kind::Optional(_) | Kind::Many(_)) {
        return error("`required` is for `Option` and `Vec` fields; a `T` field already is".into());
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
        value_name: flag_value_name,
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
