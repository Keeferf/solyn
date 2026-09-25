import { useState, useRef, useEffect } from "react";
import {
  Search,
  X,
  CloudDownload,
  ThumbsUp,
  Clock,
} from "lucide-react";
import { useMaintainFocus } from "./hooks/useMaintainFocus";

interface ModelToolbarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onClearSearch: () => void;
  currentFilter: string;
  onFilterChange: (filter: string) => void;
  loading?: boolean;
  disabled?: boolean;
  placeholder?: string;
  filterOptions?: Array<{
    value: string;
    label: string;
    icon: React.ComponentType<{ size?: number }>;
  }>;
}

const defaultFilterOptions = [
  { value: "most_downloads", label: "Most Downloads", icon: CloudDownload },
  { value: "most_liked", label: "Most Liked", icon: ThumbsUp },
  { value: "recent", label: "Recent", icon: Clock },
];

const MIN_SEARCH_CHARS = 3;

export const ModelToolbar = ({
  searchQuery,
  onSearchChange,
  onClearSearch,
  currentFilter,
  onFilterChange,
  loading = false,
  disabled = false,
  placeholder = "Search models by name, author, or description...",
  filterOptions = defaultFilterOptions,
}: ModelToolbarProps) => {
  const [localQuery, setLocalQuery] = useState(searchQuery);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useMaintainFocus<HTMLInputElement>();

  useEffect(() => {
    if (loading && inputRef.current) {
      const input = inputRef.current;

      const restoreFocus = () => {
        if (
          input &&
          document.activeElement !== input &&
          document.contains(input)
        ) {
          setTimeout(() => {
            if (
              input &&
              document.activeElement !== input &&
              document.contains(input)
            ) {
              input.focus();
            }
          }, 10);
        }
      };

      const handleFocusOut = (_e: FocusEvent) => {
        if (loading && input && document.activeElement !== input) {
          restoreFocus();
        }
      };

      const observer = new MutationObserver(() => {
        if (
          loading &&
          input &&
          document.activeElement !== input &&
          document.contains(input)
        ) {
          restoreFocus();
        }
      });

      const parent = input.parentElement?.parentElement;
      if (parent) {
        observer.observe(parent, {
          childList: true,
          subtree: true,
          attributes: true,
        });
      }

      document.addEventListener("focusout", handleFocusOut);

      return () => {
        document.removeEventListener("focusout", handleFocusOut);
        observer.disconnect();
      };
    }
  }, [loading, inputRef]);

  useEffect(() => {
    if (searchQuery === "" && localQuery !== "") {
      setLocalQuery("");
    }
  }, [searchQuery]);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;

    setLocalQuery(newValue);

    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    debounceTimerRef.current = setTimeout(() => {
      if (newValue.length >= MIN_SEARCH_CHARS || newValue === "") {
        onSearchChange(newValue);
      }
    }, 300);
  };

  const handleClear = () => {
    setLocalQuery("");
    onClearSearch();
  };

  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  return (
    <div className="flex flex-col sm:flex-row gap-3">
      <div className="relative flex-1">
        <div className="relative">
          <Search
            className="absolute left-3 top-1/2 -translate-y-1/2 text-white/30"
            size={16}
          />
          <input
            ref={inputRef}
            type="text"
            value={localQuery}
            onChange={handleInputChange}
            placeholder={placeholder}
            className="w-full bg-black/50 border border-white/10 rounded-lg px-10 py-2.5 text-white text-sm placeholder:text-white/30 focus:outline-none focus:border-purple-accent focus:ring-2 focus:ring-purple-accent transition-all"
            disabled={disabled || loading}
          />
          {localQuery && (
            <button
              onClick={handleClear}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60 transition-colors cursor-pointer"
              aria-label="Clear search"
            >
              <X size={16} />
            </button>
          )}
        </div>
        {localQuery.length > 0 && localQuery.length < MIN_SEARCH_CHARS && (
          <p className="text-xs text-white/30 mt-1 ml-3">
            Type at least {MIN_SEARCH_CHARS} characters to search
          </p>
        )}
      </div>

      <div className="flex items-center gap-2 flex-wrap shrink-0">
        {filterOptions.map(({ value, label, icon: Icon }) => (
          <button
            key={value}
            onClick={() => onFilterChange(value)}
            disabled={disabled || loading}
            className={`px-3 py-2.5 rounded-lg text-xs transition-all flex items-center gap-1.5 cursor-pointer whitespace-nowrap ${
              disabled || loading ? "opacity-50 cursor-not-allowed" : ""
            } ${
              currentFilter === value
                ? "bg-purple-accent text-white border border-purple-accent"
                : "bg-black/50 text-white/60 hover:bg-white/10 hover:text-white border border-white/10"
            }`}
          >
            <Icon size={12} />
            {label}
          </button>
        ))}
      </div>
    </div>
  );
};
