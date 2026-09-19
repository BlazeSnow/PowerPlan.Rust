import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Zap } from "lucide-react";
import { toast } from "sonner";
import {
  duplicateUltimate,
  getSettings,
  listPlans,
  openPowerOptions,
  setActivePlan,
  type AppSettings,
  type PlanInfo,
} from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";

// 系统卓越性能模板 GUID，识别规则见 references/power-plans.md
const ULTIMATE_TEMPLATE_GUID = "e9a42b02-d5df-448d-aa00-03f14749eb61";

export function HomePage() {
  const { t } = useTranslation();
  const [plans, setPlans] = useState<PlanInfo[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [planList, appSettings] = await Promise.all([
        listPlans(),
        getSettings(),
      ]);
      setPlans(planList);
      setSettings(appSettings);
    } catch (error) {
      showCommandError(t, error, "Main.Status.RefreshFailed");
    }
  }, [t]);

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
            0:
              plans.find((p) => p.guid === guid)?.name ||
              t("Main.DefaultPlanName"),
          }),
        );
      } catch (error) {
        showCommandError(t, error, "Main.Status.SwitchFailed");
      }
      await refresh();
    },
    [t, plans, refresh],
  );

  const createUltimate = useCallback(async () => {
    try {
      await duplicateUltimate();
      toast.success(t("Main.Status.UltimateCreated"));
    } catch (error) {
      showCommandError(t, error, "Main.Status.UltimateCreateFailed");
    }
    await refresh();
  }, [t, refresh]);

  const activePlan = plans.find((p) => p.isActive) ?? null;
  const ultimate =
    plans.find(
      (p) =>
        p.guid === ULTIMATE_TEMPLATE_GUID ||
        p.guid === settings?.ultimatePerformancePlanGuid,
    ) ?? null;

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
      <UltimateCard ultimate={ultimate} onCreate={createUltimate} onRefresh={refresh} />
      <Card>
        <CardHeader>
          <CardTitle>{t("Main.PlanPickerTitle")}</CardTitle>
          <CardDescription>{t("Main.DeletePlanHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          <RadioGroup
            value={activePlan?.guid ?? ""}
            onValueChange={(value) => void switchPlan(value)}
          >
            {plans.map((plan) => (
              <Label
                key={plan.guid}
                className="flex cursor-pointer items-center gap-3 rounded-lg border p-3 font-normal"
              >
                <RadioGroupItem value={plan.guid} />
                {plan.name || t("Main.DefaultPlanName")}
              </Label>
            ))}
          </RadioGroup>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>{t("Main.PowerOptions")}</CardTitle>
          <CardDescription>{t("Main.PowerOptionsDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => void openPowerOptions()}>
            {t("Settings.Tools.OpenButton")}
          </Button>
        </CardContent>
      </Card>
      <StatusCard activePlanName={activePlan?.name ?? null} />
    </div>
  );
}

function UltimateCard({
  ultimate,
  onCreate,
  onRefresh,
}: {
  ultimate: PlanInfo | null;
  onCreate: () => Promise<void>;
  onRefresh: () => Promise<void>;
}) {
  const { t } = useTranslation();

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Zap className="size-4 text-amber-500" />
          {ultimate ? ultimate.name || t("Main.DefaultPlanName") : t("Main.UltimateMissingTitle")}
        </CardTitle>
        <CardDescription>
          {ultimate
            ? t("Main.Status.UltimateExists")
            : t("Main.UltimateMissingMessage")}
        </CardDescription>
      </CardHeader>
      {!ultimate && (
        <CardContent>
          <Button onClick={() => void onCreate()}>
            {t("Main.CreateUltimateButton")}
          </Button>
        </CardContent>
      )}
      {ultimate && !ultimate.isActive && (
        <CardContent>
          <Button
            onClick={() => void (async () => {
              try {
                await setActivePlan(ultimate.guid);
                toast.success(
                  t("Main.Status.SwitchSuccess", { 0: ultimate.name }),
                );
              } catch (error) {
                showCommandError(t, error, "Main.Status.SwitchFailed");
              }
              await onRefresh();
            })()}
          >
            {t("Main.ActivateUltimateButton")}
          </Button>
        </CardContent>
      )}
    </Card>
  );
}

/** 状态：当前计划名称与当前时间；定时器仅在页面可见时运行 */
function StatusCard({ activePlanName }: { activePlanName: string | null }) {
  const { t } = useTranslation();
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    let timer: number | undefined;
    const start = () => {
      timer = window.setInterval(() => setNow(new Date()), 1000);
    };
    const stop = () => {
      if (timer !== undefined) window.clearInterval(timer);
    };
    const onVisibility = () => {
      if (document.hidden) {
        stop();
      } else {
        setNow(new Date());
        start();
      }
    };
    if (!document.hidden) start();
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      stop();
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  return (
    <Card>
      <CardContent className="flex items-center justify-between text-sm">
        <span>{activePlanName ?? t("Main.StatusWaiting")}</span>
        <span className="text-muted-foreground tabular-nums">
          {now.toLocaleTimeString()}
        </span>
      </CardContent>
    </Card>
  );
}
