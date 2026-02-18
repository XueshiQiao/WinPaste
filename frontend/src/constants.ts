import { isMacOS } from './utils/platform';

export const LAYOUT = {
  WINDOW_HEIGHT: isMacOS() ? 293 : 330, // keep sync with backend (constants.rs)
  CONTROL_BAR_HEIGHT: 64,
  CARD_WIDTH: 210,
  CARD_GAP: 24,
  SIDE_PADDING: 20,
  CARD_VERTICAL_PADDING: 16,
  PADDING_OPACITY: 0.2,
  WINDOW_PADDING: 16, // In pixels
  BLUR_AMOUNT: '8px', // Intensity of the blur
};

export const CLIP_LIST_HEIGHT = LAYOUT.WINDOW_HEIGHT - LAYOUT.CONTROL_BAR_HEIGHT;
export const TOTAL_COLUMN_WIDTH = LAYOUT.CARD_WIDTH + LAYOUT.CARD_GAP;
export const PREVIEW_CHAR_LIMIT = 300;
