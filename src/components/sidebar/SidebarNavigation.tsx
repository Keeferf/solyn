import {
  Plus,
  Search,
  MessageSquare,
  Ellipsis,
  Layers,
} from "lucide-react";

export const NAVIGATION_ITEMS = [
  { id: "new-chat", icon: Plus, label: "New Chat" },
  { id: "search", icon: Search, label: "Search" },
  { id: "chats", icon: MessageSquare, label: "Chats" },
  { id: "models", icon: Layers, label: "Models" },
];

export const FOOTER_ITEMS = [
  {
    id: "more",
    icon: Ellipsis,
    label: "More features coming",
    disabled: true,
  },
];
