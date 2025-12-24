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
import { Plus, Loader2, Copy, Check } from 'lucide-react';
import { toast } from 'sonner';

interface CreateKeyModalProps {
    onSuccess: () => void;
}

export function CreateKeyModal({ onSuccess }: CreateKeyModalProps) {
    const { token, user } = useAuthStore();
    const [open, setOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [name, setName] = useState('');
    const [scopes, setScopes] = useState('');
    const [createdKey, setCreatedKey] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);

        try {
            const scopesArray = scopes.split(',').map((s) => s.trim()).filter(Boolean);

            const res = await fetch('http://localhost:3001/api/admin/keys', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${token}`,
                },
                body: JSON.stringify({
                    name,
                    owner_id: user?.id || 'unknown',
                    scopes: scopesArray,
                }),
            });

            if (!res.ok) throw new Error('Failed to create key');

            const data = await res.json();
            setCreatedKey(data.raw_key); // Show the raw key
            toast.success('API Key created successfully');
            onSuccess();
        } catch (err) {
            toast.error('Failed to create key');
        } finally {
            setLoading(false);
        }
    };

    const copyToClipboard = () => {
        if (createdKey) {
            navigator.clipboard.writeText(createdKey);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
            toast.success("Copied to clipboard");
        }
    };

    const handleClose = () => {
        setOpen(false);
        setCreatedKey(null);
        setName('');
        setScopes('');
    };

    return (
        <Dialog open={open} onOpenChange={(val) => !val && handleClose()}>
            <DialogTrigger asChild>
                <Button onClick={() => setOpen(true)}>
                    <Plus className="mr-2 h-4 w-4" />
                    Create API Key
                </Button>
            </DialogTrigger>
            <DialogContent className="bg-zinc-950 border-zinc-800 text-white sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Create New API Key</DialogTitle>
                    <DialogDescription className="text-zinc-400">
                        Generate a new API key for external access.
                    </DialogDescription>
                </DialogHeader>

                {!createdKey ? (
                    <form onSubmit={handleSubmit} className="space-y-4 py-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium leading-none">Key Name</label>
                            <Input
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="My Service Key"
                                className="bg-zinc-900 border-zinc-700 text-white"
                                required
                            />
                        </div>
                        <div className="space-y-2">
                            <label className="text-sm font-medium leading-none">Scopes (comma separated)</label>
                            <Input
                                value={scopes}
                                onChange={(e) => setScopes(e.target.value)}
                                placeholder="read, write"
                                className="bg-zinc-900 border-zinc-700 text-white"
                            />
                        </div>
                        <DialogFooter>
                            <Button type="submit" disabled={loading} className="w-full">
                                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                Generate Key
                            </Button>
                        </DialogFooter>
                    </form>
                ) : (
                    <div className="space-y-4 py-4">
                        <div className="rounded-md bg-amber-900/20 p-4 border border-amber-900/50">
                            <p className="text-sm text-amber-200">
                                This key will only be shown once. Please copy it now.
                            </p>
                        </div>
                        <div className="flex items-center gap-2">
                            <code className="flex-1 rounded bg-black p-3 font-mono text-sm text-green-400 break-all border border-zinc-800">
                                {createdKey}
                            </code>
                            <Button size="icon" variant="outline" onClick={copyToClipboard} className="shrink-0 border-zinc-700 hover:bg-zinc-800">
                                {copied ? <Check className="h-4 w-4 text-green-500" /> : <Copy className="h-4 w-4" />}
                            </Button>
                        </div>
                        <DialogFooter>
                            <Button onClick={handleClose} className="w-full">
                                Done
                            </Button>
                        </DialogFooter>
                    </div>
                )}
            </DialogContent>
        </Dialog>
    );
}
