import "@testing-library/jest-dom/vitest";

// jsdom 未实现 scrollIntoView：radix Select 等挂载滚动时调用
Element.prototype.scrollIntoView = () => {};

// jsdom 未实现 matchMedia：shadcn sidebar 的 use-mobile hook 依赖它
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});
