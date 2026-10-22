- `view::hl7v2`, the HL7 v2 message viewer (B8): the message raw
  (`hl7v2_raw`), never wrapped and scrolling under the scroll cue, and
  parsed (`hl7v2_segments`), one block per segment with its fields by
  position, name and value, the names dropped on a phone.
- `highlight::Language::Hl7v2`, the HL7 v2 highlighter: each segment id and
  the five separators the message's header declares, total over any input.
