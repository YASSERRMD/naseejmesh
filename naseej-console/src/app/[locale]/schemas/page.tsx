"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import { FileJson, Plus, Copy, Check, ExternalLink, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";

interface SchemasPageProps {
    params: Promise<{ locale: string }>;
}

interface Schema {
    id: string;
    name: string;
    type: "openapi" | "jsonschema" | "graphql";
    version: string;
    endpoints: number;
    lastUpdated: string;
    status: "valid" | "warning" | "error";
}

const mockSchemas: Schema[] = [
    {
        id: "1",
        name: "User Service API",
        type: "openapi",
        version: "3.0.1",
        endpoints: 12,
        lastUpdated: "2 hours ago",
        status: "valid",
    },
    {
        id: "2",
        name: "Orders Schema",
        type: "jsonschema",
        version: "draft-07",
        endpoints: 5,
        lastUpdated: "1 day ago",
        status: "valid",
    },
    {
        id: "3",
        name: "Product Catalog",
        type: "openapi",
        version: "3.0.0",
        endpoints: 8,
        lastUpdated: "3 days ago",
        status: "warning",
    },
    {
        id: "4",
        name: "GraphQL Gateway",
        type: "graphql",
        version: "SDL",
        endpoints: 24,
        lastUpdated: "1 week ago",
        status: "valid",
    },
];

const typeColors: Record<string, string> = {
    openapi: "bg-green-500",
    jsonschema: "bg-blue-500",
    graphql: "bg-purple-500",
};

export default function SchemasPage({ params }: SchemasPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");

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
                            <Button>
                                <Plus className="h-4 w-4 me-2" />
                                Import Schema
                            </Button>
                        </div>

                        {/* Stats */}
                        <div className="grid gap-4 md:grid-cols-4">
                            <StatsCard title="Total Schemas" value={mockSchemas.length.toString()} />
                            <StatsCard
                                title="OpenAPI"
                                value={mockSchemas.filter((s) => s.type === "openapi").length.toString()}
                            />
                            <StatsCard
                                title="JSON Schema"
                                value={mockSchemas.filter((s) => s.type === "jsonschema").length.toString()}
                            />
                            <StatsCard
                                title="GraphQL"
                                value={mockSchemas.filter((s) => s.type === "graphql").length.toString()}
                            />
                        </div>

                        {/* Schemas Grid */}
                        <div className="grid gap-4 md:grid-cols-2">
                            {mockSchemas.map((schema) => (
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
                                                        Updated {schema.lastUpdated}
                                                    </CardDescription>
                                                </div>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <span
                                                    className={cn(
                                                        "px-2 py-0.5 rounded text-xs font-medium text-white",
                                                        typeColors[schema.type]
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
                                                <Button variant="ghost" size="icon" title="View">
                                                    <ExternalLink className="h-4 w-4" />
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
