'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Sparkles, Loader2, Zap, AlertCircle } from 'lucide-react';
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
    const [error, setError] = useState<string | null>(null);
    const { addNode, layout, reset } = useMeshStore();

    const handleGenerate = async () => {
        if (!prompt.trim()) {
            toast.error('Please enter a description');
            return;
        }

        setLoading(true);
        setError(null);

        try {
            const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';
            const res = await fetch(`${apiUrl}/api/design/generate`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt }),
            });

            if (!res.ok) {
                const errorText = await res.text();
                throw new Error(errorText || 'Generation failed');
            }

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
        } catch (err: any) {
            const errorMessage = err?.message || 'Failed to generate flow. Check if the backend is running.';
            setError(errorMessage);
            toast.error(errorMessage);
        } finally {
            setLoading(false);
        }
    };

    const examplePrompts = [
        "Create a flow that receives MQTT sensor data, transforms temperature from Celsius to Fahrenheit, and stores in a database",
        "Build an API gateway that validates requests, applies rate limiting, and forwards to a backend service",
        "Design a pipeline that ingests webhooks, filters based on event type, and routes to different services",
    ];

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="default" size="sm" className="bg-gradient-to-r from-pink-500 to-violet-500 hover:from-pink-600 hover:to-violet-600 text-white">
                    <Sparkles className="h-4 w-4 me-2" />
                    Smart Design
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[600px]">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <div className="h-8 w-8 rounded-lg bg-gradient-to-r from-pink-500 to-violet-500 flex items-center justify-center">
                            <Sparkles className="h-4 w-4 text-white" />
                        </div>
                        Smart Design with AI
                    </DialogTitle>
                    <DialogDescription>
                        Describe your desired flow in natural language and AI will generate it for you.
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    {/* Error Message */}
                    {error && (
                        <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
                            <AlertCircle className="h-4 w-4 shrink-0" />
                            <span>{error}</span>
                        </div>
                    )}

                    <div className="space-y-2">
                        <label className="text-sm font-medium">Flow Description</label>
                        <Textarea
                            value={prompt}
                            onChange={(e) => setPrompt(e.target.value)}
                            placeholder="e.g., Create a flow that receives MQTT sensor data, filters by temperature > 25, and saves to PostgreSQL"
                            className="min-h-[100px] resize-none"
                        />
                    </div>

                    {/* Example Prompts */}
                    <div className="space-y-2">
                        <label className="text-xs font-medium text-muted-foreground">Quick Examples</label>
                        <div className="flex flex-wrap gap-2">
                            {examplePrompts.map((example, i) => (
                                <button
                                    key={i}
                                    onClick={() => setPrompt(example)}
                                    className="text-xs px-2 py-1 rounded-full border hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
                                >
                                    <Zap className="h-3 w-3 inline me-1" />
                                    Example {i + 1}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div className="rounded-md bg-muted/50 p-3 text-xs text-muted-foreground">
                        <p className="font-medium text-foreground mb-1">Tips for better results:</p>
                        <ul className="list-disc list-inside space-y-1">
                            <li>Be specific about data sources (MQTT, HTTP webhooks, Kafka)</li>
                            <li>Mention transformations needed (convert, filter, aggregate)</li>
                            <li>Specify destinations (database, API, message queue)</li>
                        </ul>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>
                        Cancel
                    </Button>
                    <Button
                        onClick={handleGenerate}
                        disabled={loading || !prompt.trim()}
                        className="bg-gradient-to-r from-pink-500 to-violet-500 hover:from-pink-600 hover:to-violet-600 text-white"
                    >
                        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                        {loading ? 'Generating...' : 'Generate Flow'}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
