import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Power } from "lucide-react";
import { toast } from "sonner";
import {
  clearSavedUltimate,
  duplicateUltimate,
  getSettings,
  listPlans,
  openPowerOptions,
  setActivePlan,
  type AppSettings,
  type PlanInfo,
} from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import { PageHeader } from "@/components/page-header";
import { resolveUltimateState } from "@/lib/plan";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { PlanList } from "./plan-list";
import { UltimateCard } from "./ultimate-card";

export function HomePage() {
  const { t } = useTranslation();
  const [plans, setPlans] = useState<PlanInfo[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  // 首次加载完成前渲染骨架屏：避免切页挂载瞬间以"无计划"状态渲染
  // （卓越性能卡片闪现"未发现"再消失的竞态）
  const [loaded, setLoaded] = useState(false);

  const refresh = useCallback(
    async (force = false) => {
      try {
        const [planList, appSettings] = await Promise.all([
          listPlans(force),
          getSettings(),
        ]);
        setPlans(planList);
        setSettings(appSettings);
      } catch (error) {
        showCommandError(t, error, "Main.Status.RefreshFailed");
      } finally {
        setLoaded(true);
      }
    },
    [t],
  );

  useEffect(() => {
    void refresh();
    // 托盘切换计划后通知前端刷新
    const unlistenPromise = listen("plans-changed", () => void refresh());
    return () => {
      void unlistenPromise.then((fn) => fn());
    };
  }, [refresh]);

  const switchPlan = useCallback(
    async (guid: string) => {
      try {
        await setActivePlan(guid);
        toast.success(
          t("Main.Status.SwitchSuccess", {
            0: plans.find((p) => p.guid === guid)?.name ?? guid,
          }),
        );
      } catch (error) {
        showCommandError(t, error, "Main.Status.SwitchFailed");
      }
      await refresh();
    },
    [t, plans, refresh],
  );

  const savedGuid = settings?.ultimatePerformancePlanGuid ?? null;
  // 卓越性能卡片三态（exists 隐藏卡片 / hidden 提供激活 / missing 提供创建）
  const ultimateState = resolveUltimateState(plans, savedGuid);

  const activateSavedUltimate = useCallback(async () => {
    if (!savedGuid) return;
    try {
      await setActivePlan(savedGuid);
      toast.success(t("Main.Status.UltimateActivated"));
    } catch (error) {
      // 激活失败说明计划已被删除，清空储存的 UUID（旧版 MainPage 行为）
      await clearSavedUltimate().catch(() => {});
      showCommandError(t, error, "Main.Status.UltimateActivateFailed");
    }
    await refresh(true);
  }, [t, savedGuid, refresh]);

  const createUltimate = useCallback(async () => {
    try {
      await duplicateUltimate();
      toast.success(t("Main.Status.UltimateCreated"));
    } catch (error) {
      showCommandError(t, error, "Main.Status.UltimateCreateFailed");
    }
    await refresh(true);
  }, [t, refresh]);

  const activePlan = plans.find((p) => p.isActive) ?? null;

  // 首次加载前以骨架占位，布局稳定且无卡片闪现
  if (!loaded) {
    return (
      <>
        <PageHeader titleKey="Shell.Home" />
        <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
          <Skeleton className="h-28 rounded-xl" />
          <Skeleton className="h-48 rounded-xl" />
          <Skeleton className="h-24 rounded-xl" />
        </div>
      </>
    );
  }

  return (
    <>
      <PageHeader titleKey="Shell.Home" />
      <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
        {ultimateState !== "exists" && (
          <UltimateCard
            state={ultimateState}
            onActivate={() => void activateSavedUltimate()}
            onCreate={() => void createUltimate()}
          />
        )}
        <PlanList
          plans={plans}
          activeGuid={activePlan?.guid ?? ""}
          onSwitch={switchPlan}
          onRefresh={refresh}
        />
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Power className="size-4" />
              {t("Main.PowerOptions")}
            </CardTitle>
            <CardDescription>{t("Main.PowerOptionsDesc")}</CardDescription>
            <CardAction>
              <Button variant="outline" onClick={() => void openPowerOptions()}>
                {t("Settings.Tools.OpenButton")}
              </Button>
            </CardAction>
          </CardHeader>
        </Card>
      </div>
    </>
  );
}
