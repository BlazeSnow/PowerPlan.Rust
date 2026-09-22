import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { listen, toastError } = vi.hoisted(() => ({
  listen: vi.fn(),
  toastError: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("sonner", () => ({ toast: { error: toastError } }));

import i18n from "@/i18n";
import { useAutostartErrorToast } from "@/lib/use-autostart-error";

let emit: ((e: { payload: unknown }) => void) | null = null;

beforeEach(() => {
  listen.mockReset();
  toastError.mockClear();
  listen.mockImplementation((_event, handler) => {
    emit = handler;
    return Promise.resolve(() => {});
  });
});

describe("useAutostartErrorToast", () => {
  it("toasts the translated message for disabled-by-user", () => {
    renderHook(() => useAutostartErrorToast());

    act(() => {
      emit?.({
        payload: { key: "App.Status.StartupSettingDisabledByUser" },
      });
    });
    expect(toastError).toHaveBeenCalledTimes(1);
    expect(toastError).toHaveBeenCalledWith(
      i18n.t("App.Status.StartupSettingDisabledByUser"),
    );
  });

  it("interpolates args for generic failures", () => {
    renderHook(() => useAutostartErrorToast());

    act(() => {
      emit?.({
        payload: {
          key: "App.Status.StartupSettingFailed",
          args: { 0: "boom" },
        },
      });
    });
    expect(toastError).toHaveBeenCalledWith(
      i18n.t("App.Status.StartupSettingFailed", { 0: "boom" }),
    );
  });
});
