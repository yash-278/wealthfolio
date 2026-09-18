import { useEffect, useState } from "react";
import { listCategorizationRules, deleteCategorizationRule } from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
export default function LearnedRules() {
  const [rules, setRules] = useState<{ id: string; name: string; pattern: string }[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let active = true;
    listCategorizationRules()
      .then((rows) => {
        if (active) setRules(rows.filter((r) => r.id.startsWith("quick-add-learned:")));
      })
      .catch(() => {
        if (active) setError("Could not load learned rules.");
      });
    return () => {
      active = false;
    };
  }, []);
  async function reset() {
    setBusy(true);
    setError("");
    try {
      for (const rule of rules) await deleteCategorizationRule(rule.id);
      setRules([]);
    } catch {
      setError("Could not reset all rules. Reload to see what remains.");
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="space-y-3">
      <h2 className="font-medium">Learned merchant categories</h2>
      <p className="text-muted-foreground text-sm">
        Category corrections apply to future captures in the same account and direction. Explicit
        rules take priority. Editing or resetting learning leaves saved categories unchanged.
      </p>
      {error && (
        <p role="alert" className="text-destructive">
          {error}
        </p>
      )}
      <ul>
        {rules.map((rule) => (
          <li key={rule.id}>{rule.name}</li>
        ))}
      </ul>
      <div className="flex gap-3">
        <a className="text-primary underline" href="/settings/spending/rules">
          Inspect and edit category rules
        </a>
        <Button variant="outline" disabled={busy || !rules.length} onClick={reset}>
          Reset learned rules
        </Button>
      </div>
    </section>
  );
}
