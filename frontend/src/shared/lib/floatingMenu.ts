/** Viewport padding for fixed context menus / floating panels. */
export const FLOATING_MENU_PAD = 8;

export interface FloatingMenuPlacement {
  maxHeight: number;
  x: number;
  y: number;
}

/**
 * Place a fixed menu at an anchor point, clamp into the viewport, and cap height
 * so the panel can scroll inside the remaining space instead of hanging off-screen.
 *
 * When `menuSize` is known (after layout), position is refined so the panel fits.
 * When only the anchor is known, `maxHeight` uses the larger of space above/below.
 */
export function placeFloatingMenu(
  anchorX: number,
  anchorY: number,
  menuSize?: { height: number; width: number },
  pad: number = FLOATING_MENU_PAD,
): FloatingMenuPlacement {
  const viewportWidth = typeof window === "undefined" ? 1280 : window.innerWidth;
  const viewportHeight = typeof window === "undefined" ? 720 : window.innerHeight;

  const spaceBelow = Math.max(0, viewportHeight - anchorY - pad);
  const spaceAbove = Math.max(0, anchorY - pad);
  // Prefer opening downward unless there is clearly more room above.
  const openDown = spaceBelow >= spaceAbove || spaceBelow >= 200;
  const availableHeight = Math.max(openDown ? spaceBelow : spaceAbove, 160);

  if (!menuSize) {
    return {
      x: clamp(anchorX, pad, Math.max(pad, viewportWidth - pad)),
      y: openDown ? anchorY : Math.max(pad, anchorY - availableHeight),
      maxHeight: availableHeight,
    };
  }

  const width = menuSize.width;
  const height = Math.min(menuSize.height, availableHeight);
  let x = anchorX;
  let y = openDown ? anchorY : anchorY - height;

  if (x + width > viewportWidth - pad) {
    x = Math.max(pad, viewportWidth - width - pad);
  }
  if (x < pad) {
    x = pad;
  }
  if (y + height > viewportHeight - pad) {
    y = Math.max(pad, viewportHeight - height - pad);
  }
  if (y < pad) {
    y = pad;
  }

  // After clamping Y, recompute max height from the final top edge so resize stays valid.
  const maxHeight = Math.max(160, viewportHeight - y - pad);

  return { x, y, maxHeight };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
