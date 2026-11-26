import assert from "node:assert/strict";
import test from "node:test";
import {
  canAdvanceWizard,
  canCompleteWizard,
  createInitialWizardState,
  nextWizardStep,
  prevWizardStep,
  resetWizard,
  selectWizardModel,
} from "../src/lib/wizard.ts";

test("wizard state machine enforces mandatory answers and verified model before completion", () => {
  let s = createInitialWizardState();
  assert.equal(s.step, 1);

  // Step 1 -> Step 2 (language chosen)
  assert.equal(canAdvanceWizard(s, false), true);
  s = nextWizardStep(s, false);
  assert.equal(s.step, 2);

  // Cannot skip Step 2 without answering both questions
  assert.equal(canAdvanceWizard(s, false), false);
  s = { ...s, usage: "occasional" };
  assert.equal(canAdvanceWizard(s, false), false);
  s = nextWizardStep(s, false);
  assert.equal(s.step, 2);

  // Answer Question 2 -> advance to Step 3
  s = { ...s, photoType: "animals_hair_or_fur" };
  assert.equal(canAdvanceWizard(s, false), true);
  s = nextWizardStep(s, false);
  assert.equal(s.step, 3);

  // Cannot advance past Step 3 without a selected model
  assert.equal(canAdvanceWizard(s, false), false);
  s = selectWizardModel(s, "balanced");
  assert.equal(canAdvanceWizard(s, false), true);
  s = nextWizardStep(s, false);
  assert.equal(s.step, 4);

  // Cannot pass Step 4 or complete wizard until verified model is cached
  assert.equal(canAdvanceWizard(s, false), false);
  assert.equal(canCompleteWizard(s, false), false);
  s = nextWizardStep(s, false);
  assert.equal(s.step, 4);

  // Model downloaded/verified -> advance to Step 5 and complete
  assert.equal(canAdvanceWizard(s, true), true);
  s = nextWizardStep(s, true);
  assert.equal(s.step, 5);
  assert.equal(canCompleteWizard(s, true), true);
});

test("wizard Back and mid-way reset behave cleanly", () => {
  let s = createInitialWizardState();
  s = nextWizardStep(s, false);
  s = { ...s, usage: "regular_work", photoType: "products_plain_bg" };
  s = nextWizardStep(s, false);
  s = selectWizardModel(s, "fast");
  s = nextWizardStep(s, false);
  assert.equal(s.step, 4);

  s = prevWizardStep(s);
  assert.equal(s.step, 3);
  s = prevWizardStep(s);
  assert.equal(s.step, 2);
  s = prevWizardStep(s);
  assert.equal(s.step, 1);
  s = prevWizardStep(s);
  assert.equal(s.step, 1);

  // Closing mid-way restarts cleanly from Step 1
  const closed = resetWizard();
  assert.equal(closed.step, 1);
  assert.equal(closed.usage, null);
  assert.equal(closed.photoType, null);
  assert.equal(closed.selectedModelId, null);
});
