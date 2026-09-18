export type CaptureInputKind = "bank_alert" | "card_alert" | "typed_note" | "unknown";
export interface CaptureInput {
  clientRequestId: string;
  text: string;
  inputKind: CaptureInputKind;
  sender?: string;
  sourceTimestamp?: string;
}
export interface CaptureFields {
  accountId: string;
  amount: string;
  currency: string;
  date: string;
  direction: string;
  merchant: string | null;
  reference: string | null;
  kind: string;
}
export interface CaptureReview {
  id: string;
  reason: string;
  status: string;
  activityId: string | null;
}
export interface CaptureReceipt {
  id: string;
  input: CaptureInput;
  submittedAt: string;
  status: string;
  version: number;
  reviews: CaptureReview[];
  candidates: {
    id: string;
    fields: CaptureFields;
    status: string;
    activityId: string;
    sourceText: string;
    sourceState?: string;
    sourceDates?: Record<string, string>;
    fieldOrigins?: Record<string, string>;
    extractionVersion?: string;
  }[];
}
export interface CaptureResolution {
  version: number;
  fields?: CaptureFields;
  action?: "complete" | "categorize" | "link_refund";
  relatedActivityId?: string;
  taxonomyId?: string;
  categoryId?: string;
}
export interface CaptureSettings {
  extractionVersion?: string;
  sourceRetentionDays?: number | null;
  provider: string;
  model: string;
  monthlyBudgetMicros: number | null;
  timezone: string;
  mappings: { alias: string; accountId: string }[];
  typedNoteAccountId: string | null;
  typedNoteToday: boolean;
  automaticPosting: boolean;
  evaluationPassed: boolean;
  supervisedTrialCompleted: boolean;
}
export interface CaptureToken {
  id: string;
  name: string;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
}

export interface CaptureUsage {
  month: string;
  reservedOrUsedMicros: number;
  monthlyBudgetMicros: number | null;
}
