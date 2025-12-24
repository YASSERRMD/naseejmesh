'use client';

import { useState } from 'react';
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
import { Sparkles, Loader2 } from 'lucide-react';
import { toast } from 'sonner';
import { useMeshStore, ServiceNode, ServiceType } from '@/stores/mesh-store';
import { generateNodeId } from '@/lib/flow-layout';

interface SmartDesignDialogProps {
    onSuccess?: () => void;
}

export function SmartDesignDialog({ onSuccess }: SmartDesignDialogProps) {
    const [open, setOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [prompt, setPrompt] = useState('');
    const { addNode, layout, reset } = useMeshStore();

    const handleGenerate = async () => {
        if (!prompt.trim()) {
            toast.error('Please enter a description');
            return;
        }

        setLoading(true);

        try {
            const res = await fetch('http://localhost:3001/api/design/generate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt }),
            });

            if (!res.ok) throw new Error('Generation failed');

            const data = await res.json();

            // Reset canvas and add generated nodes
            reset();

            // Add nodes from AI response
            if (data.nodes && Array.isArray(data.nodes)) {
                data.nodes.forEach((nodeData: { type: ServiceType; label: string; config?: Record<string, unknown> }, index: number) => {
                    const node: ServiceNode = {
                        id: generateNodeId(nodeData.type),
                        type: 'service',
                        position: { x: index * 300, y: 100 + (index % 2) * 100 },
                        data: {
                            label: nodeData.label,
                            serviceType: nodeData.type,
                            status: 'healthy',
                            description: nodeData.config?.description as string || 'AI Generated',
                            ...nodeData.config,
                        },
                    };
                    addNode(node);
                });
            }

            // Apply layout
            setTimeout(() => layout(), 100);

            toast.success('Flow generated successfully!');
            setOpen(false);
            setPrompt('');
            onSuccess?.();
        } catch (err) {
            toast.error('Failed to generate flow. Make sure the backend is running.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="default" size="sm" className="bg-gradient-to-r from-pink-500 to-violet-500 hover:from-pink-600 hover:to-violet-600">
                    <Sparkles className="h-4 w-4 me-2" />
                    Smart Design
                </Button>
            </DialogTrigger>
            <DialogContent className="bg-zinc-950 border-zinc-800 text-white sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Sparkles className="h-5 w-5 text-pink-400" />
                        Smart Design with AI
                    </DialogTitle>
                    <DialogDescription className="text-zinc-400">
                        Describe your desired flow in natural language and AI will generate it for you.
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none">Flow Description</label>
                        <Input
                            value={prompt}
                            onChange={(e) => setPrompt(e.target.value)}
                            placeholder="e.g., Create a flow that receives MQTT sensor data, filters by temperature > 25, and saves to PostgreSQL"
                            className="bg-zinc-900 border-zinc-700 text-white"
                        />
                    </div>

                    <div className="rounded-md bg-zinc-900 p-3 text-xs text-zinc-400">
                        <p className="font-medium text-zinc-300 mb-1">Tip:</p>
                        <p>Be specific about data sources, transformations, and destinations. Mention protocols (MQTT, HTTP), databases, and any AI processing needed.</p>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)} className="border-zinc-700">
                        Cancel
                    </Button>
                    <Button
                        onClick={handleGenerate}
                        disabled={loading}
                        className="bg-gradient-to-r from-pink-500 to-violet-500"
                    >
                        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                        Generate Flow
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
