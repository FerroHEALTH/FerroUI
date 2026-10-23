- The code viewer draws every finding: one on line 0 sits above the first
  line and one past the last line under the last, where before neither was
  drawn. Its findings are indexed by line once, so a file of 20000 lines
  with a finding on many of them draws in time linear in both.
