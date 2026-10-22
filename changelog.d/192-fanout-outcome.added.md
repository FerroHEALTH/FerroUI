- The fan-out outcome (F5), `view::fanout::fanout`: the per-node state of
  one request that went to many. When a node failed, "Incomplete answer" comes
  first in words with what is missing, then the verdict (Complete, Incomplete
  or Failed) as a pill with the console's summary line, then one row per node
  with its state as a shape, a colour and a word, its protocol term, what it
  gave and the reason it failed. A node the request left out is drawn dashed
  and stays listed. At a phone's width each node is one line with its reason
  under it.
