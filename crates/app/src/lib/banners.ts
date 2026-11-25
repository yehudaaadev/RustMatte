import type { Translations } from "./i18n.ts";
import type { ItemWarning, RejectedItem } from "./queue.ts";

export const BANNER_CODES = [
  "job_done",
  "job_partial",
  "job_cancelled",
  "saved_single",
  "saved_all",
  "oom_warning",
  "damaged_warning",
  "rejected_files",
] as const;

export type BannerCode = (typeof BANNER_CODES)[number];

export type StatusBanner =
  | { code: "job_done"; processed: number; total: number; durationSec: number }
  | { code: "job_partial"; processed: number; total: number; durationSec: number }
  | { code: "job_cancelled" }
  | { code: "saved_single"; displayPath: string }
  | { code: "saved_all"; count: number; displayDir: string };

export const REJECTED_REASON_CODES = [
  "inaccessible",
  "read_dir_failed",
  "not_file_or_dir",
  "unsupported_format",
] as const;

export type RejectedReasonCode = (typeof REJECTED_REASON_CODES)[number];

/**
 * Strips Windows extended-length prefixes (`\\?\C:\...`, `\\?\UNC\...`) for UI display only.
 */
export function formatDisplayPath(raw: string): string {
  if (raw.length >= 8 && raw.startsWith("\\\\?\\") && raw.slice(4, 8).toLowerCase() === "unc\\") {
    return "\\\\" + raw.slice(8);
  }
  if (raw.startsWith("\\\\?\\")) {
    const rest = raw.slice(4);
    if (
      rest.length >= 2 &&
      /^[A-Za-z]$/.test(rest[0]) &&
      rest[1] === ":" &&
      (rest.length === 2 || rest[2] === "\\" || rest[2] === "/")
    ) {
      return rest;
    }
  }
  return raw;
}

function interpolate(template: string, vars: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_match, key: string) =>
    Object.prototype.hasOwnProperty.call(vars, key) ? String(vars[key]) : `{${key}}`
  );
}

export function renderStatusBanner(banner: StatusBanner, t: Translations): string {
  switch (banner.code) {
    case "job_done":
      return interpolate(t.bannerJobDone, {
        processed: banner.processed,
        total: banner.total,
        sec: banner.durationSec.toFixed(2),
      });
    case "job_partial":
      return interpolate(t.bannerJobPartial, {
        processed: banner.processed,
        total: banner.total,
        sec: banner.durationSec.toFixed(2),
      });
    case "job_cancelled":
      return t.stateCancelledSummary;
    case "saved_single":
      return interpolate(t.bannerSavedSingle, {
        path: formatDisplayPath(banner.displayPath),
      });
    case "saved_all":
      return interpolate(t.bannerSavedAll, {
        count: banner.count,
        dir: formatDisplayPath(banner.displayDir),
      });
  }
}

export function renderBannerByCode(
  code: BannerCode,
  t: Translations
): string {
  switch (code) {
    case "job_done":
      return renderStatusBanner({ code: "job_done", processed: 1, total: 1, durationSec: 1.0 }, t);
    case "job_partial":
      return renderStatusBanner({ code: "job_partial", processed: 1, total: 2, durationSec: 1.5 }, t);
    case "job_cancelled":
      return renderStatusBanner({ code: "job_cancelled" }, t);
    case "saved_single":
      return renderStatusBanner({ code: "saved_single", displayPath: "C:\\out\\cutout.png" }, t);
    case "saved_all":
      return renderStatusBanner({ code: "saved_all", count: 1, displayDir: "C:\\out" }, t);
    case "oom_warning":
      return t.oomWarningBanner;
    case "damaged_warning":
      return t.damagedWarningPrefix;
    case "rejected_files":
      return t.rejectedFilesBanner;
    default: {
      const exhaustive: never = code;
      throw new Error(`Missing banner translation for code: ${String(exhaustive)}`);
    }
  }
}

export function renderRejectedReason(item: RejectedItem, t: Translations): string {
  switch (item.reason_code) {
    case "inaccessible":
      return t.rejectedReasonInaccessible;
    case "read_dir_failed":
      return t.rejectedReasonReadDir;
    case "not_file_or_dir":
      return t.rejectedReasonNotFileOrDir;
    case "unsupported_format":
      return t.rejectedReasonUnsupported;
    default:
      return item.reason || t.rejectedReasonInaccessible;
  }
}

export function renderWarningItem(warning: ItemWarning | string, t: Translations): string {
  if (typeof warning === "string") {
    return warning;
  }
  switch (warning.code) {
    case "damaged_input":
      return warning.detail
        ? `${t.warningDamagedInput} (${warning.detail})`
        : t.warningDamagedInput;
    default:
      return warning.detail || t.warningDamagedInput;
  }
}
