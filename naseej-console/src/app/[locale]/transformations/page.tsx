"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import { Workflow, Plus, Code, Play, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";

interface TransformationsPageProps {
    params: Promise<{ locale: string }>;
}

interface Transformation {
    id: string;
    name: string;
    description: string;
    language: "rhai" | "javascript";
    inputType: string;
    outputType: string;
    usedBy: number;
    lastModified: string;
}

const mockTransformations: Transformation[] = [
    {
        id: "1",
        name: "Celsius to Fahrenheit",
        description: "Converts temperature values from Celsius to Fahrenheit",
        language: "rhai",
        inputType: "JSON",
        outputType: "JSON",
        usedBy: 3,
        lastModified: "2 hours ago",
    },
    {
        id: "2",
        name: "XML to JSON",
        description: "Transforms legacy SOAP XML responses to JSON format",
        language: "rhai",
        inputType: "XML",
        outputType: "JSON",
        usedBy: 5,
        lastModified: "1 day ago",
    },
    {
        id: "3",
        name: "Add Timestamp",
        description: "Adds a timestamp field to all incoming requests",
        language: "rhai",
        inputType: "JSON",
        outputType: "JSON",
        usedBy: 8,
        lastModified: "3 days ago",
    },
    {
        id: "4",
        name: "Filter Sensitive Data",
        description: "Removes PII fields from response payloads",
        language: "rhai",
        inputType: "JSON",
        outputType: "JSON",
        usedBy: 2,
        lastModified: "1 week ago",
    },
];

export default function TransformationsPage({ params }: TransformationsPageProps) {
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
                                <h1 className="text-3xl font-bold tracking-tight">{t("transformations")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Manage data transformation scripts
                                </p>
                            </div>
                            <Button>
                                <Plus className="h-4 w-4 me-2" />
                                New Transformation
                            </Button>
                        </div>

                        {/* Stats */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <StatsCard title="Total Scripts" value={mockTransformations.length.toString()} />
                            <StatsCard
                                title="Active Usage"
                                value={mockTransformations.reduce((acc, t) => acc + t.usedBy, 0).toString()}
                            />
                            <StatsCard title="Language" value="Rhai" />
                        </div>

                        {/* Transformations Grid */}
                        <div className="grid gap-4 md:grid-cols-2">
                            {mockTransformations.map((transform) => (
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
                                                        {transform.lastModified}
                                                    </CardDescription>
                                                </div>
                                            </div>
                                            <Badge variant="outline">{transform.language}</Badge>
                                        </div>
                                    </CardHeader>
                                    <CardContent>
                                        <p className="text-sm text-muted-foreground mb-4">
                                            {transform.description}
                                        </p>
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-4 text-xs text-muted-foreground">
                                                <span>{transform.inputType} → {transform.outputType}</span>
                                                <span>{transform.usedBy} routes</span>
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
