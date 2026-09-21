import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

/** 设置行：左侧标题与描述，右侧开关 */
export function SwitchRow({
  title,
  description,
  checked,
  disabled,
  onCheckedChange,
}: {
  title: string;
  description: string;
  checked: boolean;
  disabled?: boolean;
  onCheckedChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex flex-col gap-1">
        <Label className="text-sm font-medium">{title}</Label>
        <span className="text-sm text-muted-foreground">{description}</span>
      </div>
      <Switch
        checked={checked}
        disabled={disabled}
        onCheckedChange={onCheckedChange}
      />
    </div>
  );
}

/** 外链卡片：官网、代码仓库等"前往"入口 */
export function LinkCard({
  title,
  description,
  url,
  openLabel,
}: {
  title: string;
  description: string;
  url: string;
  openLabel: string;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <ExternalLink className="size-4" />
          {title}
        </CardTitle>
        <CardDescription>{description}</CardDescription>
        <CardAction>
          <Button variant="outline" onClick={() => void openUrl(url)}>
            {openLabel}
          </Button>
        </CardAction>
      </CardHeader>
    </Card>
  );
}
