import re, sys
sys.path.insert(0, sys.argv[3])
sys.argv_saved = sys.argv
src_path, out_dir = sys.argv[1], sys.argv[2]
sys.argv = [sys.argv[0], src_path]
import moldgen as G

arms = G.arms
NUL = "\\0"

LTO_FLAGS = [
    ("--lto-cs-profile-generate", "cs-profile-generate"),
    ("--lto-debug-pass-manager", "debug-pass-manager"),
    ("disable-verify", "disable-verify"),
    ("--lto-emit-asm", "emit-asm"),
    ("no-legacy-pass-manager", "legacy-pass-manager"),
    ("no-lto-legacy-pass-manager", "new-pass-manager"),
    ("--opt-remarks-with-hotness", "opt-remarks-with-hotness"),
    ("lto-pseudo-probe-for-profiling", "pseudo-probe-for-profiling"),
    ("save-temps", "save-temps"),
    ("thinlto-emit-imports-files", "thinlto-emit-imports-files"),
    ("thinlto-index-only", "thinlto-index-only"),
]
LTO_ARGS = [
    ("--lto-cs-profile-file", "cs-profile-path="),
    ("--lto-partitions", "lto-partitions="),
    ("--lto-obj-path", "obj-path="),
    ("--opt-remarks-filename", "opt-remarks-filename="),
    ("--opt-remarks-format", "opt-remarks-format="),
    ("--opt-remarks-hotness-threshold", "opt-remarks-hotness-threshold="),
    ("--opt-remarks-passes", "opt-remarks-passes="),
    ("--lto-sample-profile", "sample-profile="),
    ("thinlto-index-only", "thinlto-index-only="),
    ("thinlto-object-suffix-replace", "thinlto-object-suffix-replace="),
    ("thinlto-prefix-replace", "thinlto-prefix-replace="),
    ("thinlto-cache-dir", "cache-dir="),
    ("thinlto-cache-policy", "cache-policy="),
    ("thinlto-jobs", "jobs="),
]

def camel(s):
    s = {'(': 'open-paren', ')': 'close-paren'}.get(s, s)
    parts = re.split(r'[^A-Za-z0-9]+', s)
    return ''.join(p[:1].upper() + p[1:] for p in parts if p)

class Sp:
    """One spelling: an arm, a kind (flag/arg/eq), a name, maybe a literal."""
    def __init__(s, arm, kind, name, raw=False, switch=None, lto=None):
        s.arm, s.kind, s.raw, s.switch, s.lto = arm, kind, raw, switch, lto
        s.two_dashes = name.startswith('--')
        n = name[2:] if s.two_dashes else name
        if '=' in n and kind == 'flag':
            n, s.literal = n.split('=', 1)
        else:
            s.literal = None
        s.base = n
        s.short = len(n) == 1

class ZSp:
    def __init__(s, arm, kind, name, switch=None):
        s.arm, s.kind, s.name, s.switch = arm, kind, name, switch

spellings, zspellings = [], []
for n, a in enumerate(arms):
    for kind, g in a.matchers:
        if kind == 'flag': spellings.append(Sp(n, 'flag', g[0]))
        elif kind in ('arg', 'eq'): spellings.append(Sp(n, kind, g[0], raw=bool(g[1])))
        elif kind == 'switch':
            spellings.append(Sp(n, 'flag', g[0], switch=True))
            spellings.append(Sp(n, 'flag', g[1], switch=False))
        elif kind == 'zflag': zspellings.append(ZSp(n, 'flag', g[0]))
        elif kind == 'zarg': zspellings.append(ZSp(n, 'arg', g[0]))
        elif kind == 'zswitch':
            zspellings.append(ZSp(n, 'flag', g[0], switch=True))
            zspellings.append(ZSp(n, 'flag', g[1], switch=False))
        elif kind == 'lto':
            for name, value in LTO_FLAGS:
                spellings.append(Sp(n, 'flag', name, lto=('flag', value)))
            for name, prefix in LTO_ARGS:
                spellings.append(Sp(n, 'arg', name, raw=True, lto=('arg', prefix)))

# Groups by spelling base: one variant each.
groups = {}
for sp in spellings:
    groups.setdefault(sp.base, []).append(sp)

ignored = {n for n, a in enumerate(arms) if not any(l.strip() and not l.strip().startswith('//') for l in a.body)}

