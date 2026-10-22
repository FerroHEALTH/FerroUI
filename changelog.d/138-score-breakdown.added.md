- `view::score::{score_breakdown, score_bar}`, the score breakdown (R16):
  one signed bar per compared field (`Contribution`), its plain words first
  and the comparator as a quiet hint, the total weight, the score with its
  grade, and the thresholds (`Threshold`) on one scale, each label centred
  on its mark and the start kept on a phone. The scale is an image named by
  the score, the grade, the thresholds and every contribution;
  `Score::tone` gives the grade's tone, and `score_bar` is the compact form
  for a table cell.
