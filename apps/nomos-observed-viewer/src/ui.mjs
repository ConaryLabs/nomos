import { CATALOG } from "./catalog.mjs";

const px = (value) => `${value}px`;

export const mountControls = (document, count, selected, choose) => {
  const panel = document.createElement("nav");
  Object.assign(panel.style, {
    display: "flex",
    gap: px(CATALOG.ui.gap_px),
    left: px(CATALOG.ui.inset_px),
    position: "fixed",
    top: px(CATALOG.ui.inset_px),
    zIndex: "1",
  });
  for (let index = 0; index < count; index += 1) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = CATALOG.ui.labels[index];
    button.setAttribute("aria-label", `Observation ${CATALOG.ui.labels[index]}`);
    Object.assign(button.style, {
      background: index === selected ? CATALOG.ui.active_background : CATALOG.ui.background,
      border: `${px(CATALOG.ui.border_width_px)} solid ${CATALOG.ui.border_color}`,
      borderRadius: px(CATALOG.ui.radius_px),
      color: CATALOG.ui.text_color,
      font: CATALOG.ui.font,
      height: px(CATALOG.ui.control_height_px),
      minWidth: px(CATALOG.ui.control_min_width_px),
    });
    button.addEventListener("click", () => choose(index));
    panel.append(button);
  }
  document.body.append(panel);
  return panel;
};
