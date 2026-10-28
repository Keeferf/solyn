import { useEffect, useState } from "react";

interface UseSkeletonCountOptions {
  /** Min width of one item, mirrors the CSS grid track size. */
  itemMinWidth?: number;
  /** Estimated rendered height of one skeleton item. */
  itemHeight: number;
  /** Gap between items, in px. */
  gap: number;
  /** Fixed column count. Omit to derive columns from container width. */
  columns?: number;
  minCount?: number;
  maxCount?: number;
  /** Space to leave below the last row so skeletons do not touch the edge. */
  bottomPadding?: number;
}

/**
 * Count of skeleton placeholders needed to fill the remaining viewport space.
 * Attach the returned ref to the skeleton container.
 */
export function useSkeletonCount<T extends HTMLElement>({
  itemMinWidth = 280,
  itemHeight,
  gap,
  columns,
  minCount = 1,
  maxCount = 60,
  bottomPadding = 16,
}: UseSkeletonCountOptions) {
  const [element, setElement] = useState<T | null>(null);
  const [count, setCount] = useState(minCount);

  useEffect(() => {
    if (!element) return;

    const update = () => {
      const width = element.clientWidth;
      if (!width) return;

      const columnCount =
        columns ??
        Math.max(1, Math.floor((width + gap) / (itemMinWidth + gap)));

      const availableHeight = Math.max(
        0,
        window.innerHeight - element.getBoundingClientRect().top - bottomPadding,
      );
      const rowCount = Math.max(
        1,
        Math.ceil(availableHeight / (itemHeight + gap)),
      );

      const next = Math.min(
        maxCount,
        Math.max(minCount, columnCount * rowCount),
      );
      setCount((prev) => (prev === next ? prev : next));
    };

    update();

    const observer = new ResizeObserver(update);
    observer.observe(element);
    window.addEventListener("resize", update);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", update);
    };
  }, [
    element,
    itemMinWidth,
    itemHeight,
    gap,
    columns,
    minCount,
    maxCount,
    bottomPadding,
  ]);

  return { ref: setElement, count };
}
