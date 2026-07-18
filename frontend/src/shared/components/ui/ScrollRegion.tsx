import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
  type Ref,
  type UIEventHandler,
} from "react";

import { cn } from "@/shared/lib/utils";

export type ScrollCue = {
  top: boolean;
  bottom: boolean;
};

type ScrollRegionProps = {
  children: ReactNode;
  className?: string;
  contentClassName?: string;
  /** Classes on the scrollable viewport (defaults hide the native scrollbar). */
  scrollClassName?: string;
  /**
   * Extra classes for the top fade cue (e.g. `top-11` so it sits under a sticky table header
   * instead of covering it). Defaults to top of the scrollport.
   */
  topCueClassName?: string;
  /** Inline styles for the top fade cue (prefer measured header height for a flush join). */
  topCueStyle?: CSSProperties;
  "aria-label"?: string;
  onScroll?: UIEventHandler<HTMLDivElement>;
  /** Optional external ref to the scrollable element (e.g. virtualization). */
  scrollRef?: Ref<HTMLDivElement | null>;
};

function assignRef<T>(ref: Ref<T | null> | undefined, value: T | null): void {
  if (!ref) {
    return;
  }
  if (typeof ref === "function") {
    ref(value);
    return;
  }
  ref.current = value;
}

/**
 * Modular scroll container (product standard).
 *
 * - Hides the native scrollbar (`.scroll-region-viewport` in index.css)
 * - Shows top/bottom fade + chevron heads when more content exists
 * - Theme-aware (light/dark via `from-card` / `text-foreground`)
 *
 * Usage:
 * - Full-height pane: `className="min-h-0 flex-1"` (default includes flex-1)
 * - Capped menus: `className="max-h-[min(20rem,calc(100vh-8rem))]"`
 * - Virtualized table: pass `scrollRef` + `onScroll`; optional `topCueStyle` under sticky headers
 *
 * Confirmed on the main inventory table — reuse elsewhere without changing table wiring.
 */
export function ScrollRegion({
  children,
  className,
  contentClassName,
  scrollClassName,
  topCueClassName,
  topCueStyle,
  "aria-label": ariaLabel,
  onScroll,
  scrollRef: externalScrollRef,
}: ScrollRegionProps) {
  const internalScrollRef = useRef<HTMLDivElement | null>(null);
  const [scrollCue, setScrollCue] = useState<ScrollCue>({ top: false, bottom: false });

  const setScrollNode = useCallback(
    (node: HTMLDivElement | null) => {
      internalScrollRef.current = node;
      assignRef(externalScrollRef, node);
    },
    [externalScrollRef],
  );

  const updateScrollCue = useCallback(() => {
    const element = internalScrollRef.current;
    if (!element) {
      setScrollCue({ top: false, bottom: false });
      return;
    }

    const overflow = element.scrollHeight > element.clientHeight + 1;
    const atTop = element.scrollTop <= 1;
    const atBottom = element.scrollTop + element.clientHeight >= element.scrollHeight - 1;
    const nextCue = {
      top: overflow && !atTop,
      bottom: overflow && !atBottom,
    };

    setScrollCue((current) =>
      current.top === nextCue.top && current.bottom === nextCue.bottom ? current : nextCue,
    );
  }, []);

  useEffect(() => {
    updateScrollCue();
    const element = internalScrollRef.current;
    if (!element || typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(() => updateScrollCue());
    observer.observe(element);
    if (element.firstElementChild) {
      observer.observe(element.firstElementChild);
    }
    return () => observer.disconnect();
  }, [updateScrollCue, children]);

  const handleScroll: UIEventHandler<HTMLDivElement> = (event) => {
    updateScrollCue();
    onScroll?.(event);
  };

  return (
    // flex-col + min-h-0 so max-height parents (dropdowns) actually constrain the viewport
    <div className={cn("relative flex min-h-0 flex-1 flex-col overflow-hidden", className)}>
      {scrollCue.top ? (
        <div
          className={cn(
            "pointer-events-none absolute inset-x-0 top-0 z-20 flex h-8 items-start justify-center bg-gradient-to-b from-card via-card/85 to-transparent pt-1",
            topCueClassName,
          )}
          style={topCueStyle}
        >
          <ChevronUpIcon aria-hidden className="size-3.5 text-foreground/50" strokeWidth={2.5} />
        </div>
      ) : null}
      <div
        ref={setScrollNode}
        aria-label={ariaLabel}
        className={cn(
          // Prefer flex-1/min-h-0 over h-full: h-full fails when parent only has max-height.
          "scroll-region-viewport min-h-0 flex-1 overflow-y-auto overflow-x-hidden",
          scrollClassName,
        )}
        onScroll={handleScroll}
      >
        <div className={cn(contentClassName)}>{children}</div>
      </div>
      {scrollCue.bottom ? (
        <div className="pointer-events-none absolute inset-x-0 bottom-0 z-20 flex h-8 items-end justify-center bg-gradient-to-t from-card via-card/85 to-transparent pb-1">
          <ChevronDownIcon aria-hidden className="size-3.5 text-foreground/50" strokeWidth={2.5} />
        </div>
      ) : null}
    </div>
  );
}
