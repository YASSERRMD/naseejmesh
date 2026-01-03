"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use, useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import {
    Workflow,
    Plus,
    Code,
    Play,
    Copy,
    RefreshCw,
    Loader2,
    AlertCircle,
    Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { getTransformations } from "@/lib/api-client";

interface TransformationsPageProps {
    params: Promise<{ locale: string }>;
}

interface Transformation {
    id: string;
    name: string;
    description: string;
    language: string;
    inputType: string;
    outputType: string;
    usedBy: string[];
    script?: string;
    createdAt?: string;
    updatedAt?: string;
}

export default function TransformationsPage({ params }: TransformationsPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");

    const [transformations, setTransformations] = useState<Transformation[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [creating, setCreating] = useState(false);

    // Form state
    const [newTransform, setNewTransform] = useState({
        name: "",
        description: "",
        script: "// Transform your data\npayload.processed_at = now_utc();",
        inputType: "json",
        outputType: "json",
    });

    const fetchTransformations = async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await getTransformations();
            setTransformations(data.map((t: any) => ({
                id: t.id,
                name: t.name,
                description: t.description || "",
                language: t.language || "rhai",
                inputType: t.inputType || "json",
                outputType: t.outputType || "json",
                usedBy: t.usedBy || [],
                script: t.script,
                createdAt: t.createdAt,
                updatedAt: t.updatedAt,
            })));
        } catch (err: any) {
            setError(err.message || "Failed to load transformations");
            // No mock fallback - show empty state
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchTransformations();
    }, []);

    const handleCreateTransformation = async () => {
        if (!newTransform.name || !newTransform.script) return;

        setCreating(true);
        try {
            // TODO: Add createTransformation API call
            // For now, just close dialog
            setDialogOpen(false);
            setNewTransform({
                name: "",
                description: "",
                script: "// Transform your data\npayload.processed_at = now_utc();",
                inputType: "json",
                outputType: "json",
            });
            await fetchTransformations();
        } catch (err: any) {
            setError(err.message || "Failed to create transformation");
        } finally {
            setCreating(false);
        }
    };

    return (
        <div className="min-h-screen bg-background">
            <Sidebar locale={locale} />
            <CommandPalette locale={locale} />
            <div
                className={cn(
                    "transition-all duration-300",
                    isCollapsed ? (isRTL ? "me-16" : "ms-16") : (isRTL ? "me-64" : "ms-64")
                )}
            >
                <Header locale={locale} />
                <main className="p-6">
                    <div className="space-y-6">
                        {/* Page Header */}
                        <div className="flex items-center justify-between">
                            <div>
                                <h1 className="text-3xl font-bold tracking-tight">{t("transformations")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Manage data transformation scripts
                                </p>
                            </div>
                            <div className="flex gap-2">
                                <Button variant="outline" onClick={fetchTransformations} disabled={loading}>
                                    <RefreshCw className={cn("h-4 w-4 me-2", loading && "animate-spin")} />
                                    Refresh
                                </Button>
                                <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                                    <DialogTrigger asChild>
                                        <Button>
                                            <Plus className="h-4 w-4 me-2" />
                                            New Transformation
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent className="sm:max-w-[600px]">
                                        <DialogHeader>
                                            <DialogTitle>Create New Transformation</DialogTitle>
                                            <DialogDescription>
                                                Write a Rhai script to transform request/response data.
                                            </DialogDescription>
                                        </DialogHeader>
                                        <div className="grid gap-4 py-4">
                                            <div className="grid gap-2">
                                                <Label htmlFor="name">Name *</Label>
                                                <Input
                                                    id="name"
                                                    placeholder="Temperature Converter"
                                                    value={newTransform.name}
                                                    onChange={(e) => setNewTransform({ ...newTransform, name: e.target.value })}
                                                />
                                            </div>
                                            <div className="grid gap-2">
                                                <Label htmlFor="description">Description</Label>
                                                <Input
                                                    id="description"
                                                    placeholder="Converts Celsius to Fahrenheit"
                                                    value={newTransform.description}
                                                    onChange={(e) => setNewTransform({ ...newTransform, description: e.target.value })}
                                                />
                                            </div>
                                            <div className="grid grid-cols-2 gap-4">
                                                <div className="grid gap-2">
                                                    <Label htmlFor="inputType">Input Type</Label>
                                                    <Select
                                                        value={newTransform.inputType}
                                                        onValueChange={(value) => setNewTransform({ ...newTransform, inputType: value })}
                                                    >
                                                        <SelectTrigger>
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="json">JSON</SelectItem>
                                                            <SelectItem value="xml">XML</SelectItem>
                                                            <SelectItem value="text">Text</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                                <div className="grid gap-2">
                                                    <Label htmlFor="outputType">Output Type</Label>
                                                    <Select
                                                        value={newTransform.outputType}
                                                        onValueChange={(value) => setNewTransform({ ...newTransform, outputType: value })}
                                                    >
                                                        <SelectTrigger>
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="json">JSON</SelectItem>
                                                            <SelectItem value="xml">XML</SelectItem>
                                                            <SelectItem value="text">Text</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                            </div>
                                            <div className="grid gap-2">
                                                <Label htmlFor="script">Script (Rhai) *</Label>
                                                <Textarea
                                                    id="script"
                                                    className="font-mono text-sm h-40"
                                                    placeholder="// Your Rhai transformation script"
                                                    value={newTransform.script}
                                                    onChange={(e) => setNewTransform({ ...newTransform, script: e.target.value })}
                                                />
                                            </div>
                                        </div>
                                        <DialogFooter>
                                            <Button variant="outline" onClick={() => setDialogOpen(false)}>
                                                Cancel
                                            </Button>
                                            <Button onClick={handleCreateTransformation} disabled={creating || !newTransform.name || !newTransform.script}>
                                                {creating && <Loader2 className="h-4 w-4 me-2 animate-spin" />}
                                                Create Transformation
                                            </Button>
                                        </DialogFooter>
                                    </DialogContent>
                                </Dialog>
                            </div>
                        </div>

                        {/* Error Message */}
                        {error && (
                            <div className="flex items-center gap-2 p-4 rounded-lg bg-destructive/10 text-destructive">
                                <AlertCircle className="h-5 w-5" />
                                <span>{error}</span>
                                <Button variant="ghost" size="sm" onClick={() => setError(null)}>
                                    Dismiss
                                </Button>
                            </div>
                        )}

                        {/* Stats */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <StatsCard title="Total Scripts" value={transformations.length.toString()} />
                            <StatsCard
                                title="Active Usage"
                                value={transformations.reduce((acc, t) => acc + (t.usedBy?.length || 0), 0).toString()}
                            />
                            <StatsCard title="Language" value="Rhai" />
                        </div>

                        {/* Transformations Grid */}
                        {loading ? (
                            <div className="flex items-center justify-center py-12">
                                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                            </div>
                        ) : transformations.length === 0 ? (
                            <Card>
                                <CardContent className="text-center py-12 text-muted-foreground">
                                    <Workflow className="h-12 w-12 mx-auto mb-4 opacity-50" />
                                    <p>No transformations configured yet.</p>
                                    <p className="text-sm">Click &quot;New Transformation&quot; to create your first script.</p>
                                </CardContent>
                            </Card>
                        ) : (
                            <div className="grid gap-4 md:grid-cols-2">
                                {transformations.map((transform) => (
                                    <Card key={transform.id}>
                                        <CardHeader>
                                            <div className="flex items-start justify-between">
                                                <div className="flex items-center gap-2">
                                                    <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                                                        <Workflow className="h-5 w-5 text-primary" />
                                                    </div>
                                                    <div>
                                                        <CardTitle className="text-base">{transform.name}</CardTitle>
                                                        <CardDescription className="text-xs mt-0.5">
                                                            {transform.updatedAt || "Recently updated"}
                                                        </CardDescription>
                                                    </div>
                                                </div>
                                                <Badge variant="outline">{transform.language}</Badge>
                                            </div>
                                        </CardHeader>
                                        <CardContent>
                                            <p className="text-sm text-muted-foreground mb-4">
                                                {transform.description || "No description"}
                                            </p>
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                                                    <span>{transform.inputType.toUpperCase()} → {transform.outputType.toUpperCase()}</span>
                                                    <span>{transform.usedBy?.length || 0} routes</span>
                                                </div>
                                                <div className="flex items-center gap-1">
                                                    <Button variant="ghost" size="icon" title="View Code">
                                                        <Code className="h-4 w-4" />
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="Test">
                                                        <Play className="h-4 w-4" />
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="Duplicate">
                                                        <Copy className="h-4 w-4" />
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="Delete">
                                                        <Trash2 className="h-4 w-4 text-destructive" />
                                                    </Button>
                                                </div>
                                            </div>
                                        </CardContent>
                                    </Card>
                                ))}
                            </div>
                        )}
                    </div>
                </main>
            </div>
        </div>
    );
}

function StatsCard({ title, value }: { title: string; value: string }) {
    return (
        <div className="rounded-lg border bg-card p-4">
            <p className="text-sm text-muted-foreground">{title}</p>
            <p className="text-2xl font-bold mt-1">{value}</p>
        </div>
    );
}
