import type { Translations } from "./i18n.ts";

export interface FriendlyError {
  title: string;
  fix: string;
  details: string;
  kind:
    | "offline"
    | "network"
    | "checksum"
    | "disk"
    | "oom"
    | "unsupported"
    | "overwrite"
    | "generic";
}

export function mapTechnicalError(raw: string, t: Translations): FriendlyError {
  const details = String(raw || "Unknown error").trim();
  const lower = details.toLowerCase();

  if (lower.includes("never use the network") || lower.includes("offline lock")) {
    return {
      title: t.errOfflineTitle,
      fix: t.errOfflineFix,
      details,
      kind: "offline",
    };
  }

  if (
    lower.includes("sha-256") ||
    lower.includes("checksum") ||
    lower.includes("hash mismatch") ||
    lower.includes("integrity")
  ) {
    return {
      title: t.errChecksumTitle,
      fix: t.errChecksumFix,
      details,
      kind: "checksum",
    };
  }

  if (
    lower.includes("8007000e") ||
    lower.includes("out of memory") ||
    lower.includes("allocation") ||
    lower.includes("memory")
  ) {
    return {
      title: t.errOomTitle,
      fix: t.errOomFix,
      details,
      kind: "oom",
    };
  }

  if (lower.includes("overwrite") || lower.includes("on top of input")) {
    return {
      title: t.errOverwriteTitle,
      fix: t.errOverwriteFix,
      details,
      kind: "overwrite",
    };
  }

  if (
    lower.includes("unsupported") ||
    lower.includes("decode") ||
    lower.includes("corrupt") ||
    lower.includes("zero-byte") ||
    lower.includes("empty file") ||
    lower.includes("invalid image")
  ) {
    return {
      title: t.errUnsupportedTitle,
      fix: t.errUnsupportedFix,
      details,
      kind: "unsupported",
    };
  }

  if (
    lower.includes("dns") ||
    lower.includes("network") ||
    lower.includes("http") ||
    lower.includes("tls") ||
    lower.includes("timed out") ||
    lower.includes("connection") ||
    lower.includes("404") ||
    lower.includes("download")
  ) {
    return {
      title: t.errNetworkTitle,
      fix: t.errNetworkFix,
      details,
      kind: "network",
    };
  }

  if (
    lower.includes("permission denied") ||
    lower.includes("disk") ||
    lower.includes("space") ||
    lower.includes("os error 112") ||
    lower.includes("read-only") ||
    lower.includes("no such file") ||
    lower.includes("not accessible")
  ) {
    return {
      title: t.errDiskTitle,
      fix: t.errDiskFix,
      details,
      kind: "disk",
    };
  }

  return {
    title: t.errGenericTitle,
    fix: t.errGenericFix,
    details,
    kind: "generic",
  };
}
