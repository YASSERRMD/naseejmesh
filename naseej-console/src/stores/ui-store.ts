import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SidebarState {
    isCollapsed: boolean;
    toggle: () => void;
    setCollapsed: (collapsed: boolean) => void;
}

export const useSidebarStore = create<SidebarState>()(
    persist(
        (set) => ({
            isCollapsed: false,
            toggle: () => set((state) => ({ isCollapsed: !state.isCollapsed })),
            setCollapsed: (collapsed) => set({ isCollapsed: collapsed }),
        }),
        {
            name: "sidebar-state",
        }
    )
);

interface ThemeState {
    theme: "light" | "dark" | "system";
    setTheme: (theme: "light" | "dark" | "system") => void;
}

export const useThemeStore = create<ThemeState>()(
    persist(
        (set) => ({
            theme: "system",
            setTheme: (theme) => set({ theme }),
        }),
        {
            name: "theme-state",
        }
    )
);
