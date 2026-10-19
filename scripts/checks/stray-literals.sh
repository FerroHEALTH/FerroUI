#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Stray-literal guard: a string a reader sees belongs in a catalog under
# locales/, never in the code (.claude/rules/leptos-ui.md §13; no
# specification governs this: our own design).
#
# It reads the hand-written .rs files under each ROOT, up to the file's
# `#[cfg(test)]` module, and reports every string literal that reads as
# words a reader sees:
#
#   1. the value of an attribute a reader sees or hears (`aria-label`,
#      `title`, `placeholder`, `alt`, `data-label`, `aria-description`,
#      `aria-roledescription`, `aria-valuetext`, `label`) holding a letter;
#   2. a text node of a `view!`: a literal between two tags (`>"Close"<`)
#      or alone on its line, holding a letter;
#   3. any other literal holding words: an upper-case letter, or a space
#      next to a letter, outside its `{}` placeables, and not a class list.
#
# A single lower-case word in code (`=> "fatal"`) reads as a key and is
# passed over, so a word drawn that way is review's to catch. Lines that
# speak to a developer are passed over: `expect(`, log macros,
# `write!(f`, `f.write_str(`, `debug_struct(`, `.field(`, assertions and
# attributes. A literal the check reads wrong is allowed by a line of the
# allow file, and a file the check does not cover is skipped there:
#
#   allow <perl regex matched against the literal's text>   # the reason
#   skip <path or directory prefix, from the repository root> # the reason
#
# The baseline file holds `<count> <path>` lines for a tree whose strings
# are still moving to the catalogs: a file may hold up to its count and no
# more, and a file not named may hold none. With --update the script
# rewrites the baseline to the counts it finds, which only a change that
# moves strings to the catalogs should need.
#
# Usage:
#   scripts/checks/stray-literals.sh [--update] ROOT...
#
# The allow file is scripts/checks/stray-literals.allow and the baseline
# scripts/checks/stray-literals.baseline, both optional.
#
# Exit 0 = clean, 1 = a stray literal (listed as file:line: literal), 2 =
# a usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

update=0
if [ "${1:-}" = "--update" ]; then
  update=1
  shift
fi
if [ "$#" -eq 0 ]; then
  echo "usage: scripts/checks/stray-literals.sh [--update] ROOT..." >&2
  exit 2
fi
for dir in "$@"; do
  if [ ! -d "$dir" ]; then
    echo "stray-literals: $dir is no directory" >&2
    exit 2
  fi
done

allow="scripts/checks/stray-literals.allow"
baseline="scripts/checks/stray-literals.baseline"

files=()
while IFS= read -r file; do
  files+=("$file")
done < <(find "$@" -type f -name '*.rs' | LC_ALL=C sort)
if [ "${#files[@]}" -eq 0 ]; then
  echo "stray-literals: no .rs file under $*"
  exit 0
fi

