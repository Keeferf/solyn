import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Copy, X } from "lucide-react";

export const TitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const checkMaximized = async () => {
      try {
        const maximized = await appWindow.isMaximized();
        setIsMaximized(maximized);
      } catch {}
    };

    checkMaximized();

    const unlisten = appWindow.onResized(() => {
      appWindow
        .isMaximized()
        .then(setIsMaximized)
        .catch(() => {});
    });

    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [appWindow]);

  const handleMinimize = async () => {
    try {
      await appWindow.minimize();
    } catch {}
  };

  const handleMaximize = async () => {
    try {
      await appWindow.toggleMaximize();
      setIsMaximized(!isMaximized);
    } catch {}
  };

  const handleClose = async () => {
    try {
      await appWindow.close();
    } catch {}
  };

  return (
    <div
      data-tauri-drag-region="deep"
      className="fixed top-0 left-0 right-0 h-10 bg-black flex items-center justify-between px-3 z-50 select-none border-b border-white/5"
    >
      <div className="flex items-center gap-2">
        <span className="font-anton text-lg tracking-wider bg-linear-to-r from-purple-accent to-white/80 bg-clip-text text-transparent">
          Solyn
        </span>
      </div>

      <div className="flex items-stretch h-full -mr-3">
        <button
          onClick={handleMinimize}
          className="w-12 hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer"
          aria-label="Minimize to tray"
          type="button"
        >
          <Minus
            className="w-4 h-4 text-[#d8d4cf] hover:text-white transition-colors"
            strokeWidth={1.5}
          />
        </button>
        <button
          onClick={handleMaximize}
          className="w-12 hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer"
          aria-label={isMaximized ? "Restore" : "Maximize"}
          type="button"
        >
          {isMaximized ? (
            <Copy
              className="w-4 h-4 text-[#d8d4cf] hover:text-white transition-colors"
              strokeWidth={1.5}
            />
          ) : (
            <Square
              className="w-4 h-4 text-[#d8d4cf] hover:text-white transition-colors"
              strokeWidth={1.5}
            />
          )}
        </button>
        <button
          onClick={handleClose}
          className="w-12 hover:bg-red-500 flex items-center justify-center transition-colors cursor-pointer"
          aria-label="Close application"
          type="button"
        >
          <X
            className="w-4 h-4 text-[#d8d4cf] hover:text-white transition-colors"
            strokeWidth={1.5}
          />
        </button>
      </div>
    </div>
  );
};
