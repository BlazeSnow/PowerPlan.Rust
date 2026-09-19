import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, listen } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

import { useSystemTheme } from "@/lib/use-system-theme";

let emitSystemTheme: ((e: { payload: "light" | "dark" }) => void) | null = null;

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
  listen.mockImplementation((_event, handler) => {
    emitSystemTheme = handler;
    return Promise.resolve(() => {});
  });
  document.documentElement.className = "";
});

describe("useSystemTheme", () => {
  it("applies backend theme on mount", async () => {
    invoke.mockResolvedValue("dark");
    const { result } = renderHook(() => useSystemTheme());

    await waitFor(() => expect(result.current).toBe("dark"));
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("follows system-theme events and toggles the dark class", async () => {
    invoke.mockResolvedValue("light");
    const { result } = renderHook(() => useSystemTheme());
    await waitFor(() => expect(result.current).toBe("light"));

    act(() => {
      emitSystemTheme?.({ payload: "dark" });
    });
    expect(result.current).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);

    act(() => {
      emitSystemTheme?.({ payload: "light" });
    });
    expect(result.current).toBe("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});
