export const CUTOUT_ONLY_POS = 0;
export const DEFAULT_COMPARE_POS = 50;

export interface SliderCompareState {
  sliderPos: number;
  lastComparePos: number;
}

/**
 * Starts on cutout-only so completed results reveal the cutout immediately
 * while preserving a sensible 50% split target for manual comparison.
 */
export function createInitialSliderState(): SliderCompareState {
  return {
    sliderPos: CUTOUT_ONLY_POS,
    lastComparePos: DEFAULT_COMPARE_POS,
  };
}

export function isComparingWithOriginal(state: SliderCompareState): boolean {
  return state.sliderPos > CUTOUT_ONLY_POS;
}

export function setSliderPosition(
  state: SliderCompareState,
  nextPos: number
): SliderCompareState {
  const clamped = Math.max(0, Math.min(100, nextPos));
  return {
    sliderPos: clamped,
    lastComparePos: clamped > CUTOUT_ONLY_POS ? clamped : state.lastComparePos,
  };
}

export function toggleCompareWithOriginal(
  state: SliderCompareState
): SliderCompareState {
  if (isComparingWithOriginal(state)) {
    return {
      sliderPos: CUTOUT_ONLY_POS,
      lastComparePos: state.sliderPos,
    };
  }
  const target =
    state.lastComparePos > CUTOUT_ONLY_POS
      ? state.lastComparePos
      : DEFAULT_COMPARE_POS;
  return {
    sliderPos: target,
    lastComparePos: target,
  };
}

export function resetSliderForNewPhoto(
  state: SliderCompareState
): SliderCompareState {
  return {
    sliderPos: CUTOUT_ONLY_POS,
    lastComparePos: state.lastComparePos,
  };
}
