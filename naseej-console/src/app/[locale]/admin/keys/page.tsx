'use client';

import { useState, useEffect } from 'react';
import { useAuthStore } from '@/stores/auth-store';
import { Loader2, Key, Trash2 } from 'lucide-react';
import { CreateKeyModal } from '@/components/admin/CreateKeyModal';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';

interface ApiKey {
    id: string;
    name: string;
    prefix: string;
    scopes: string[];
    last_used_at: string | null;
    created_at: string;
}

export default function KeysPage() {
    const { token } = useAuthStore();
    const [keys, setKeys] = useState<ApiKey[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState('');

    const fetchKeys = async () => {
        try {
            const res = await fetch('http://localhost:3001/api/admin/keys', {
                headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error('Failed to fetch keys');
            const data = await res.json();
            setKeys(data);
        } catch (err) {
            setError('Failed to load keys');
        } finally {
            setLoading(false);
        }
    };

    const handleDelete = async (id: string) => {
        if (!confirm('Are you sure you want to delete this key?')) return;

        try {
            const res = await fetch(`http://localhost:3001/api/admin/keys/${id}`, {
                method: 'DELETE',
                headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error('Failed to delete key');

            toast.success('Key deleted');
            fetchKeys();
        } catch (err) {
            toast.error('Failed to delete key');
        }
    };

    useEffect(() => {
        fetchKeys();
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
                <h1 className="text-2xl font-bold text-white">API Keys</h1>
                <CreateKeyModal onSuccess={fetchKeys} />
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
                                <th className="px-6 py-4 font-medium">Name</th>
                                <th className="px-6 py-4 font-medium">Prefix</th>
                                <th className="px-6 py-4 font-medium">Scopes</th>
                                <th className="px-6 py-4 font-medium">Created</th>
                                <th className="px-6 py-4 font-medium">Last Used</th>
                                <th className="px-6 py-4 font-medium">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800">
                            {keys.map((key) => (
                                <tr key={key.id} className="hover:bg-zinc-900/50">
                                    <td className="px-6 py-4">
                                        <div className="flex items-center gap-3">
                                            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-indigo-500/10">
                                                <Key className="h-4 w-4 text-indigo-400" />
                                            </div>
                                            <span className="font-medium text-white">{key.name}</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 font-mono text-zinc-300">
                                        {key.prefix}...
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex gap-2">
                                            {key.scopes.map(scope => (
                                                <span key={scope} className="rounded px-1.5 py-0.5 bg-zinc-800 text-xs text-zinc-400">{scope}</span>
                                            ))}
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        {new Date(key.created_at).toLocaleDateString()}
                                    </td>
                                    <td className="px-6 py-4">
                                        {key.last_used_at ? new Date(key.last_used_at).toLocaleDateString() : 'Never'}
                                    </td>
                                    <td className="px-6 py-4">
                                        <Button variant="ghost" size="icon" onClick={() => handleDelete(key.id)} className="text-zinc-500 hover:text-red-400 hover:bg-red-950/30">
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
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
