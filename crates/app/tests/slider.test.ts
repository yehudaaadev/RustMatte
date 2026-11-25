import assert from "node:assert/strict";
import test from "node:test";
import {
  CUTOUT_ONLY_POS,
  DEFAULT_COMPARE_POS,
  createInitialSliderState,
  isComparingWithOriginal,
  resetSliderForNewPhoto,
  setSliderPosition,
  toggleCompareWithOriginal,
} from "../src/lib/slider.ts";

test("slider defaults to cutout-only, remembers compare split in session, and resets per photo", () => {
  let state = createInitialSliderState();
  assert.equal(state.sliderPos, CUTOUT_ONLY_POS);
  assert.equal(isComparingWithOriginal(state), false);

  state = toggleCompareWithOriginal(state);
  assert.equal(state.sliderPos, DEFAULT_COMPARE_POS);
  assert.equal(isComparingWithOriginal(state), true);

  state = setSliderPosition(state, 72);
  assert.equal(state.sliderPos, 72);

  state = toggleCompareWithOriginal(state);
  assert.equal(state.sliderPos, CUTOUT_ONLY_POS);
  assert.equal(isComparingWithOriginal(state), false);

  state = toggleCompareWithOriginal(state);
  assert.equal(state.sliderPos, 72);

  state = resetSliderForNewPhoto(state);
  assert.equal(state.sliderPos, CUTOUT_ONLY_POS);
  assert.equal(isComparingWithOriginal(state), false);

  state = toggleCompareWithOriginal(state);
  assert.equal(state.sliderPos, 72);
});
