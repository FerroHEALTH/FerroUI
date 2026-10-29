// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The side panel's opener. With no script the opener is a link to the
// panel's content as a page; here the link leaves and the button that
// opens the panel as a modal dialog (the show-modal command) takes its
// place.
export function enhance(button) {
  const link = button.previousElementSibling;
  if (link && link.tagName === "A") link.hidden = true;
  button.hidden = false;
}
