"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use, useEffect, useState, useRef } from "react";
import { useTranslations } from "next-intl";
import {
    FileJson,
    Plus,
    Copy,
    Check,
    ExternalLink,
    Trash2,
    RefreshCw,
    Loader2,
    AlertCircle,
    Upload,
    Workflow,
} from "lucide-react";
import { Button } from "@/components/ui/button";
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
// ... imports above
import { getSchemas, createSchema } from "@/lib/api-client";


interface SchemasPageProps {
    params: Promise<{ locale: string }>;
}

interface Schema {
    id: string;
    name: string;
    type: string;
    version: string;
    endpoints: number;
    content?: string;
    status: string;
    errors?: string[];
    createdAt?: string;
    updatedAt?: string;
}

const typeColors: Record<string, string> = {
    openapi: "bg-green-500",
    jsonschema: "bg-blue-500",
    graphql: "bg-purple-500",
    soap: "bg-orange-500",
    grpc: "bg-cyan-500",
    mcp: "bg-pink-500",
    mqtt: "bg-yellow-500",
};

export default function SchemasPage({ params }: SchemasPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");
    const fileInputRef = useRef<HTMLInputElement>(null);

    const [schemas, setSchemas] = useState<Schema[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [creating, setCreating] = useState(false);

    // Form state
    const [newSchema, setNewSchema] = useState({
        name: "",
        type: "openapi",
        version: "3.0.0",
        content: "",
    });

    const fetchSchemas = async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await getSchemas();
            setSchemas(data.map((s: any) => ({
                id: s.id,
                name: s.name,
                type: s.type || "openapi",
                version: s.version || "unknown",
                endpoints: s.endpoints || 0,
                content: s.content,
                status: s.status || "valid",
                errors: s.errors,
                createdAt: s.createdAt,
                updatedAt: s.updatedAt,
            })));
        } catch (err: any) {
            setError(err.message || "Failed to load schemas");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchSchemas();
    }, []);

    const handleImportSchema = async () => {
        if (!newSchema.name || !newSchema.content) return;

        setCreating(true);
        try {
            await createSchema({
                name: newSchema.name,
                type: newSchema.type as any,
                version: newSchema.version,
                content: newSchema.content,
                status: "valid",
                endpoints: 0,
            });
            setDialogOpen(false);
            setNewSchema({
                name: "",
                type: "openapi",
                version: "3.0.0",
                content: "",
            });
            await fetchSchemas();
        } catch (err: any) {
            setError(err.message || "Failed to import schema");
        } finally {
            setCreating(false);
        }
    };

    const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        if (!file) return;

        const reader = new FileReader();
        reader.onload = (e) => {
            const content = e.target?.result as string;
            setNewSchema({
                ...newSchema,
                name: newSchema.name || file.name.replace(/\.[^/.]+$/, ""),
                content,
            });
        };
        reader.readAsText(file);
    };

    const handleDelete = async (id: string, name: string) => {
        if (!confirm(`Are you sure you want to delete schema "${name}"?`)) return;

        try {
            // @ts-ignore - deleteSchema imported implicitly with new api-client
            await import("@/lib/api-client").then(mod => mod.deleteSchema(id));
            setSchemas(schemas.filter(s => s.id !== id));
        } catch (err: any) {
            setError(err.message || "Failed to delete schema");
        }
    };

    const handleDuplicate = async (schema: Schema) => {
        try {
            await createSchema({
                name: `Copy of ${schema.name}`,
                type: schema.type as any,
                version: schema.version,
                content: schema.content,
                status: "valid",
                endpoints: schema.endpoints,
            });
            await fetchSchemas();
        } catch (err: any) {
            setError(err.message || "Failed to duplicate schema");
        }
    };

    const handleView = (schema: Schema) => {
        setNewSchema({
            name: schema.name,
            type: schema.type,
            version: schema.version,
            content: schema.content || "",
        });
        setDialogOpen(true);
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
                                <h1 className="text-3xl font-bold tracking-tight">{t("schemas")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Manage API schemas and documentation
                                </p>
                            </div>
                            <div className="flex gap-2">
                                <Button variant="outline" onClick={fetchSchemas} disabled={loading}>
                                    <RefreshCw className={cn("h-4 w-4 me-2", loading && "animate-spin")} />
                                    Refresh
                                </Button>
                                <Dialog open={dialogOpen} onOpenChange={(open) => {
                                    setDialogOpen(open);
                                    if (!open) {
                                        setNewSchema({ name: "", type: "openapi", version: "3.0.0", content: "" });
                                    }
                                }}>
                                    <DialogTrigger asChild>
                                        <Button onClick={() => setNewSchema({ name: "", type: "openapi", version: "3.0.0", content: "" })}>
                                            <Plus className="h-4 w-4 me-2" />
                                            Import Schema
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent className="sm:max-w-[600px]">
                                        <DialogHeader>
                                            <DialogTitle>{newSchema.content ? "Edit/View Schema" : "Import API Schema"}</DialogTitle>
                                            <DialogDescription>
                                                Details for the API Schema.
                                            </DialogDescription>
                                        </DialogHeader>
                                        <div className="grid gap-4 py-4">
                                            <div className="grid gap-2">
                                                <Label htmlFor="name">Name *</Label>
                                                <Input
                                                    id="name"
                                                    placeholder="User Service API"
                                                    value={newSchema.name}
                                                    onChange={(e) => setNewSchema({ ...newSchema, name: e.target.value })}
                                                />
                                            </div>
                                            <div className="grid grid-cols-2 gap-4">
                                                <div className="grid gap-2">
                                                    <Label htmlFor="type">Schema Type</Label>
                                                    <Select
                                                        value={newSchema.type}
                                                        onValueChange={(value) => setNewSchema({ ...newSchema, type: value })}
                                                    >
                                                        <SelectTrigger>
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="openapi">OpenAPI (REST)</SelectItem>
                                                            <SelectItem value="graphql">GraphQL</SelectItem>
                                                            <SelectItem value="soap">SOAP (WSDL)</SelectItem>
                                                            <SelectItem value="grpc">gRPC (Protobuf)</SelectItem>
                                                            <SelectItem value="mcp">MCP (Model Context Protocol)</SelectItem>
                                                            <SelectItem value="mqtt">MQTT</SelectItem>
                                                            <SelectItem value="jsonschema">JSON Schema</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                                <div className="grid gap-2">
                                                    <Label htmlFor="version">Version</Label>
                                                    <Input
                                                        id="version"
                                                        placeholder="3.0.0"
                                                        value={newSchema.version}
                                                        onChange={(e) => setNewSchema({ ...newSchema, version: e.target.value })}
                                                    />
                                                </div>
                                            </div>
                                            <div className="grid gap-2">
                                                <div className="flex items-center justify-between">
                                                    <Label htmlFor="content">Schema Content *</Label>
                                                    <Button
                                                        variant="outline"
                                                        size="sm"
                                                        onClick={() => fileInputRef.current?.click()}
                                                    >
                                                        <Upload className="h-4 w-4 me-2" />
                                                        Upload File
                                                    </Button>
                                                    <input
                                                        ref={fileInputRef}
                                                        type="file"
                                                        accept=".json,.yaml,.yml,.graphql,.wsdl,.proto,.mcp"
                                                        className="hidden"
                                                        onChange={handleFileUpload}
                                                    />
                                                </div>
                                                <Textarea
                                                    id="content"
                                                    className="font-mono text-sm h-48"
                                                    placeholder='{"openapi": "3.0.0", "info": {...}}'
                                                    value={newSchema.content}
                                                    onChange={(e) => setNewSchema({ ...newSchema, content: e.target.value })}
                                                />
                                            </div>
                                        </div>
                                        <DialogFooter>
                                            <Button variant="outline" onClick={() => setDialogOpen(false)}>
                                                Cancel
                                            </Button>
                                            <Button onClick={handleImportSchema} disabled={creating || !newSchema.name || !newSchema.content}>
                                                {creating && <Loader2 className="h-4 w-4 me-2 animate-spin" />}
                                                Save Schema
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
                        <div className="grid gap-4 md:grid-cols-4">
                            <StatsCard title="Total Schemas" value={schemas.length.toString()} />
                            <StatsCard
                                title="OpenAPI"
                                value={schemas.filter((s) => s.type === "openapi").length.toString()}
                            />
                            <StatsCard
                                title="JSON Schema"
                                value={schemas.filter((s) => s.type === "jsonschema").length.toString()}
                            />
                            <StatsCard
                                title="GraphQL"
                                value={schemas.filter((s) => s.type === "graphql").length.toString()}
                            />
                        </div>

                        {/* Schemas Grid */}
                        {loading ? (
                            <div className="flex items-center justify-center py-12">
                                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                            </div>
                        ) : schemas.length === 0 ? (
                            <Card>
                                <CardContent className="text-center py-12 text-muted-foreground">
                                    <FileJson className="h-12 w-12 mx-auto mb-4 opacity-50" />
                                    <p>No API schemas imported yet.</p>
                                    <p className="text-sm">Click &quot;Import Schema&quot; to add your first schema.</p>
                                </CardContent>
                            </Card>
                        ) : (
                            <div className="grid gap-4 md:grid-cols-2">
                                {schemas.map((schema) => (
                                    <Card key={schema.id}>
                                        <CardHeader>
                                            <div className="flex items-start justify-between">
                                                <div className="flex items-center gap-3">
                                                    <div className="h-10 w-10 rounded-lg bg-muted flex items-center justify-center">
                                                        <FileJson className="h-5 w-5 text-muted-foreground" />
                                                    </div>
                                                    <div>
                                                        <CardTitle className="text-base">{schema.name}</CardTitle>
                                                        <CardDescription className="text-xs mt-0.5">
                                                            Updated {schema.updatedAt || "recently"}
                                                        </CardDescription>
                                                    </div>
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <span
                                                        className={cn(
                                                            "px-2 py-0.5 rounded text-xs font-medium text-white",
                                                            typeColors[schema.type] || "bg-gray-500"
                                                        )}
                                                    >
                                                        {schema.type}
                                                    </span>
                                                    {schema.status === "valid" && (
                                                        <Check className="h-4 w-4 text-green-500" />
                                                    )}
                                                </div>
                                            </div>
                                        </CardHeader>
                                        <CardContent>
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-4 text-sm text-muted-foreground">
                                                    <span>v{schema.version}</span>
                                                    <span>{schema.endpoints} endpoints</span>
                                                </div>
                                                <div className="flex items-center gap-1">
                                                    <Button
                                                        variant="outline"
                                                        size="sm"
                                                        className="h-8 text-xs me-2"
                                                        title="Generate Routes from Schema"
                                                        onClick={() => {
                                                            window.location.href = `/${locale}/routes/generate?schema=${schema.id}`;
                                                        }}
                                                    >
                                                        <Workflow className="h-3 w-3 me-1" />
                                                        Gen Routes
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="View/Edit" onClick={() => handleView(schema)}>
                                                        <ExternalLink className="h-4 w-4" />
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="Duplicate" onClick={() => handleDuplicate(schema)}>
                                                        <Copy className="h-4 w-4" />
                                                    </Button>
                                                    <Button variant="ghost" size="icon" title="Delete" onClick={() => handleDelete(schema.id, schema.name)}>
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
