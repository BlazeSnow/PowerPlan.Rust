import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

// vi.mock 工厂被提升到文件顶部，invoke 须用 vi.hoisted 声明
const { invoke, listen } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

import i18n from "@/i18n";
import { HomePage } from "@/pages/home";
import { SidebarProvider } from "@/components/ui/sidebar";

const BALANCED = "381b4222-f694-41f0-9685-ff5bb260df2e";
const CUSTOM = "11111111-2222-3333-4444-555555555555";
const HIDDEN = "99999999-8888-7777-6666-555555555555";

const PLANS = [
  { guid: BALANCED, name: "平衡", isActive: true },
  { guid: CUSTOM, name: "游戏", isActive: false },
];

const baseSettings = {
  language: "auto",
  autoStartEnabled: false,
  trayEnabled: true,
  launchToTray: false,
  ultimatePerformancePlanGuid: null as string | null,
};

function mockBackend(settings = { ...baseSettings }) {
  invoke.mockImplementation((command: string) => {
    if (command === "power_list_plans") return Promise.resolve(PLANS);
    if (command === "settings_get") return Promise.resolve(settings);
    return Promise.resolve(null);
  });
}

// 断言文案经真实 i18n 资源解析，避免硬编码语言
function renderHome() {
  // PageHeader 的 SidebarTrigger 依赖 SidebarProvider 上下文
  return render(
    <SidebarProvider>
      <HomePage />
    </SidebarProvider>,
  );
}

const text = (key: string) => String(i18n.t(key));

// vitest 未开启 globals 时 testing-library 不会自动清理，手动卸载避免 DOM 残留
afterEach(() => cleanup());

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
  // 组件 effect 会 await listen(...) 并保存返回的取消订阅函数
  listen.mockResolvedValue(() => {});
});

describe("HomePage", () => {
  it("renders plan list from backend", async () => {
    mockBackend();
    renderHome();

    // 状态卡与列表均可能显示当前计划名，用 radio 角色精确定位列表项
    expect(
      await screen.findByRole("radio", { name: /^平衡/ }),
    ).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /^游戏/ })).toBeInTheDocument();
  });

  it("shows create entry when no ultimate performance plan", async () => {
    mockBackend();
    renderHome();

    expect(
      await screen.findByText(text("Main.UltimateMissingTitle")),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: text("Main.CreateUltimateButton") }),
    ).toBeInTheDocument();
  });

  it("shows activate entry when saved ultimate is hidden", async () => {
    mockBackend({ ...baseSettings, ultimatePerformancePlanGuid: HIDDEN });
    renderHome();

    expect(
      await screen.findByText(text("Main.UltimateHiddenTitle")),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: text("Main.ActivateUltimateButton") }),
    ).toBeInTheDocument();
  });

  it("hides ultimate card when ultimate exists in plans", async () => {
    mockBackend({ ...baseSettings, ultimatePerformancePlanGuid: CUSTOM });
    renderHome();

    await screen.findByRole("radio", { name: /^平衡/ });
    expect(
      screen.queryByText(text("Main.UltimateMissingTitle")),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByText(text("Main.UltimateHiddenTitle")),
    ).not.toBeInTheDocument();
  });

  it("switches plan through backend command", async () => {
    mockBackend();
    renderHome();

    fireEvent.click(await screen.findByRole("radio", { name: /游戏/ }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("power_set_active", { guid: CUSTOM });
    });
  });

  it("prefills copy dialog with plan name and suffix", async () => {
    mockBackend();
    renderHome();

    const copyButtons = await screen.findAllByRole("button", {
      name: text("Main.CopyPlanButton"),
    });
    fireEvent.click(copyButtons[0]);

    const input = await screen.findByRole("textbox");
    expect(input).toHaveValue(
      `平衡 - ${text("Main.CopySuffix")}`,
    );
  });
});
