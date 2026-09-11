import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Input } from "@wealthfolio/ui/components/ui/input";
import { Label } from "@wealthfolio/ui/components/ui/label";

function bedrockRegionUrl(region: string): string | null {
  const value = region.trim();
  return /^[a-z]+-[a-z]+(?:-[a-z]+)?-\d+$/.test(value)
    ? `https://bedrock-mantle.${value}.api.aws/v1`
    : null;
}

function regionFromUrl(url?: string): string {
  return (
    url?.match(/^https:\/\/bedrock-mantle\.([a-z0-9-]+)\.api\.aws\/v1\/?$/)?.[1] ??
    url?.match(/^https:\/\/bedrock-runtime\.([a-z0-9-]+)\.amazonaws\.com\/openai\/v1\/?$/)?.[1] ??
    ""
  );
}

export function BedrockRegionField({
  url,
  onSave,
}: {
  url?: string | null;
  onSave?: (url: string) => void;
}) {
  const { t } = useTranslation();
  const [region, setRegion] = useState(() => regionFromUrl(url ?? undefined));
  useEffect(() => setRegion(regionFromUrl(url ?? undefined)), [url]);
  const endpoint = bedrockRegionUrl(region);
  return (
    <div className="bg-muted/40 space-y-3 rounded-lg p-4">
      <Label htmlFor="bedrock-region">{t("ai:providerSettings.bedrockRegion")}</Label>
      <p className="text-muted-foreground text-xs">{t("ai:providerSettings.bedrockDescription")}</p>
      <div className="flex items-center gap-2">
        <Input
          id="bedrock-region"
          value={region}
          placeholder="us-east-1"
          autoCapitalize="none"
          spellCheck={false}
          aria-invalid={region.length > 0 && !endpoint}
          onChange={(event) => setRegion(event.target.value)}
        />
        <Button
          type="button"
          variant="outline"
          disabled={!endpoint || endpoint === url || !onSave}
          onClick={() => endpoint && onSave?.(endpoint)}
        >
          {t("ai:providerSettings.save")}
        </Button>
      </div>
      <p className="text-muted-foreground text-xs">{t("ai:providerSettings.bedrockModels")}</p>
    </div>
  );
}
