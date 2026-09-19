import { invoke } from "@tauri-apps/api/core";

/** 后端命令错误：本地化键名 + 命名参数（i18next 插值渲染） */
export type CommandError = { key: string; args?: Record<string, string> };

export type PlanInfo = {
  guid: string;
  name: string;
  isActive: boolean;
};

export type AppSettings = {
  language: string;
  autoStartEnabled: boolean;
  trayEnabled: boolean;
  launchToTray: boolean;
  ultimatePerformancePlanGuid: string | null;
};

export const listPlans = (force = false) =>
  invoke<PlanInfo[]>("power_list_plans", { force });

export const setActivePlan = (guid: string) =>
  invoke<void>("power_set_active", { guid });

export const copyPlan = (sourceGuid: string, newName: string) =>
  invoke<string>("power_copy_plan", { sourceGuid, newName });

export const duplicateUltimate = () =>
  invoke<string>("power_duplicate_ultimate");

export const clearSavedUltimate = () =>
  invoke<void>("power_clear_saved_ultimate");

export const restoreDefaults = () => invoke<void>("power_restore_defaults");

export const openPowerOptions = () => invoke<void>("power_open_power_options");

export const getSettings = () => invoke<AppSettings>("settings_get");

export const setLanguage = (value: string) =>
  invoke<AppSettings>("settings_set_language", { value });

export const setAutoStart = (value: boolean) =>
  invoke<AppSettings>("settings_set_auto_start", { value });

export const setTray = (value: boolean) =>
  invoke<AppSettings>("settings_set_tray", { value });

export const setLaunchToTray = (value: boolean) =>
  invoke<AppSettings>("settings_set_launch_to_tray", { value });
