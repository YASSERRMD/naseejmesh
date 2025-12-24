'use client';

import { useState, useEffect } from 'react';
import { useAuthStore } from '@/stores/auth-store';
import { Loader2, Trash2, User as UserIcon } from 'lucide-react';
import { CreateUserModal } from '@/components/admin/CreateUserModal';

interface User {
    id: string;
    username: string;
    roles: string[];
    active: boolean;
    created_at: string;
}

export default function UsersPage() {
    const { token } = useAuthStore();
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState('');

    const fetchUsers = async () => {
        try {
            const res = await fetch('http://localhost:3001/api/admin/users', {
                headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error('Failed to fetch users');
            const data = await res.json();
            setUsers(data);
        } catch (err) {
            setError('Failed to load users');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchUsers();
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
                <h1 className="text-2xl font-bold text-white">Users</h1>
                <CreateUserModal onSuccess={fetchUsers} />
            </div>
            {error && (
                <div className="rounded-md bg-red-900/20 p-4 text-red-400">
                    {error}
                </div>
            )}

            <div className="rounded-xl border border-zinc-800 bg-zinc-900/50 backdrop-blur-xl">
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm text-zinc-400">
                        <thead className="bg-zinc-900/50 text-xs uppercase text-zinc-500">
                            <tr>
                                <th className="px-6 py-4 font-medium">Username</th>
                                <th className="px-6 py-4 font-medium">Roles</th>
                                <th className="px-6 py-4 font-medium">Status</th>
                                <th className="px-6 py-4 font-medium">Created</th>
                                <th className="px-6 py-4 font-medium">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800">
                            {users.map((user) => (
                                <tr key={user.id} className="hover:bg-zinc-900/50">
                                    <td className="px-6 py-4">
                                        <div className="flex items-center gap-3">
                                            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-zinc-800">
                                                <UserIcon className="h-4 w-4 text-zinc-500" />
                                            </div>
                                            <span className="font-medium text-white">{user.username}</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex gap-2">
                                            {user.roles.map((role) => (
                                                <span
                                                    key={role}
                                                    className="rounded-full bg-indigo-500/10 px-2 py-0.5 text-xs font-medium text-indigo-400"
                                                >
                                                    {role}
                                                </span>
                                            ))}
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <span
                                            className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${user.active
                                                ? 'bg-green-500/10 text-green-400'
                                                : 'bg-red-500/10 text-red-400'
                                                }`}
                                        >
                                            {user.active ? 'Active' : 'Inactive'}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4">
                                        {new Date(user.created_at).toLocaleDateString()}
                                    </td>
                                    <td className="px-6 py-4">
                                        <button className="text-zinc-500 hover:text-red-400">
                                            <Trash2 className="h-4 w-4" />
                                        </button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