found="$(
  ALLOW_FILE="$allow" perl -e '
    use strict;
    use warnings;

    my (@allow, @skip);
    if (open(my $fh, "<", $ENV{ALLOW_FILE})) {
      while (my $line = <$fh>) {
        chomp $line;
        $line =~ s/\s+#.*$//;
        next if $line =~ /^\s*(#|$)/;
        if ($line =~ /^allow\s+(.+?)\s*$/) { push @allow, qr/$1/; }
        elsif ($line =~ /^skip\s+(\S+)\s*$/) { push @skip, $1; }
        else { die "stray-literals: $ENV{ALLOW_FILE}: unreadable line: $line\n"; }
      }
    }

    my $visible = qr/(?:aria-label|title|placeholder|alt|data-label|aria-description|aria-roledescription|aria-valuetext|label)/;

    # Whether the literal text $t reads as words outside its placeables.
    sub wordy {
      my ($t) = @_;
      (my $bare = $t) =~ s/\{[^{}]*\}//g;
      return 0 unless $bare =~ /[A-Za-z]/;
      my @tokens = grep { length } split /\s+/, $bare;
      return 0 if @tokens && !grep { $_ !~ /^fui-[a-z0-9-]+$/ } @tokens;
      return 1 if $bare =~ /[A-Z]/;
      return 1 if $bare =~ /[A-Za-z]\s|\s[A-Za-z]/;
      return 0;
    }

    FILE: for my $file (@ARGV) {
      next FILE if grep { index($file, $_) == 0 } @skip;
      open(my $fh, "<", $file) or die "stray-literals: cannot read $file\n";
      my $n = 0;
      while (my $line = <$fh>) {
        $n++;
        next FILE if $n == 1 && $line =~ /\@generated/;
        last if $line =~ /^#\[cfg\(test\)\]/;
        next if $line =~ m{^\s*//};
        next if $line =~ /^\s*#!?\[/;
        next if $line =~ /expect\(|logging::|warn!\(|error!\(|debug!\(|log!\(|write!\(f|write_str\(|debug_struct\(|\.field\(|assert|panic!\(|unreachable!\(|include_str!\(/;
        my $code = $line;
        $code =~ s/r#+".*?"#+//g;
        while ($code =~ /(?<pre>(?:[A-Za-z_:-]+\s*=\s*)?)"(?<lit>(?:[^"\\]|\\.)*)"/g) {
          my ($pre, $lit) = ($+{pre}, $+{lit});
          my $before = substr($code, 0, $-[0]);
          my $after = substr($code, $+[0]);
          my $hit = 0;
          if ($pre =~ /^($visible)\s*=/) {
            $hit = $lit =~ /[A-Za-z]/;
          } elsif ($pre =~ /=\s*$/ && $pre =~ /^[a-z-:]+/) {
            $hit = 0;
          } elsif ($before =~ />\s*$/ && $after =~ /^\s*</) {
            $hit = $lit =~ /[A-Za-z]/;
          } elsif ($before =~ /^\s*$/ && $after =~ /^\s*$/) {
            my @tokens = grep { length } split /\s+/, $lit;
            $hit = $lit =~ /[A-Za-z]/ && grep { $_ !~ /^fui-[a-z0-9-]+$/ } @tokens;
          } else {
            $hit = wordy($lit);
          }
          next unless $hit;
          next if grep { $lit =~ $_ } @allow;
          print "$file:$n: \"$lit\"\n";
        }
      }
    }
  ' "${files[@]}"
)"

counts="$(printf '%s\n' "$found" | sed -n 's/^\([^:]*\):[0-9]*:.*/\1/p' | LC_ALL=C sort | uniq -c | awk '{print $1, $2}')"

if [ "$update" -eq 1 ]; then
  {
    echo "# The stray literals each file may still hold while its strings move to"
    echo "# the catalogs; rewritten by scripts/checks/stray-literals.sh --update."
    if [ -n "$counts" ]; then printf '%s\n' "$counts"; fi
  } > "$baseline"
  echo "stray-literals: wrote $baseline"
  exit 0
fi

status=0
while IFS= read -r entry; do
  [ -n "$entry" ] || continue
  count="${entry%% *}"
  file="${entry#* }"
  allowed=0
  if [ -f "$baseline" ]; then
    allowed="$(awk -v f="$file" '$2 == f { print $1 }' "$baseline")"
    allowed="${allowed:-0}"
  fi
  if [ "$count" -gt "$allowed" ]; then
    status=1
    echo "stray-literals: $file holds $count stray literal(s), $allowed allowed:" >&2
    printf '%s\n' "$found" | awk -v f="$file:" 'index($0, f) == 1' >&2
  fi
done <<< "$counts"

if [ "$status" -ne 0 ]; then
  echo "stray-literals: move each string into the catalog under locales/ and look it up, or allow a literal that is no reader's text in $allow with the reason." >&2
  exit 1
fi
echo "stray-literals: no stray literal under $*"
