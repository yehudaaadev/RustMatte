import assert from "node:assert/strict";
import test from "node:test";
import { mapTechnicalError } from "../src/lib/errors.ts";
import { dictionaries } from "../src/lib/i18n.ts";

test("maps technical failures to friendly titles, fixes, and technical details", () => {
  const cases: Array<[string, ReturnType<typeof mapTechnicalError>["kind"]]> = [
    ["Offline lock is enabled ('Never use the network')", "offline"],
    ["SHA-256 checksum mismatch for u2netp.onnx", "checksum"],
    ["DirectML error 8007000E out of memory allocation failed", "oom"],
    ["Refusing to overwrite an input image file", "overwrite"],
    ["Unsupported image format for doc.pdf", "unsupported"],
    ["HTTP download failed: connection timed out", "network"],
    ["Permission denied writing to output folder", "disk"],
  ];

  for (const [raw, expectedKind] of cases) {
    for (const lang of ["en", "id"] as const) {
      const mapped = mapTechnicalError(raw, dictionaries[lang]);
      assert.equal(mapped.kind, expectedKind);
      assert.ok(mapped.title.trim().length > 0);
      assert.ok(mapped.fix.trim().length > 0);
      assert.equal(mapped.details, raw);
    }
  }
});
