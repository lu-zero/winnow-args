#!/bin/bash
# Regenerate mold's winnow-args parser from a pristine src/cmdline.rs.
set -e
S=$(cd "$(dirname "$0")" && pwd)
OUT=$(mktemp -d)
cd "${1:?usage: apply.sh MOLD_DIR}"
git show origin/main:src/cmdline.rs > src/cmdline.rs
python3 $S/moldgen2.py src/cmdline.rs $OUT $S
export S OUT
python3 - <<'EOF'
import os
S=os.environ['S']; OUT=os.environ['OUT']
enum=open(OUT+'/enum.rs').read()
zopt=open(OUT+'/z_opt.rs').read()
eq=[l for l in open(OUT+'/eq_only.txt').read().split('\n') if l]
eq_list=',\n'.join('    "%s"' % e for e in eq)
mod=open(S+'/mold_template.rs').read().replace('@ENUM@', enum).replace('@ZOPT@', zopt).replace('@EQ@', eq_list)
open('src/cmdline_winnow.rs','w').write(mod)
p='src/cmdline.rs'; L=open(p).read().split('\n')
start=next(i for i,l in enumerate(L) if l.strip()=='let mut cursor = ArgCursor { args: raw_cmdline, index: 1 };')
w=next(i for i,l in enumerate(L) if i>start and l.strip()=='while cursor.index < raw_cmdline.len() {')
end=next(i for i,l in enumerate(L) if i>w and l=='    }')
fold=open(OUT+'/fold.rs').read().split('\n')
legacy = []
depth = 0
for l in L[start:end+1]:
    # Each top-level statement of the built-in parser, compiled out by the feature.
    if depth == 0 and l.strip() and not l.strip().startswith('//'):
        legacy.append('    #[cfg(not(feature = "winnow-args"))]')
    legacy.append(l)
    depth += l.count('{') - l.count('}') + l.count('(') - l.count(')') + l.count('[') - l.count(']')
new = (L[:start]
  + ['    // The built-in parser: each option tried in turn against the current word.']
  + legacy
  + ['',
     '    // winnow-args lexes the whole command line into options, in order;',
     '    // each is handled as the built-in parser handles it.',
     '    #[cfg(feature = "winnow-args")]', '    {',
     '        use crate::cmdline_winnow::{Item, utf8_arg};', '']
  + ['        '+l if l else l for l in fold]
  + ['    }']
  + L[end+1:])
text='\n'.join(new)
for item in ['fn match_option<', 'struct ArgCursor<', "impl<'a> ArgCursor<'a> {"]:
    assert text.count('\n'+item)==1, item
    text=text.replace('\n'+item, '\n#[cfg_attr(feature = "winnow-args", allow(dead_code))]\n'+item)
open(p,'w').write(text)
EOF
cargo fmt -- src/cmdline.rs src/cmdline_winnow.rs 2>/dev/null || true