def first_long(arm):
    for sp in spellings:
        if sp.arm == arm and not sp.short:
            return sp.base
    return None

class Variant:
    pass

variants = {}   # base -> Variant
names_used = set()
for base, sps in groups.items():
    v = Variant(); v.base = base; v.sps = sps
    kinds = {sp.kind for sp in sps}
    v.has_plain_flag = any(sp.kind == 'flag' and sp.literal is None for sp in sps)
    v.literals = [sp.literal for sp in sps if sp.literal is not None]
    v.unit = kinds == {'flag'} and not v.literals
    v.require_equals = not v.unit and ('flag' in kinds or 'arg' not in kinds)
    v.default_missing = not v.unit and v.has_plain_flag
    v.two_dashes = all(sp.two_dashes for sp in sps)
    v.short = sps[0].short
    arm = sps[0].arm
    if arm in ignored:
        name = 'Ignored' + camel(base) if not v.short else 'Ignored' + (camel(base) if not base.isalpha() else ('Short' + base.upper() + ('' if base.isupper() else 'Lower')))
    elif v.short:
        long = first_long(arm)
        name = (camel(long) + 'Short') if long else ('Short' + camel(base).upper() + ('' if base.isupper() else 'Lower'))
    elif base.startswith(':'):
        name = 'Internal' + camel(base)
    else:
        name = camel(base)
    if v.short and base.isalpha():
        # mold's -O takes a value, -O0 is a flag spelled O0 etc.
        pass
    while name in names_used:
        name += 'X'
    names_used.add(name)
    v.name = name
    variants[base] = v

# z keywords: skip variants for arms that only have z spellings
zonly = {n for n in {z.arm for z in zspellings} if not any(sp.arm == n for sp in spellings)}
zvariants = {}   # keyword -> (variant name, takes value)
for z in zspellings:
    if z.arm in zonly:
        name = 'Z' + camel(z.name)
        while name in names_used: name += 'X'
        names_used.add(name)
        zvariants[z.name] = (name, z.kind == 'arg')

