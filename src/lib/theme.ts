/** 应用系统主题：切换 dark 类、写内联 color-scheme（覆盖 wry 建窗时按
 * "系统模式"写入的内联值，否则浅色系统下滚动条等仍是深色）。测试共用。 */
export function applySystemTheme(theme: "light" | "dark") {
  document.documentElement.classList.toggle("dark", theme === "dark");
  document.documentElement.style.colorScheme = theme;
}
