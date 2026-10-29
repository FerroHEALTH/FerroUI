// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The line chart's legend. With no script it is a static key, each
// series' swatch beside its name; here each entry becomes a button with
// aria-pressed, and a press sets the hidden attribute of that series' two
// groups in the plot alone, so the drawing is never parsed again. The last
// series shown cannot be hidden, so the chart never empties itself.
export function enhance(legend) {
  const plot = document.getElementById(legend.getAttribute("data-fui-plot") || "");
  if (!plot) return;
  const keys = [...legend.querySelectorAll(".fui-legend-key")];
  const shown = keys.map(() => true);
  const buttons = keys.map((key, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "fui-legend-chip";
    button.setAttribute("data-series", key.getAttribute("data-series") || "");
    button.append(...key.childNodes);
    key.replaceWith(button);
    button.addEventListener("click", () => {
      const count = shown.filter(Boolean).length;
      if (shown[index] && count === 1) return;
      shown[index] = !shown[index];
      draw();
    });
    return button;
  });
  function draw() {
    const count = shown.filter(Boolean).length;
    buttons.forEach((button, index) => {
      button.setAttribute("aria-pressed", String(shown[index]));
      button.disabled = shown[index] && count === 1;
      for (const part of ["lines", "marks"]) {
        const group = plot.querySelector("[data-series-" + part + "='" + index + "']");
        if (group) group.toggleAttribute("hidden", !shown[index]);
      }
    });
  }
  draw();
}
