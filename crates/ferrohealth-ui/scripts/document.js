// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The document pane's view switch (B1). With no script each segment is a
// link to its view and the stylesheet shows the view targeted (:target);
// here the views go live and a press shows its view in place, with no jump
// and no history entry, and marks its segment current.
export function enhance(views) {
  const group = views.previousElementSibling;
  if (!group) return;
  views.classList.add("is-live");
  group.addEventListener("click", (event) => {
    const segment = event.target.closest("a[data-view]");
    if (!segment) return;
    event.preventDefault();
    views.setAttribute("data-shown", segment.getAttribute("data-view"));
    for (const each of group.querySelectorAll("a[data-view]")) {
      if (each === segment) each.setAttribute("aria-current", "true");
      else each.removeAttribute("aria-current");
    }
  });
}
