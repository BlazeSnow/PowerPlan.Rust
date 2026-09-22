import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

const { invoke, listen, getVersion, openUrl, toastWarning, toastInfo, toastError } =
  vi.hoisted(() => ({
    invoke: vi.fn(),
    listen: vi.fn(),
    getVersion: vi.fn(),
    openUrl: vi.fn(),
    toastWarning: vi.fn(),
    toastInfo: vi.fn(),
    toastError: vi.fn(),
  }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ listen: vi.fn().mockResolvedValue(() => {}) }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl }));
vi.mock("sonner", () => ({
  toast: {
    error: toastError,
    success: vi.fn(),
    warning: toastWarning,
    info: toastInfo,
  },
  Toaster: () => null,
}));

import i18n from "@/i18n";
import { SettingsPage } from "@/pages/settings";
import { SidebarProvider } from "@/components/ui/sidebar";

const SETTINGS = {
  language: "auto",
  autoStartEnabled: false,
  trayEnabled: true,
  launchToTray: false,
  ultimatePerformancePlanGuid: null,
  autoStartState: "disabled" as const,
};

const text = (key: string) => String(i18n.t(key));

beforeEach(() => {
  [invoke, getVersion, openUrl, toastWarning, toastInfo, toastError].forEach(
    (mock) => mock.mockReset(),
  );
  listen.mockReset();
  listen.mockResolvedValue(() => {});
  getVersion.mockResolvedValue("2026.9.19");
  invoke.mockImplementation((command: string) => {
    if (command === "settings_get") return Promise.resolve({ ...SETTINGS });
    return Promise.resolve(null);
  });
});

afterEach(() => {
  cleanup();
  // 语言切换测试会改全局 i18n 语言，恢复到 jsdom 默认
  void i18n.changeLanguage("en");
});

function renderSettings() {
  return render(
    <SidebarProvider>
      <SettingsPage />
    </SidebarProvider>,
  );
}

describe("SettingsPage", () => {
  it("renders version from app metadata", async () => {
    renderSettings();
    expect(await screen.findByText("2026.9.19")).toBeInTheDocument();
  });

  it("disables launch-to-tray switch when tray is disabled", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get")
        return Promise.resolve({ ...SETTINGS, trayEnabled: false });
      return Promise.resolve(null);
    });
    renderSettings();

    const switches = await screen.findAllByRole("switch");
    expect(switches[2]).toBeDisabled();
    expect(switches[1]).not.toBeDisabled();
  });

  it("warns via toast when enabled autostart is disabled by user", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get")
        return Promise.resolve({
          ...SETTINGS,
          autoStartEnabled: true,
          autoStartState: "disabled_by_user",
        });
      return Promise.resolve(null);
    });
    renderSettings();

    await screen.findAllByRole("switch");
    expect(toastWarning).toHaveBeenCalledWith(
      text("Settings.AutoStart.Title"),
      { description: text("Settings.AutoStart.StateDisabledByUser") },
    );
  });

  it("stays silent when autostart state matches the switch", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get")
        return Promise.resolve({
          ...SETTINGS,
          autoStartEnabled: false,
          autoStartState: "disabled_by_user",
        });
      return Promise.resolve(null);
    });
    renderSettings();

    await screen.findAllByRole("switch");
    expect(toastWarning).not.toHaveBeenCalled();
    expect(toastInfo).not.toHaveBeenCalled();
  });

  it("disables autostart switch and informs when unsupported", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get")
        return Promise.resolve({
          ...SETTINGS,
          autoStartState: "unsupported",
        });
      return Promise.resolve(null);
    });
    renderSettings();

    const switches = await screen.findAllByRole("switch");
    expect(switches[0]).toBeDisabled();
    expect(toastInfo).toHaveBeenCalledWith(text("Settings.AutoStart.Title"), {
      description: text("Settings.AutoStart.StateUnsupported"),
    });
  });

  it("renders three switches reflecting backend state", async () => {
    renderSettings();
    const switches = await screen.findAllByRole("switch");
    expect(switches).toHaveLength(3);
    // 渲染顺序：开机自启动（关）、启用托盘（开）、启动到托盘（关）
    expect(switches[0]).not.toBeChecked();
    expect(switches[1]).toBeChecked();
    expect(switches[2]).not.toBeChecked();
  });

  it("persists language change through the backend", async () => {
    const user = userEvent.setup();
    renderSettings();
    const trigger = await screen.findByRole("combobox");
    trigger.focus();
    await user.keyboard("{Enter}");
    await user.click(await screen.findByRole("option", { name: "English" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("settings_set_language", {
        value: "en",
      });
    });
  });

  it("rolls back autostart switch when the backend rejects", async () => {
    const user = userEvent.setup();
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get") return Promise.resolve({ ...SETTINGS });
      if (command === "settings_set_auto_start")
        return Promise.reject({
          key: "App.Status.StartupSettingFailed",
          args: { 0: "boom" },
        });
      return Promise.resolve(null);
    });
    renderSettings();
    const switches = await screen.findAllByRole("switch");
    await user.click(switches[0]);

    // 乐观更新失败后回滚为关闭
    await waitFor(() => expect(switches[0]).not.toBeChecked());
  });

  it("rolls back tray switch when the backend rejects", async () => {
    const user = userEvent.setup();
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get") return Promise.resolve({ ...SETTINGS });
      if (command === "settings_set_tray")
        return Promise.reject({
          key: "Settings.SaveFailed",
          args: { 0: "boom" },
        });
      return Promise.resolve(null);
    });
    renderSettings();
    const switches = await screen.findAllByRole("switch");
    await user.click(switches[1]);

    // 托盘开关初始为开启，乐观关闭失败后回滚为开启
    await waitFor(() => expect(switches[1]).toBeChecked());
  });

  it("restores defaults after confirmation", async () => {
    const user = userEvent.setup();
    renderSettings();
    await user.click(
      await screen.findByRole("button", {
        name: text("Settings.Tools.RestoreButton"),
      }),
    );
    await user.click(
      await screen.findByRole("button", {
        name: text("Settings.RestoreConfirmDialog.Confirm"),
      }),
    );

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("power_restore_defaults");
    });
  });

it("surfaces restore failure through error toast", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "settings_get") return Promise.resolve({ ...SETTINGS });
      if (command === "power_restore_defaults")
        return Promise.reject({
          key: "PowerPlan.Error.Win32",
          args: { label: "PowerPlan.Error.RestoreDefaultsFailed", code: "5" },
        });
      return Promise.resolve(null);
    });
    const user = userEvent.setup();
    renderSettings();

    await user.click(
      await screen.findByRole("button", {
        name: text("Settings.Tools.RestoreButton"),
      }),
    );
    await user.click(
      await screen.findByRole("button", {
        name: text("Settings.RestoreConfirmDialog.Confirm"),
      }),
    );

    await waitFor(() => expect(toastError).toHaveBeenCalledTimes(1));
    const message = String(toastError.mock.calls[0][0]);
    expect(message).toContain(
      text("PowerPlan.Error.RestoreDefaultsFailed"),
    );
    expect(message).toContain("5");
  });


    it("opens website and repository through the opener plugin", async () => {
    const user = userEvent.setup();
    renderSettings();

    // 官网与仓库的按钮同名"打开"，官网卡先渲染，取第一个
    const openButtons = await screen.findAllByRole("button", {
      name: text("Settings.Tools.OpenButton"),
    });
    await user.click(openButtons[0]);
    expect(openUrl).toHaveBeenCalledWith("https://powerplan.blazesnow.com/");
  });
});
