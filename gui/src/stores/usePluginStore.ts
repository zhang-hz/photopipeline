import { create } from "zustand";
import type { PluginEntry, NodeSchemaResponse } from "../types/plugin";
import { MOCK_PLUGINS } from "../data/mockPlugins";

interface PluginState {
  plugins: PluginEntry[];
  categories: string[];
  nodeSchemas: Map<string, NodeSchemaResponse>;
  searchQuery: string;
  categoryFilter: string | "All";
  isLoading: boolean;
  _initialized: boolean;
}

interface PluginActions {
  fetchPlugins: () => Promise<void>;
  fetchNodeSchema: (pluginId: string) => Promise<NodeSchemaResponse | null>;
  setSearch: (query: string) => void;
  setCategoryFilter: (category: string | "All") => void;
  initMockData: () => void;
}

const CATEGORIES = [...new Set(MOCK_PLUGINS.map((p) => p.category))].sort();

export const usePluginStore = create<PluginState & PluginActions>((set, get) => ({
  plugins: [],
  categories: [],
  nodeSchemas: new Map(),
  searchQuery: "",
  categoryFilter: "All",
  isLoading: false,
  _initialized: false,

  initMockData: () => {
    if (get()._initialized) return;
    set({
      plugins: MOCK_PLUGINS,
      categories: CATEGORIES,
      _initialized: true,
    });
  },

  fetchPlugins: async () => {
    set({ isLoading: true });
    // In production: invoke("list_plugins")
    if (!get()._initialized) {
      set({ plugins: MOCK_PLUGINS, categories: CATEGORIES, _initialized: true });
    }
    set({ isLoading: false });
  },

  fetchNodeSchema: async (pluginId: string) => {
    const cached = get().nodeSchemas.get(pluginId);
    if (cached) return cached;
    // TODO: invoke("get_node_schema", { pluginId })
    return null;
  },

  setSearch: (searchQuery) => set({ searchQuery }),
  setCategoryFilter: (categoryFilter) => set({ categoryFilter }),
}));

// Derived: filtered plugins
export function useFilteredPlugins(): PluginEntry[] {
  const { plugins, searchQuery, categoryFilter } = usePluginStore();
  return plugins.filter((p) => {
    if (categoryFilter !== "All" && p.category !== categoryFilter) return false;
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      return (
        p.name.toLowerCase().includes(q) ||
        p.tags.some((t) => t.toLowerCase().includes(q))
      );
    }
    return true;
  });
}
