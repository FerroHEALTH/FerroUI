#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""selects.py <src dir>: every <select> whose chosen text is wider than its box, at 1440 and 390; the pages come from the manifest beside this file."""
import os, sys, importlib.util
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
spec = importlib.util.spec_from_file_location("render", os.path.join(HERE, "render.py"))
render = importlib.util.module_from_spec(spec); spec.loader.exec_module(render)
import pages
JS = r"""(() => { const c=document.createElement('canvas').getContext('2d'); const out=[];
for (const s of document.querySelectorAll('select')) { const r=s.getBoundingClientRect(); if (r.width<1) continue;
 const cs=getComputedStyle(s); c.font=cs.fontWeight+' '+cs.fontSize+' '+cs.fontFamily;
 const t=s.options[s.selectedIndex]?s.options[s.selectedIndex].text:''; const w=c.measureText(t).width;
 const room=r.width-parseFloat(cs.paddingLeft)-parseFloat(cs.paddingRight);
 if (w>room+1) out.push([t, Math.round(w), Math.round(room)]); } return out; })()"""
ch = render.Chrome(os.path.join(render.STATE_DIR, "chrome-profile-sel"))
try:
    t = ch.send("Target.createTarget", {"url": "about:blank"})["targetId"]
    s = ch.send("Target.attachToTarget", {"targetId": t, "flatten": True})["sessionId"]
    ch.send("Page.enable", {}, s); ch.send("Runtime.enable", {}, s)
    n = 0
    for e in pages.flat():
        for w in (1440, 390):
            ch.send("Emulation.setDeviceMetricsOverride", {"width": w, "height": 900, "deviceScaleFactor": 1, "mobile": w < 600}, s)
            ch.send("Page.navigate", {"url": "file://" + os.path.abspath(os.path.join(sys.argv[1], e[3] + ".html"))}, s); ch.wait_event("Page.loadEventFired", s)
            for x in ch.send("Runtime.evaluate", {"expression": JS, "returnByValue": True}, s)["result"].get("value") or []:
                n += 1; print(e[3], w, x)
    print(n, "truncated selects")
finally:
    ch.close()
