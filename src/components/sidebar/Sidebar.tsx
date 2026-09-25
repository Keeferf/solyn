import { SidebarItem } from "./SidebarItem";
import { NAVIGATION_ITEMS, FOOTER_ITEMS } from "./SidebarNavigation";
import { OllamaVersionIndicator } from "./OllamaVersionIndicator";
import { ThemeSwitcher } from "./ThemeSwitcher";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useChatStore } from "@/stores/chatStore";

interface SidebarProps {
  onNavigate?: (view: "chat" | "history" | "models") => void;
  currentView?: "chat" | "history" | "models";
  isCollapsed?: boolean;
  onToggleCollapse?: () => void;
}

export const Sidebar = ({
  onNavigate,
  currentView = "chat",
  isCollapsed = false,
  onToggleCollapse,
}: SidebarProps) => {
  const { startNewChat } = useChatStore();

  const handleNavigation = (id: string) => {
    if (id === "models") {
      onNavigate?.("models");
    } else if (id === "chats") {
      onNavigate?.("history");
    } else if (id === "new-chat") {
      startNewChat();
      onNavigate?.("chat");
    } else if (id === "search") {
      onNavigate?.("chat");
    }
  };

  const isItemActive = (itemId: string) => {
    if (itemId === "models") {
      return currentView === "models";
    }
    if (itemId === "chats") {
      return currentView === "history";
    }
    return false;
  };

  return (
    <aside
      className={`fixed left-0 top-10 h-[calc(100vh-40px)] bg-black border-r border-white/10 flex flex-col p-4 z-30 transition-all duration-300 ${
        isCollapsed ? "w-16" : "w-64"
      }`}
    >
      <div
        className={`mb-6 flex items-center ${isCollapsed ? "justify-center" : "justify-between"}`}
      >
        {!isCollapsed && (
          <h2 className="text-2xl font-bold font-anton bg-linear-to-r from-purple-accent to-white/80 bg-clip-text text-transparent px-3">
            Solyn
          </h2>
        )}
        <button
          onClick={onToggleCollapse}
          className={`rounded-lg transition-all duration-200 text-white/40 hover:bg-white/5 hover:text-white flex items-center justify-center cursor-pointer ${
            isCollapsed ? "w-10 h-10 mx-auto" : "h-10 w-10"
          }`}
          title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {isCollapsed ? (
            <ChevronRight size={20} />
          ) : (
            <ChevronLeft size={20} />
          )}
        </button>
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto">
        {NAVIGATION_ITEMS.map((item) => (
          <SidebarItem
            key={item.id}
            icon={<item.icon size={20} />}
            label={item.label}
            active={isItemActive(item.id)}
            onClick={() => handleNavigation(item.id)}
            collapsed={isCollapsed}
          />
        ))}

        <div className="pt-4 mt-4 border-t border-white/10">
          {/* Theme Switcher */}
          <ThemeSwitcher collapsed={isCollapsed} />

          {FOOTER_ITEMS.map((item) => (
            <SidebarItem
              key={item.id}
              icon={<item.icon size={20} />}
              label={item.label}
              disabled={item.disabled}
              collapsed={isCollapsed}
            />
          ))}
        </div>
      </nav>

      {!isCollapsed && (
        <div className="mt-auto pt-4 border-t border-white/10">
          <div className="flex items-center justify-between px-3 py-2">
            <span className="text-sm text-white/40">v0.2.2</span>
            <OllamaVersionIndicator />
          </div>
        </div>
      )}
    </aside>
  );
};
