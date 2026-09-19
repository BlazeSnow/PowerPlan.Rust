import { useTranslation } from "react-i18next";
import { Zap } from "lucide-react";
import type { UltimateState } from "@/lib/plan";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

/** 卓越性能卡片：hidden 提供激活、missing 提供创建；exists 时由页面隐藏 */
export function UltimateCard({
  state,
  onActivate,
  onCreate,
}: {
  state: UltimateState;
  onActivate: () => void;
  onCreate: () => void;
}) {
  const { t } = useTranslation();
  const hidden = state === "hidden";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Zap className="size-4 text-amber-500" />
          {t(hidden ? "Main.UltimateHiddenTitle" : "Main.UltimateMissingTitle")}
        </CardTitle>
        <CardDescription>
          {t(hidden ? "Main.UltimateHiddenMessage" : "Main.UltimateMissingMessage")}
        </CardDescription>
        <CardAction>
          {hidden ? (
            <Button onClick={() => void onActivate()}>
              {t("Main.ActivateUltimateButton")}
            </Button>
          ) : (
            <Button onClick={() => void onCreate()}>
              {t("Main.CreateUltimateButton")}
            </Button>
          )}
        </CardAction>
      </CardHeader>
    </Card>
  );
}
