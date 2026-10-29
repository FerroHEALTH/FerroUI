// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The upload field. A file dropped on the zone or chosen is checked
// against the field's kinds and limit, as the crate's admit checks it, and
// read with Blob.arrayBuffer() (W3C File API) into the editor's text area,
// which then redraws as typed text does. The field names the file read,
// says when it was the first of several dropped, and when bytes were no
// UTF-8 text; a refused or unreadable file is said in the refusal line.
const OPEN = "";
const SHUT = "";
const text = (word) => (word && typeof word === "object" ? word.en : word || "");

function fill(phrase, values) {
  let out = text(phrase);
  for (const [name, value] of Object.entries(values)) out = out.split(OPEN + name + SHUT).join(String(value));
  return out;
}

function grouped(number, group) {
  return String(number).replace(/\B(?=(\d{3})+(?!\d))/g, group);
}

// A size in its largest unit above one, as the crate's SizeWords::size.
export function size(sizes, bytes) {
  const KIB = 1024;
  const MIB = KIB * KIB;
  const count = (unit) => grouped(Math.ceil(bytes / unit), sizes.grouping.group);
  if (bytes >= MIB) return fill(sizes.mib, { count: count(MIB) });
  if (bytes >= KIB) return fill(sizes.kib, { count: count(KIB) });
  if (bytes === 1) return text(sizes.one_byte);
  return fill(sizes.bytes, { count: grouped(bytes, sizes.grouping.group) });
}

// Why a file may not be read, or null when it may.
export function refusal(words, accept, max, name, bytes) {
  const lower = name.toLowerCase();
  if (accept.length && !accept.some((kind) => lower.endsWith(kind.toLowerCase()))) return fill(words.refused_kind, { file: name });
  if (bytes > max) return fill(words.refused_size, { file: name, size: size(words.sizes, bytes) });
  return null;
}

// The text of a file's bytes and whether any were no UTF-8, as the
// crate's decode reads them (the Encoding Standard, "UTF-8 decode").
export function decode(bytes) {
  try {
    return { text: new TextDecoder("utf-8", { fatal: true }).decode(bytes), lossy: false };
  } catch {
    return { text: new TextDecoder("utf-8").decode(bytes), lossy: true };
  }
}

export function enhance(root) {
  const words = JSON.parse(root.getAttribute("data-fui-words") || "{}");
  const accept = (root.getAttribute("data-fui-accept") || "").split(",").filter(Boolean);
  const max = Number(root.getAttribute("data-fui-max") || "0");
  const zone = root.querySelector(".fui-drop");
  const input = zone && zone.querySelector("input[type='file']");
  const area = root.querySelector("textarea");
  const said = zone && zone.querySelector("[data-upload='words']");
  const line = (name) => root.querySelector("[data-upload='" + name + "']");
  if (!zone || !input || !area) return;

  function show(name, words) {
    const at = line(name);
    if (!at) return;
    at.hidden = words === null;
    at.querySelector("span").textContent = words || "";
  }
  function over(on) {
    zone.classList.toggle("fui-drop-over", on);
    if (said) said.textContent = text(on ? words.drop_over : words.drop_invite);
  }
  function refuse(sentence) {
    show("read", null);
    show("lossy", null);
    show("refused", sentence);
    zone.classList.add("fui-drop-refused");
  }
  async function read(file, several) {
    const name = file.name;
    const refused = refusal(words, accept, max, name, file.size);
    if (refused) return refuse(refused);
    let bytes;
    try {
      bytes = new Uint8Array(await file.arrayBuffer());
    } catch {
      return refuse(fill(words.unreadable, { file: name }));
    }
    const got = decode(bytes);
    area.value = got.text;
    area.dispatchEvent(new Event("input", { bubbles: true }));
    zone.classList.remove("fui-drop-refused");
    show("refused", null);
    show("read", fill(several ? words.read_one : words.read, { file: name }));
    show("lossy", got.lossy ? fill(words.read_lossy, { file: name }) : null);
  }
  zone.addEventListener("dragover", (event) => {
    event.preventDefault();
    over(true);
  });
  zone.addEventListener("dragleave", () => over(false));
  zone.addEventListener("drop", (event) => {
    event.preventDefault();
    over(false);
    const files = event.dataTransfer ? event.dataTransfer.files : null;
    if (files && files.length) read(files[0], files.length > 1);
  });
  input.addEventListener("change", () => {
    if (input.files && input.files.length) read(input.files[0], false);
  });
}
