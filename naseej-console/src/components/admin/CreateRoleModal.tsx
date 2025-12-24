'use client';

import { useState } from 'react';
import { useAuthStore } from '@/stores/auth-store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Plus, Loader2 } from 'lucide-react';
import { toast } from 'sonner';

interface CreateRoleModalProps {
    onSuccess: () => void;
}

export function CreateRoleModal({ onSuccess }: CreateRoleModalProps) {
    const { token } = useAuthStore();
    const [open, setOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [name, setName] = useState('');
    const [permissions, setPermissions] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);

        try {
            const permsArray = permissions.split(',').map((p) => p.trim()).filter(Boolean);

            const res = await fetch('http://localhost:3001/api/admin/roles', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${token}`,
                },
                body: JSON.stringify({
                    name,
                    permissions: permsArray,
                }),
            });

            if (!res.ok) throw new Error('Failed to create role');

            toast.success('Role created successfully');
            setOpen(false);
            setName('');
            setPermissions('');
            onSuccess();
        } catch (err) {
            toast.error('Failed to create role');
        } finally {
            setLoading(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button>
                    <Plus className="mr-2 h-4 w-4" />
                    Create Role
                </Button>
            </DialogTrigger>
            <DialogContent className="bg-zinc-950 border-zinc-800 text-white sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Create New Role</DialogTitle>
                    <DialogDescription className="text-zinc-400">
                        Define a role with specific permissions.
                    </DialogDescription>
                </DialogHeader>
                <form onSubmit={handleSubmit} className="space-y-4 py-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none">Role Name</label>
                        <Input
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder="editor"
                            className="bg-zinc-900 border-zinc-700 text-white"
                            required
                        />
                    </div>
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none">Permissions (comma separated)</label>
                        <Input
                            value={permissions}
                            onChange={(e) => setPermissions(e.target.value)}
                            placeholder="read, write, delete"
                            className="bg-zinc-900 border-zinc-700 text-white"
                        />
                    </div>
                    <DialogFooter>
                        <Button type="submit" disabled={loading} className="w-full">
                            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            Create Role
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
