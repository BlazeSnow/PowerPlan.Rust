import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, listen, getCurrentWindowListen } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  getCurrentWindowListen: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ listen: getCurrentWindowListen }),
}));
vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn(), warning: vi.fn(), info: vi.fn() },
  Toaster: () => null,
}));

import i18n from "@/i18n";
import App from "@/App";

const SETTINGS = {
  language: "auto",
  autoStartEnabled: false,
  trayEnabled: true,
  launchToTray: false,
  ultimatePerformancePlanGuid: null,
  autoStartState: "disabled" as const,
};

const text = (key: string) => String(i18n.t(key));

let pendingRequest = false;
let openSettingsHandler: (() => void) | null = null;

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
  getCurrentWindowListen.mockReset().mockResolvedValue(() => {});
  pendingRequest = false;
  openSettingsHandler = null;
  listen.mockImplementation((event: string, handler: (e?: unknown) => void) => {
    if (event === "open-settings") openSettingsHandler = handler;
    return Promise.resolve(() => {});
  });
  invoke.mockImplementation((command: string) => {
    if (command === "power_list_plans") return Promise.resolve([]);
    if (command === "settings_get") return Promise.resolve({ ...SETTINGS });
    if (command === "take_open_settings_request") {
      // 模拟后端 swap 语义：取走后即清零（StrictMode 下 effect 会跑两遍）
      const value = pendingRequest;
      pendingRequest = false;
      return Promise.resolve(value);
    }
    if (command === "system_theme") return Promise.resolve("light");
    return Promise.resolve(null);
  });
});

afterEach(() => {
  cleanup();
});

const takeCalls = () =>
  invoke.mock.calls.filter(([command]) => command === "take_open_settings_request")
    .length;

describe("托盘「打开软件设置」导航", () => {
  it("stays on home when no request is pending", async () => {
    render(<App />);
    await waitFor(() =>
      expect(screen.getByText(text("Main.PowerOptions"))).toBeInTheDocument(),
    );
    expect(screen.queryByText(text("Settings.Language.Title"))).toBeNull();
  });

  it("navigates to settings at mount when a request is pending", async () => {
    // webview 销毁重建场景：事件早于监听注册，挂载时消费挂起标记兜底
    pendingRequest = true;
    render(<App />);
    expect(await screen.findByText(text("Settings.Language.Title"))).toBeVisible();
    expect(takeCalls()).toBeGreaterThanOrEqual(1);
  });

  it("navigates on the open-settings event and consumes the pending flag", async () => {
    render(<App />);
    await waitFor(() => expect(takeCalls()).toBeGreaterThanOrEqual(1));
    const before = takeCalls();

    act(() => {
      openSettingsHandler?.({});
    });
    expect(await screen.findByText(text("Settings.Language.Title"))).toBeVisible();
    // 事件路径同样消费标记，防止陈旧标记在无关重开窗口时误导航
    // （StrictMode 下挂载 effect 双跑，计数用相对断言）
    expect(takeCalls()).toBeGreaterThanOrEqual(before + 1);
  });
});
