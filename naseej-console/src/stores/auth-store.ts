import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface User {
    id: string;
    username: string;
    roles: string[];
}

interface AuthState {
    token: string | null;
    user: User | null;
    expiresAt: number | null;
    setAuth: (token: string, user: User, expiresAt: number) => void;
    logout: () => void;
    isAuthenticated: () => boolean;
    hasRole: (role: string) => boolean;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            token: null,
            user: null,
            expiresAt: null,

            setAuth: (token, user, expiresAt) => set({ token, user, expiresAt }),

            logout: () => set({ token: null, user: null, expiresAt: null }),

            isAuthenticated: () => {
                const { token, expiresAt } = get();
                if (!token || !expiresAt) return false;
                return Date.now() / 1000 < expiresAt;
            },

            hasRole: (role: string) => {
                const { user } = get();
                if (!user) return false;
                return user.roles.includes('admin') || user.roles.includes(role);
            }
        }),
        {
            name: 'naseej-auth',
        }
    )
);
