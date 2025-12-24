'use client';

import { useState, useEffect } from 'react';
import { useAuthStore } from '@/stores/auth-store';
import { Loader2, Shield } from 'lucide-react';
import { CreateRoleModal } from '@/components/admin/CreateRoleModal';

interface Role {
    id: string;
    name: string;
    permissions: string[];
    created_at: string;
}

export default function RolesPage() {
    const { token } = useAuthStore();
    const [roles, setRoles] = useState<Role[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState('');

    const fetchRoles = async () => {
        try {
            const res = await fetch('http://localhost:3001/api/admin/roles', {
                headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error('Failed to fetch roles');
            const data = await res.json();
            setRoles(data);
        } catch (err) {
            setError('Failed to load roles');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchRoles();
    }, [token]);

    if (loading) {
        return (
            <div className="flex h-96 items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <h1 className="text-2xl font-bold text-white">Roles</h1>
                <CreateRoleModal onSuccess={fetchRoles} />
            </div>

            {error && (
                <div className="rounded-md bg-red-900/20 p-4 text-red-400">
                    {error}
                </div>
            )}

            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {roles.map((role) => (
                    <div
                        key={role.id}
                        className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-6 backdrop-blur-xl"
                    >
                        <div className="flex items-center gap-3">
                            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-500/10">
                                <Shield className="h-5 w-5 text-indigo-400" />
                            </div>
                            <div>
                                <h3 className="font-semibold text-white">{role.name}</h3>
                                <p className="text-xs text-zinc-500">
                                    {new Date(role.created_at).toLocaleDateString()}
                                </p>
                            </div>
                        </div>

                        <div className="mt-4">
                            <h4 className="mb-2 text-xs font-medium uppercase text-zinc-500">Permissions</h4>
                            <div className="flex flex-wrap gap-2">
                                {role.permissions.map((perm) => (
                                    <span
                                        key={perm}
                                        className="rounded-md bg-zinc-800 px-2 py-1 text-xs text-zinc-300"
                                    >
                                        {perm}
                                    </span>
                                ))}
                            </div>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}
