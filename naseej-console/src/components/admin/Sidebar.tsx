'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import { Users, Shield, Key, LayoutDashboard, LogOut } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useAuthStore } from '@/stores/auth-store';
import { useRouter } from 'next/navigation';

export function AdminSidebar() {
    const pathname = usePathname();
    const t = useTranslations('Admin'); // Assuming Admin namespace
    const { logout } = useAuthStore();
    const router = useRouter();

    const handleLogout = () => {
        logout();
        router.push('/login');
    };

    const links = [
        { href: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
        { href: '/admin/users', label: 'Users', icon: Users },
        { href: '/admin/roles', label: 'Roles', icon: Shield },
        { href: '/admin/keys', label: 'API Keys', icon: Key },
    ];

    return (
        <div className="flex h-screen w-64 flex-col border-r border-zinc-800 bg-zinc-950">
            <div className="flex h-16 items-center border-b border-zinc-800 px-6">
                <span className="text-lg font-bold text-white">Naseej Admin</span>
            </div>

            <nav className="flex-1 space-y-1 p-4">
                {links.map((link) => {
                    const isActive = pathname.includes(link.href);
                    return (
                        <Link
                            key={link.href}
                            href={link.href}
                            className={cn(
                                'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                                isActive
                                    ? 'bg-indigo-500/10 text-indigo-400'
                                    : 'text-zinc-400 hover:bg-zinc-900 hover:text-zinc-100'
                            )}
                        >
                            <link.icon className="h-4 w-4" />
                            {link.label}
                        </Link>
                    );
                })}
            </nav>

            <div className="border-t border-zinc-800 p-4">
                <button
                    onClick={handleLogout}
                    className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-red-400 transition-colors hover:bg-red-950/30"
                >
                    <LogOut className="h-4 w-4" />
                    Sign Out
                </button>
            </div>
        </div>
    );
}
