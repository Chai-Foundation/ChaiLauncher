import { useEffect } from 'react';

interface KeyboardShortcuts {
  onUpload?: () => void;
  onRefresh?: () => void;
  onSearch?: () => void;
  onDelete?: () => void;
  onSelectAll?: () => void;
  onEscape?: () => void;
  onNewFolder?: () => void;
}

export const useKeyboardShortcuts = (shortcuts: KeyboardShortcuts) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const { ctrlKey, metaKey, shiftKey, key } = event;
      const isModifierPressed = ctrlKey || metaKey;

      // Prevent default behavior for our shortcuts
      const preventDefault = () => {
        event.preventDefault();
        event.stopPropagation();
      };

      // Upload files (Ctrl/Cmd + U)
      if (isModifierPressed && key.toLowerCase() === 'u' && shortcuts.onUpload) {
        preventDefault();
        shortcuts.onUpload();
        return;
      }

      // Refresh (Ctrl/Cmd + R or F5)
      if ((isModifierPressed && key.toLowerCase() === 'r') || key === 'F5') {
        if (shortcuts.onRefresh) {
          preventDefault();
          shortcuts.onRefresh();
        }
        return;
      }

      // Search (Ctrl/Cmd + F)
      if (isModifierPressed && key.toLowerCase() === 'f' && shortcuts.onSearch) {
        preventDefault();
        shortcuts.onSearch();
        return;
      }

      // Select all (Ctrl/Cmd + A)
      if (isModifierPressed && key.toLowerCase() === 'a' && shortcuts.onSelectAll) {
        preventDefault();
        shortcuts.onSelectAll();
        return;
      }

      // Delete (Delete key)
      if (key === 'Delete' && shortcuts.onDelete) {
        preventDefault();
        shortcuts.onDelete();
        return;
      }

      // Escape
      if (key === 'Escape' && shortcuts.onEscape) {
        preventDefault();
        shortcuts.onEscape();
        return;
      }

      // New folder (Ctrl/Cmd + Shift + N)
      if (isModifierPressed && shiftKey && key.toLowerCase() === 'n' && shortcuts.onNewFolder) {
        preventDefault();
        shortcuts.onNewFolder();
        return;
      }
    };

    // Add event listener
    document.addEventListener('keydown', handleKeyDown);

    // Cleanup
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [shortcuts]);
};