export type ThemePref = "system" | "light" | "dark";

const KEY = "mcpm.theme";
const media = window.matchMedia("(prefers-color-scheme: dark)");

export function getThemePref(): ThemePref {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* storage unavailable */
  }
  return "system";
}

export function applyTheme(pref: ThemePref = getThemePref()) {
  const dark = pref === "dark" || (pref === "system" && media.matches);
  document.documentElement.classList.toggle("dark", dark);
}

export function setThemePref(pref: ThemePref) {
  try {
    localStorage.setItem(KEY, pref);
  } catch {
    /* storage unavailable */
  }
  applyTheme(pref);
}

media.addEventListener("change", () => applyTheme());