def rust_str(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'

# ---- the enum (written after the fold, which decides what is read) ----
def emit_enum(used_values):
  out = []
  out.append('#[derive(Occurrence)]\n#[arg(allow_hyphen_values, keep_equals)]\npub(crate) enum Item {')
  for base, v in variants.items():
      attrs = []
      if v.short:
          attrs.append(f"short = '{base}'" if base != "'" else "short = '\\''")
          if base == 'l':
              attrs.append('prefix')
      else:
          attrs.append(f'long = {rust_str(base)}')
          if v.two_dashes:
              attrs.append('two_dashes')
      if v.require_equals:
          attrs.append('require_equals')
      if v.default_missing:
          attrs.append(f'default_missing = "{NUL}"')
      spell = ', '.join(sorted({('--' if sp.two_dashes else '-') + sp.base + (('=' + sp.literal) if sp.literal else '') for sp in v.sps}))
      out.append(f'    /// `{spell}`')
      out.append(f'    #[arg({", ".join(attrs)})]')
      out.append(f'    {v.name}' + ('' if v.unit else '(OsString)') + ',')
  out.append("    /// `-z KEYWORD`, `-zKEYWORD`: mapped by [`z_opt`] as it is handled;\n    /// one that stays is unknown.\n    #[arg(short = 'z')]\n    Z(Spanned<OsString>),")
  for kw, (name, takes) in zvariants.items():
      out.append(f'    /// `-z {kw}{"=VALUE" if takes else ""}`.\n    #[arg(skip)]\n    {name}' + ('(OsString)' if takes else '') + ',')
  out.append('    /// A flag nothing above names, whole: `--lto-O3`, or an error.\n    #[arg(unknown)]\n    Unknown(OsString),')
  out.append('    /// Several short options in one word (`-sS`): accepted, with a warning,\n    /// as GNU ld does.\n    #[arg(bundle)]\n    Grouped(OsString),')
  out.append('    /// An input file.\n    #[arg(positional)]\n    Input(OsString),')
  out.append('    /// `--help`.\n    #[arg(long = "help")]\n    Help,')
  out.append('}')
  return '\n'.join(out)

# ---- z_opt ----
zl = []
zl.append('/// The option a `-z` keyword stands for, as mold reads `-z` (the exact\n/// keyword, or `name=value`); `None` for an unknown one.\npub(crate) fn z_opt(word: &OsStr) -> Option<Item> {')
zl.append('    let word = word.to_str()?;')
zl.append('    Some(match word {')
def representative(arm):
    for sp in spellings:
        if sp.arm == arm:
            v = variants[sp.base]
            if v.unit: return f'Item::{v.name}'
            if sp.literal is not None: return f'Item::{v.name}(OsString::from({rust_str(sp.literal)}))'
            if sp.kind == 'flag': return f'Item::{v.name}(OsString::from("{NUL}"))'
            return f'Item::{v.name}(OsString::new())'
    raise ValueError(arm)
zargs = []
for z in zspellings:
    if z.kind == 'arg':
        zargs.append(z); continue
    if z.arm in zonly:
        target = f'Item::{zvariants[z.name][0]}'
    else:
        target = representative(z.arm)
    zl.append(f'        {rust_str(z.name)} => {target},')
zl.append('        _ => {')
for z in zargs:
    if z.arm in zonly:
        target = f'Item::{zvariants[z.name][0]}(OsString::from(value))'
    else:
        target = representative(z.arm)
    zl.append(f'            if let Some(value) = word.strip_prefix({rust_str(z.name + "=")}) {{\n                let _ = value;\n                return Some({target});\n            }}' if z.arm not in zonly else
              f'            if let Some(value) = word.strip_prefix({rust_str(z.name + "=")}) {{\n                return Some({target});\n            }}')
zl.append('            return None;\n        }\n    })\n}')
z_code = '\n'.join(zl)

# ---- the fold ----
def body_of(arm):
    return arms[arm].body

def uses(body, ident):
    return re.search(r'\b' + ident + r'\b', '\n'.join(body)) is not None

def indent(lines, extra):
    return [(' ' * extra + l[8:] if l.startswith('        ') else (l if not l.strip() else ' ' * extra + l.lstrip())) for l in lines]

match_arms = []   # (variant name, guard, binding?, prelude lines, arm index)
for base, v in variants.items():
    guarded, plain = [], []
    for sp in v.sps:
        prelude = []
        if sp.switch is not None:
            prelude.append(f'let value = {"true" if sp.switch else "false"};')
        if sp.lto:
            kind, text = sp.lto
            if kind == 'flag':
                prelude.append(f'let option = b{rust_str(text)}.to_vec();')
            else:
                prelude.append(f'let option = [b{rust_str(text)}.as_slice(), raw_arg.as_encoded_bytes()].concat();')
        spelled = ('--' if not sp.short else '-') + sp.base
        if not v.unit:
            body = body_of(sp.arm)
            if sp.kind in ('arg', 'eq') or sp.lto:
                need_raw = uses(body, 'raw_arg') or (sp.lto and sp.lto[0] == 'arg')
                if not sp.raw and not sp.lto and uses(body, 'arg'):
                    prelude.insert(0, f'let arg = utf8_arg(value_os, {rust_str(spelled)});')
                if need_raw:
                    prelude.insert(0, 'let raw_arg: &OsStr = value_os;')
        guard = None
        if not v.unit:
            if sp.literal is not None:
                guard = f'value_os.as_encoded_bytes() == b{rust_str(sp.literal)}'
            elif sp.kind == 'flag':
                guard = f'value_os.as_encoded_bytes() == b"{NUL}"'
        entry = (guard, prelude, sp.arm)
        (guarded if guard else plain).append(entry)
    for guard, prelude, arm in guarded + plain:
        used = guard is not None or any('value_os' in p for p in prelude)
        pat = f'Item::{v.name}' + ('' if v.unit else ('(value_os)' if used else '(_)'))
        match_arms.append((pat + (f' if {guard}' if guard else ''), prelude, arm))
    if not plain and not v.unit:
        lit = '--' + base + '='
        match_arms.append((f'Item::{v.name}(value_os)', [f'fatal!("unknown command line option: {lit}{{}}", value_os.to_string_lossy());'], None))

for kw, (name, takes) in zvariants.items():
    z = next(z for z in zspellings if z.name == kw)
    prelude = []
    if z.switch is not None:
        prelude.append(f'let value = {"true" if z.switch else "false"};')
    if takes:
        prelude.append(f'let arg = utf8_arg(value_os, "-z {kw}");')
    match_arms.append((f'Item::{name}' + ('(value_os)' if takes else ''), prelude, z.arm))

# Spellings of one mold arm with the same handling share a match arm: `A | B`.
merged = []
for pat, prelude, arm in match_arms:
    if ' if ' not in pat and merged and merged[-1][2] == arm and arm is not None \
            and ' if ' not in merged[-1][0][-1] \
            and ('(value_os)' in pat) == ('(value_os)' in merged[-1][0][-1]) \
            and ('(_)' in pat) == ('(_)' in merged[-1][0][-1]) \
            and [l for l in prelude if 'utf8_arg' not in l] == [l for l in merged[-1][1] if 'utf8_arg' not in l]:
        merged[-1][0].append(pat)
    else:
        merged.append(([pat], prelude, arm))
match_arms = [(' | '.join(pats), prelude, arm) for pats, prelude, arm in merged]

fold = []
fold.append('let mut opts = crate::cmdline_winnow::parse(raw_cmdline);')
fold.append('for opt in &mut opts {')
fold.append('    if let Item::Z(z) = opt')
fold.append('        && let Some(known) = crate::cmdline_winnow::z_opt(&z.value)')
fold.append('    {')
fold.append('        *opt = known;')
fold.append('    }')
fold.append('    match opt {')
# input
inp = G.input_block
# input block: first `if !cursor...starts_with(b"-") {` ... `}` then help block
ib = '\n'.join(inp)
m = re.search(r'if !cursor\.current\(\)\.as_encoded_bytes\(\)\.starts_with\(b"-"\) \{\n(.*?)\n            cursor\.index \+= 1;\n            continue;\n        \}', ib, re.S)
input_body = m.group(1).replace('PathBuf::from(&cursor.current())', 'PathBuf::from(std::mem::take(value_os))')
fold.append('        Item::Input(value_os) => {')
fold.extend('    ' + l for l in input_body.split('\n'))
fold.append('        }')
m = re.search(r'if cursor\.read_flag\("help"\) \{\n(.*?)\n        \}', ib, re.S)
fold.append('        Item::Help => {')
fold.extend('    ' + l for l in m.group(1).split('\n'))
fold.append('        }')
for pat, prelude, arm in match_arms:
    fold.append(f'        {pat} => {{')
    for p in prelude:
        fold.append('            ' + p)
    if arm is not None:
        for l in body_of(arm):
            fold.append('    ' + l if l.strip() else '')
    fold.append('        }')
# unknown
dyn_body = body_of(217)
fold.append('        Item::Grouped(value_os) => {')
fold.append('            warn!(')
fold.append('                "grouped short command line options are deprecated: {}",')
fold.append('                value_os.to_string_lossy()')
fold.append('            );')
fold.append('        }')
fold.append('        Item::Unknown(value_os) => {')
fold.append('            if let Some(level) = value_os.as_encoded_bytes().strip_prefix(b"--lto-O") {')
fold.append('                a.plugin_opt.push([b"O", level].concat());')
fold.append('            } else if value_os.as_os_str() == "-dynamic" {')
for l in dyn_body:
    fold.append('        ' + l if l.strip() else '')
fold.append('            } else {')
fold.append('                fatal!("unknown command line option: {}", value_os.to_string_lossy());')
fold.append('            }')
fold.append('        }')
fold.append('        Item::Z(z) => {')
fold.append('            if z.attached {')
fold.append('                warn!("unknown command line option: -z{}", z.value.to_string_lossy());')
fold.append('            } else {')
fold.append('                warn!("unknown command line option: -z {}", z.value.to_string_lossy());')
fold.append('            }')
fold.append('        }')
fold.append('    }')
fold.append('}')
fold_code = '\n'.join(fold)
used_values = {name for p, _, _ in match_arms for name in re.findall(r'Item::(\w+)\(value_os\)', p)}
enum_code = emit_enum(used_values)

open(out_dir + '/enum.rs', 'w').write(enum_code)
open(out_dir + '/z_opt.rs', 'w').write(z_code)
open(out_dir + '/fold.rs', 'w').write(fold_code)
eq_only = sorted('--' + v.base for v in variants.values() if v.require_equals)
open(out_dir + '/eq_only.txt', 'w').write('\n'.join(eq_only))
print(len(variants), 'variants,', len(zvariants), 'z variants,', len(match_arms), 'match arms')
