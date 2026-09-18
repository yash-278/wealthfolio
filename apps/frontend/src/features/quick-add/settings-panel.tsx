import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useEffect, useState } from "react";
import {
  getAccounts,
  getCaptureUsage,
  getCaptureSettings,
  updateCaptureSettings,
  isWeb,
} from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Input } from "@wealthfolio/ui/components/ui/input";
import LearnedRules from "./learned-rules";
import ShortcutTokens from "./shortcut-tokens";
import type { CaptureSettings, CaptureUsage } from "./types";
import type { CaptureAccount } from "./correction-form";
export default function SettingsPanel() {
  const [usage, setUsage] = useState<CaptureUsage | null>(null);
  const [loaded, setLoaded] = useState(false);
  const form = useForm<CaptureSettings>({
    resolver: zodResolver(
      z
        .object({
          extractionVersion: z.string().optional(),
          sourceRetentionDays: z.number().int().min(1).max(3650).nullable().optional(),
          provider: z.string().refine((v): boolean => v === "bedrock"),
          model: z.string().refine((v): boolean => v === "openai.gpt-5.6-luna"),
          monthlyBudgetMicros: z
            .number()
            .positive("Set a positive AI budget")
            .nullable()
            .refine((v) => v !== null, "Set an AI budget"),
          timezone: z.string().refine((v) => {
            try {
              new Intl.DateTimeFormat("en", { timeZone: v });
              return true;
            } catch {
              return false;
            }
          }, "Choose a valid timezone"),
          mappings: z.array(
            z.object({ alias: z.string().trim().min(1), accountId: z.string().min(1) }),
          ),
          typedNoteAccountId: z.string().nullable(),
          typedNoteToday: z.boolean(),
          automaticPosting: z.boolean(),
          evaluationPassed: z.boolean(),
          supervisedTrialCompleted: z.boolean(),
        })
        .refine((s) => !s.automaticPosting || (s.evaluationPassed && s.supervisedTrialCompleted), {
          message: "Complete evaluation and supervised trial first",
          path: ["automaticPosting"],
        }),
    ),
  });
  const settings = form.watch();
  const setSettings = form.reset;
  const [accounts, setAccounts] = useState<CaptureAccount[]>([]);
  const [budget, setBudget] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let active = true;
    Promise.all([getCaptureSettings(), getAccounts(), getCaptureUsage()])
      .then(([s, a, usage]) => {
        if (active) setUsage(usage);
        if (!active) return;
        setSettings({
          ...s,
          evaluationPassed: Boolean(s.evaluationPassed),
          supervisedTrialCompleted: Boolean(s.supervisedTrialCompleted),
          provider: s.provider || "bedrock",
          model: s.model || "openai.gpt-5.6-luna",
          timezone: s.timezone || Intl.DateTimeFormat().resolvedOptions().timeZone,
        });
        setBudget(s.monthlyBudgetMicros ? String(s.monthlyBudgetMicros / 1_000_000) : "");
        setAccounts(a);
        setLoaded(true);
      })
      .catch(() => {
        if (active) setError("Could not load Quick Add settings.");
      });
    return () => {
      active = false;
    };
  }, [setSettings]);
  async function save(values: CaptureSettings) {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await updateCaptureSettings(values);
      setMessage("Settings saved");
    } catch (error) {
      setError(error instanceof Error ? error.message : "Could not save settings");
    } finally {
      setBusy(false);
    }
  }
  const selectStyle =
    "border-input bg-background block min-h-11 w-full min-w-0 rounded-md border p-2 text-base";
  return (
    <section className="max-w-3xl space-y-6 [&_button]:min-h-11 [&_button]:whitespace-normal">
      <header>
        <h1 className="text-2xl font-semibold">Quick Add settings</h1>
        <p className="text-muted-foreground">
          Capture uses its own model selection and your existing AI Providers credentials.
        </p>
      </header>
      {error && (
        <p role="alert" className="text-destructive">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {loaded && (
        <form onSubmit={form.handleSubmit(save)}>
          <fieldset disabled={busy} className="space-y-5">
            <label className="block space-y-2">
              Extraction model
              <select
                className={selectStyle}
                value={settings.model}
                onChange={(e) => setSettings({ ...settings, model: e.target.value })}
              >
                <option value="openai.gpt-5.6-luna">AWS Bedrock GPT-5.6 Luna</option>
              </select>
            </label>
            <a className="text-primary underline" href="/settings/ai-providers">
              Configure AI Providers credentials
            </a>
            <label className="block space-y-2">
              Monthly AI budget in USD
              <Input
                type="number"
                min="0.01"
                step="0.01"
                required
                value={budget}
                onChange={(e) => {
                  setBudget(e.target.value);
                  form.setValue(
                    "monthlyBudgetMicros",
                    Math.round(Number(e.target.value) * 1_000_000),
                  );
                }}
              />
            </label>
            {usage && (
              <p className="text-muted-foreground text-sm">
                {usage.month}: ${(usage.reservedOrUsedMicros / 1_000_000).toFixed(6)} estimated
                usage and pending allowances
              </p>
            )}
            <p className="text-muted-foreground text-sm">
              At the limit, captures remain in review while AI processing pauses. Usage is an
              estimate; provider billing may differ.
            </p>
            <label className="block space-y-2">
              Timezone
              <Input
                required
                value={settings.timezone}
                onChange={(e) => setSettings({ ...settings, timezone: e.target.value })}
              />
            </label>
            <div className="space-y-3">
              <h2 className="font-medium">Account aliases and suffixes</h2>
              <p className="text-muted-foreground text-sm">
                Ambiguous matches stay in review. These mappings never create accounts.
              </p>
              {settings.mappings.map((mapping, index) => (
                <div key={index} className="grid gap-2 sm:grid-cols-3">
                  <Input
                    aria-label={`Account alias ${index + 1}`}
                    required
                    value={mapping.alias}
                    onChange={(e) =>
                      setSettings({
                        ...settings,
                        mappings: settings.mappings.map((m, i) =>
                          i === index ? { ...m, alias: e.target.value } : m,
                        ),
                      })
                    }
                  />
                  <select
                    aria-label={`Mapped account ${index + 1}`}
                    required
                    className={selectStyle}
                    value={mapping.accountId}
                    onChange={(e) =>
                      setSettings({
                        ...settings,
                        mappings: settings.mappings.map((m, i) =>
                          i === index ? { ...m, accountId: e.target.value } : m,
                        ),
                      })
                    }
                  >
                    <option value="">Choose account</option>
                    {accounts
                      .filter((a) => a.isActive && !a.isArchived)
                      .map((a) => (
                        <option key={a.id} value={a.id}>
                          {a.name}
                        </option>
                      ))}
                  </select>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() =>
                      setSettings({
                        ...settings,
                        mappings: settings.mappings.filter((_, i) => i !== index),
                      })
                    }
                  >
                    Remove alias {index + 1}
                  </Button>
                </div>
              ))}
              <Button
                type="button"
                variant="outline"
                onClick={() =>
                  setSettings({
                    ...settings,
                    mappings: [...settings.mappings, { alias: "", accountId: "" }],
                  })
                }
              >
                Add account mapping
              </Button>
            </div>
            <label className="block space-y-2">
              Default account for typed notes
              <select
                className={selectStyle}
                value={settings.typedNoteAccountId ?? ""}
                onChange={(e) =>
                  setSettings({ ...settings, typedNoteAccountId: e.target.value || null })
                }
              >
                <option value="">Ask in review</option>
                {accounts
                  .filter((a) => a.isActive && !a.isArchived)
                  .map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.name}
                    </option>
                  ))}
              </select>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={settings.typedNoteToday}
                onChange={(e) => setSettings({ ...settings, typedNoteToday: e.target.checked })}
              />
              Use today&apos;s local date for typed notes without a date
            </label>
            <p className="text-muted-foreground text-sm">
              Bank and card alerts do not inherit typed-note defaults.
            </p>
            <div className="space-y-3">
              <h2 className="font-medium">Automatic posting</h2>
              <p className="text-muted-foreground text-sm">
                Enable only after verifying a representative evaluation set and a small supervised
                trial. Critical errors include wrong accounts, amounts, directions, dates, or
                duplicate payments.
              </p>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.evaluationPassed}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      evaluationPassed: e.target.checked,
                      automaticPosting: e.target.checked && settings.automaticPosting,
                    })
                  }
                />
                Evaluation completed with zero critical errors
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.supervisedTrialCompleted}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      supervisedTrialCompleted: e.target.checked,
                      automaticPosting: e.target.checked && settings.automaticPosting,
                    })
                  }
                />
                Supervised trial completed
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.automaticPosting}
                  disabled={!settings.evaluationPassed || !settings.supervisedTrialCompleted}
                  onChange={(e) => setSettings({ ...settings, automaticPosting: e.target.checked })}
                />
                Automatically post eligible transactions
              </label>
              <p className="text-muted-foreground text-sm">
                Turning this off keeps accepted captures and review work intact.
              </p>
            </div>
            <label className="block space-y-2">
              Source retention in days
              <Input
                type="number"
                min="1"
                max="3650"
                placeholder="Keep until deleted"
                value={settings.sourceRetentionDays ?? ""}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    sourceRetentionDays: e.target.value ? Number(e.target.value) : null,
                  })
                }
              />
            </label>
            <p className="text-muted-foreground text-sm">
              Leave blank to keep text until deletion. Automatic cleanup removes text only from
              completed captures and preserves transactions, references and unresolved evidence.
            </p>
            {Object.entries(form.formState.errors).map(([key, error]) => (
              <p key={key} role="alert" className="text-destructive">
                {error.message || "Check account mappings"}
              </p>
            ))}
            <Button type="submit">Save settings</Button>
          </fieldset>
        </form>
      )}
      <LearnedRules />
      {isWeb ? (
        <ShortcutTokens />
      ) : (
        <p>Manage iPhone Shortcut tokens in the web application that will receive your captures.</p>
      )}
    </section>
  );
}
