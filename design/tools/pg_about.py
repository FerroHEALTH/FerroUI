# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The JSON highlighter the About page's self-description card uses."""
from common import e


def json_html(text):
    """JSON with the editor's token colours: keys, strings, numbers, literals."""
    import re
    out, pos = [], 0
    for m in re.finditer(r'"(?:[^"\\]|\\.)*"(\s*:)?|-?\d+(?:\.\d+)?|\b(?:true|false|null)\b', text):
        out.append(e(text[pos:m.start()]))
        tok = m.group(0)
        if tok.startswith('"'):
            if m.group(1):
                key = tok[: len(tok) - len(m.group(1))]
                out.append(f'<span class="fui-tok-key">{e(key)}</span>{e(m.group(1))}')
            else:
                out.append(f'<span class="fui-tok-string">{e(tok)}</span>')
        elif tok[0].isdigit() or tok[0] == "-":
            out.append(f'<span class="fui-tok-number">{tok}</span>')
        else:
            out.append(f'<span class="fui-tok-keyword">{tok}</span>')
        pos = m.end()
    out.append(e(text[pos:]))
    return "".join(out)
