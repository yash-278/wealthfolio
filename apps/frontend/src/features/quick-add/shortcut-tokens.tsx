import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect, useState } from "react";
import { createCaptureToken, listCaptureTokens, revokeCaptureToken } from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Input } from "@wealthfolio/ui/components/ui/input";
import type { CaptureToken } from "./types";
export default function ShortcutTokens() {
  const [tokens, setTokens] = useState<CaptureToken[]>([]);
  const [created, setCreated] = useState<{ id: string; token: string } | null>(null);
  const schema = z.object({
    name: z.string().trim().min(1).max(128),
    expires: z
      .string()
      .refine((v) => !v || Number.isFinite(new Date(v).getTime()), "Enter a valid expiration"),
  });
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { name: "", expires: "" },
  });
  const name = form.watch("name");
  const expires = form.watch("expires");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(true);
  useEffect(() => {
    let active = true;
    listCaptureTokens()
      .then((t) => {
        if (active) setTokens(t);
      })
      .catch(() => {
        if (active) setError("Could not load Shortcut tokens");
      })
      .finally(() => {
        if (active) setBusy(false);
      });
    return () => {
      active = false;
    };
  }, []);
  async function create() {
    setBusy(true);
    setError("");
    try {
      const token = await createCaptureToken(
        name,
        expires ? new Date(expires).toISOString() : null,
      );
      setCreated({ id: token.id, token: token.token });
      setTokens((t) => [token, ...t]);
      form.setValue("name", "");
    } catch {
      setError("Could not create token. Check the name and expiration.");
    } finally {
      setBusy(false);
    }
  }
  async function revoke(id: string) {
    setBusy(true);
    try {
      await revokeCaptureToken(id);
      setTokens((t) => t.filter((token) => token.id !== id));
      setCreated((c) => (c?.id === id ? null : c));
    } catch {
      setError("Could not revoke token. Try again.");
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">iPhone Shortcut access</h2>
      <p className="text-muted-foreground">
        Tokens can submit captures and read their own receipts. They cannot read your account
        history or change settings.
      </p>
      {error && (
        <p role="alert" className="text-destructive">
          {error}
        </p>
      )}
      <form
        onSubmit={form.handleSubmit(create, () => setError("Check the token name and expiration."))}
        className="space-y-3"
      >
        <label className="block">
          Shortcut token name
          <Input
            required
            maxLength={128}
            value={name}
            onChange={(e) => form.setValue("name", e.target.value)}
          />
        </label>
        <label className="block">
          Expiration, optional
          <Input
            type="datetime-local"
            value={expires}
            onChange={(e) => form.setValue("expires", e.target.value)}
          />
        </label>
        <Button disabled={busy || !name.trim()} type="submit">
          Create token
        </Button>
      </form>
      {created && (
        <div className="bg-muted space-y-2 rounded-md p-4">
          <p>Copy this token now. It will not be shown again after leaving this page.</p>
          <label className="block">
            New Shortcut token
            <Input readOnly value={created.token} onFocus={(e) => e.target.select()} />
          </label>
          <Button variant="outline" onClick={() => setCreated(null)}>
            Hide token
          </Button>
        </div>
      )}
      {!busy && !tokens.length && <p>No Shortcut tokens</p>}
      {tokens.map((token) => (
        <div
          key={token.id}
          className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3"
        >
          <div>
            <p>{token.name}</p>
            <p className="text-muted-foreground text-sm">
              Last used: {token.lastUsedAt ? new Date(token.lastUsedAt).toLocaleString() : "Never"}
            </p>
          </div>
          <Button disabled={busy} variant="outline" onClick={() => revoke(token.id)}>
            Revoke {token.name}
          </Button>
        </div>
      ))}
      <a className="text-primary underline" href="/quick-add/request-template.json" download>
        Download request template
      </a>
      <details>
        <summary className="cursor-pointer font-medium">Set up capture on iPhone</summary>
        <ol className="list-decimal space-y-2 py-3 pl-6">
          <li>Create a Shortcut that accepts text, or asks you to paste or type it.</li>
          <li>
            Generate a UUID for clientRequestId. Keep the same UUID and text if you retry an
            unconfirmed request.
          </li>
          <li>
            Use Get Contents of URL to POST JSON to your HTTPS Wealthfolio address followed by
            /api/v1/captures.
          </li>
          <li>
            Add Authorization: Bearer followed by your token, and Content-Type: application/json.
          </li>
          <li>
            Send clientRequestId, text, and inputKind. Use bank_alert, card_alert, typed_note, or
            unknown.
          </li>
          <li>
            Read the returned id and status. Check GET /api/v1/captures/ID with the same token up to
            three times, then open Quick Add review in the application.
          </li>
        </ol>
        <p className="text-muted-foreground">
          A network error means the receipt is unconfirmed. An accepted capture continues after the
          Shortcut closes. Messages share-sheet support depends on iOS and the source app; copied or
          typed text is the supported starting point.
        </p>
      </details>
    </section>
  );
}
