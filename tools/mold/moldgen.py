import re, sys
from collections import OrderedDict

SRC = open(sys.argv[1]).read()
lines = SRC.split('\n')

def find(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]): return i
    raise ValueError

w = find(lambda l: l.strip() == 'while cursor.index < raw_cmdline.len() {')
chain = find(lambda l: l.startswith('        if read_arg!("o", true)'), w)
# end of while: first line '    }' after chain
end = find(lambda l: l == '    }', chain)
input_block = lines[w+1:chain]

class Arm:
    def __init__(s, header, body): s.header, s.body = header, body
arms = []
i = chain
while i < end:
    l = lines[i]
    assert l.startswith('        if ') or l.startswith('        } else if ') or l == '        } else {', (i, l)
    if l == '        } else {':
        hdr = ['else']; j = i
    else:
        hdr = [l]; j = i
        while not lines[j].rstrip().endswith('{'):
            j += 1; hdr.append(lines[j])
    k = j + 1
    body = []
    while k < end and not (lines[k].startswith('        }') ):
        body.append(lines[k]); k += 1
    arms.append(Arm(' '.join(x.strip() for x in hdr), body))
    i = k
    if lines[i] == '        }': break

PAT = [
    ('flag',    re.compile(r'cursor\.read_flag\("([^"]+)"\)')),
    ('arg',     re.compile(r'read_arg!\("([^"]+)"(, true)?\)')),
    ('eq',      re.compile(r'read_eq!\("([^"]+)"(, true)?\)')),
    ('switch',  re.compile(r'cursor\.read_switch\(\s*"([^"]+)",\s*"([^"]+)"\s*\)')),
    ('zflag',   re.compile(r'cursor\.read_z_flag\("([^"]+)"\)')),
    ('zarg',    re.compile(r'read_z_arg!\("([^"]+)"\)')),
    ('zswitch', re.compile(r'cursor\.read_z_switch\(\s*"([^"]+)",\s*"([^"]+)"\s*\)')),
    ('lto',     re.compile(r'cursor\.read_lto_option\(\)')),
]
for a in arms:
    a.matchers = []
    for kind, rx in PAT:
        for m in rx.finditer(a.header):
            a.matchers.append((m.start(), kind, m.groups()))
    a.matchers.sort()
    a.matchers = [(k, g) for _, k, g in a.matchers]

if __name__ == '__main__':
    print(len(arms), 'arms; input block', len(input_block), 'lines')
    for n, a in enumerate(arms):
        if not a.matchers: print(n, 'NO MATCHERS:', a.header[:100])
    from collections import Counter
    print(Counter(k for a in arms for k, _ in a.matchers))
