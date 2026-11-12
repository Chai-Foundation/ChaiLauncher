import React, { useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

// Not exported by @tauri-apps/api, but it is the accepted
// `startResizeDragging` argument on every platform.
type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

const EDGE_SIZE = 6;
const CORNER_SIZE = 12;

interface ResizeZone {
  direction: ResizeDirection;
  className: string;
  style: React.CSSProperties;
}

const zones: ResizeZone[] = [
  // Edges
  {
    direction: 'North',
    className: 'cursor-ns-resize',
    style: { top: 0, left: 0, right: 0, height: EDGE_SIZE },
  },
  {
    direction: 'South',
    className: 'cursor-ns-resize',
    style: { bottom: 0, left: 0, right: 0, height: EDGE_SIZE },
  },
  {
    direction: 'West',
    className: 'cursor-ew-resize',
    style: { top: 0, bottom: 0, left: 0, width: EDGE_SIZE },
  },
  {
    direction: 'East',
    className: 'cursor-ew-resize',
    style: { top: 0, bottom: 0, right: 0, width: EDGE_SIZE },
  },
  // Corners (rendered above the edges)
  {
    direction: 'NorthWest',
    className: 'cursor-nwse-resize',
    style: { top: 0, left: 0, width: CORNER_SIZE, height: CORNER_SIZE },
  },
  {
    direction: 'SouthEast',
    className: 'cursor-nwse-resize',
    style: { bottom: 0, right: 0, width: CORNER_SIZE, height: CORNER_SIZE },
  },
  {
    direction: 'NorthEast',
    className: 'cursor-nesw-resize',
    style: { top: 0, right: 0, width: CORNER_SIZE, height: CORNER_SIZE },
  },
  {
    direction: 'SouthWest',
    className: 'cursor-nesw-resize',
    style: { bottom: 0, left: 0, width: CORNER_SIZE, height: CORNER_SIZE },
  },
];

/**
 * Invisible resize grips for the undecorated window.
 *
 * A frameless window has no native resize borders on Linux (GTK) and no
 * reliable ones on Windows/macOS, so we drive `startResizeDragging` ourselves.
 * The grips are pointer-transparent above the window chrome and are ignored
 * while the window is maximized or fullscreen.
 */
const ResizeHandles: React.FC = () => {
  const handlePointerDown = useCallback(
    async (event: React.PointerEvent, direction: ResizeDirection) => {
      if (event.button !== 0) return;
      event.preventDefault();
      event.stopPropagation();

      try {
        // Only a non-blocking check before starting the drag: the IPC call must
        // be the first thing that grabs the pointer. Any extra round-trip here
        // releases the grab and the drag never starts (leaving GTK and the
        // compositor disagreeing about the window size).
        const win = getCurrentWindow();
        const maximized = await win.isMaximized().catch(() => false);
        if (maximized) return;
        await win.startResizeDragging(direction);
      } catch (error) {
        console.error(`Failed to start ${direction} resize:`, error);
      }
    },
    []
  );

  return (
    <>
      {zones.map((zone) => (
        <div
          key={zone.direction}
          aria-hidden="true"
          className={`fixed z-[9999] ${zone.className}`}
          style={zone.style}
          onPointerDown={(event) => handlePointerDown(event, zone.direction)}
        />
      ))}
    </>
  );
};

export default ResizeHandles;
