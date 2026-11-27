import assert from "node:assert/strict";
import test from "node:test";
import {
  BANNER_CODES,
  REJECTED_REASON_CODES,
  formatDisplayPath,
  renderBannerByCode,
  renderRejectedReason,
  renderWarningItem,
} from "../src/lib/banners.ts";
import { dictionaries } from "../src/lib/i18n.ts";
import { SUPPORTED_IMAGE_EXTENSIONS } from "../src/lib/formats.ts";

test("English and Indonesian dictionaries have identical keys and non-empty values", () => {
  const enKeys = Object.keys(dictionaries.en).sort();
  const idKeys = Object.keys(dictionaries.id).sort();

  assert.deepEqual(idKeys, enKeys);

  for (const key of enKeys) {
    const enVal = dictionaries.en[key as keyof typeof dictionaries.en];
    const idVal = dictionaries.id[key as keyof typeof dictionaries.id];
    assert.equal(typeof enVal, "string");
    assert.equal(typeof idVal, "string");
    assert.ok(enVal.trim().length > 0, `en.${key} must not be empty`);
    assert.ok(idVal.trim().length > 0, `id.${key} must not be empty`);
  }
});

test("every known banner code, rejection code, and warning code has distinct localized EN and ID text", () => {
  for (const code of BANNER_CODES) {
    const en = renderBannerByCode(code, dictionaries.en);
    const id = renderBannerByCode(code, dictionaries.id);
    assert.ok(en.trim().length > 0, `Missing EN banner translation for ${code}`);
    assert.ok(id.trim().length > 0, `Missing ID banner translation for ${code}`);
    assert.ok(!en.includes("{"), `Unreplaced template token in EN banner ${code}: ${en}`);
    assert.ok(!id.includes("{"), `Unreplaced template token in ID banner ${code}: ${id}`);
    assert.ok(id !== en, `ID banner ${code} must not fall back to EN text`);
  }

  for (const reasonCode of REJECTED_REASON_CODES) {
    const en = renderRejectedReason(
      { path: "/tmp/x", name: "x", reason_code: reasonCode, reason: "fallback" },
      dictionaries.en
    );
    const id = renderRejectedReason(
      { path: "/tmp/x", name: "x", reason_code: reasonCode, reason: "fallback" },
      dictionaries.id
    );
    assert.ok(en.trim().length > 0);
    assert.ok(id.trim().length > 0);
    assert.ok(id !== en, `ID rejected reason ${reasonCode} must be translated`);
  }

  const warnEn = renderWarningItem(
    { code: "damaged_input", detail: "truncated header" },
    dictionaries.en
  );
  const warnId = renderWarningItem(
    { code: "damaged_input", detail: "truncated header" },
    dictionaries.id
  );
  assert.ok(warnEn.includes("truncated header"));
  assert.ok(warnId.includes("truncated header"));
  assert.ok(warnId !== warnEn, "damaged_input warning must be translated to ID");
});

test("formatDisplayPath strips Windows extended-length prefixes cleanly", () => {
  assert.equal(
    formatDisplayPath("\\\\?\\C:\\Users\\Admin\\Downloads"),
    "C:\\Users\\Admin\\Downloads"
  );
  assert.equal(
    formatDisplayPath("\\\\?\\UNC\\server\\share\\folder"),
    "\\\\server\\share\\folder"
  );
  assert.equal(formatDisplayPath("/Users/admin/Downloads"), "/Users/admin/Downloads");
});

test("wizard recommendation reasons and model card tags translate cleanly in EN and ID", () => {
  const pairedKeys: (keyof typeof dictionaries.en)[] = [
    "wizardReasonOccasionalBalanced",
    "wizardReasonBatchSimpleFast",
    "wizardReasonBatchDetailBalanced",
    "wizardReasonDefaultBalanced",
    "wizardReasonLowRamOverride",
    "wizardRamMidNote",
    "wizardRamLowWarning",
    "wizardRamUnknown",
    "cardTagHairFur",
    "cardTagGeneral",
    "cardTagVeryFast",
    "cardTagLowRam",
    "cardTagTinyDownload",
    "cardTagClosesSmallGaps",
    "cardTagLargerDownload",
    "cardTagFurSmears",
    "cardTagFineHair",
  ];
  for (const key of pairedKeys) {
    assert.ok(dictionaries.en[key].trim().length > 0);
    assert.ok(dictionaries.id[key].trim().length > 0);
    assert.ok(
      dictionaries.id[key] !== dictionaries.en[key],
      `Expected distinct Indonesian translation for ${key}`
    );
  }
});

test("supported image extensions and format copy match v1.1 Phase 1 contract", () => {
  assert.deepEqual([...SUPPORTED_IMAGE_EXTENSIONS], [
    "jpg",
    "jpeg",
    "png",
    "webp",
    "bmp",
    "cr2",
    "nef",
    "arw",
    "dng",
    "heic",
    "heif",
  ]);

  const formatKeys: (keyof typeof dictionaries.en)[] = [
    "dropSubtitle",
    "rejectedReasonUnsupported",
    "errUnsupportedFix",
  ];
  for (const key of formatKeys) {
    for (const lang of ["en", "id"] as const) {
      const text = dictionaries[lang][key];
      assert.ok(text.includes("HEIC/HEIF"), `${lang}.${key} must mention HEIC/HEIF`);
      assert.ok(
        text.includes("CR2") && text.includes("NEF") && text.includes("ARW") && text.includes("DNG"),
        `${lang}.${key} must mention RAW extensions (CR2, NEF, ARW, DNG)`
      );
    }
  }
});
