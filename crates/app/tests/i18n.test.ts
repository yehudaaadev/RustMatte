import assert from "node:assert/strict";
import test from "node:test";
import { dictionaries } from "../src/lib/i18n.ts";

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
