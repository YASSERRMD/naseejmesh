"use client";

import * as React from "react";
import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use, useState } from "react";
import { useRouter } from "next/navigation";
import {
    ArrowLeft,
    Plus,
    Trash2,
    Save,
    TestTube,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import type { HttpMethod } from "@/lib/api-types";

interface NewRoutePageProps {
    params: Promise<{ locale: string }>;
}

const HTTP_METHODS: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH"];

const methodColors: Record<string, string> = {
    GET: "bg-green-500 hover:bg-green-600",
    POST: "bg-blue-500 hover:bg-blue-600",
    PUT: "bg-yellow-500 hover:bg-yellow-600",
    DELETE: "bg-red-500 hover:bg-red-600",
    PATCH: "bg-purple-500 hover:bg-purple-600",
};

export default function NewRoutePage({ params }: NewRoutePageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const router = useRouter();

    const [name, setName] = useState("");
    const [path, setPath] = useState("/api/");
    const [methods, setMethods] = useState<HttpMethod[]>(["GET"]);
    const [upstreamUrl, setUpstreamUrl] = useState("http://");
    const [timeout, setTimeout] = useState("30000");
    const [retries, setRetries] = useState("3");
    const [rateLimit, setRateLimit] = useState("100");
    const [authType, setAuthType] = useState<"none" | "jwt" | "api_key">("none");

    const toggleMethod = (method: HttpMethod) => {
        if (methods.includes(method)) {
            setMethods(methods.filter((m) => m !== method));
        } else {
            setMethods([...methods, method]);
        }
    };

    const handleSave = () => {
        // TODO: Call API to save route
        console.log({
            name,
            path,
            methods,
            upstream: { url: upstreamUrl, timeout: parseInt(timeout), retries: parseInt(retries) },
            rateLimit: { requestsPerMinute: parseInt(rateLimit) },
            auth: { type: authType },
        });
        router.push(`/${locale}/routes`);
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
                    <div className="max-w-4xl mx-auto space-y-6">
                        {/* Header */}
                        <div className="flex items-center gap-4">
                            <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => router.push(`/${locale}/routes`)}
                            >
                                <ArrowLeft className="h-5 w-5" />
                            </Button>
                            <div className="flex-1">
                                <h1 className="text-2xl font-bold tracking-tight">Create New Route</h1>
                                <p className="text-muted-foreground">
                                    Configure a new API route for your gateway
                                </p>
                            </div>
                            <Button variant="outline">
                                <TestTube className="h-4 w-4 me-2" />
                                Test
                            </Button>
                            <Button onClick={handleSave}>
                                <Save className="h-4 w-4 me-2" />
                                Save Route
                            </Button>
                        </div>

                        {/* Basic Info */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Basic Information</CardTitle>
                                <CardDescription>Name and path for this route</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="grid gap-4 md:grid-cols-2">
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Route Name</label>
                                        <Input
                                            placeholder="e.g., User Service API"
                                            value={name}
                                            onChange={(e) => setName(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Path Pattern</label>
                                        <Input
                                            placeholder="/api/v1/users"
                                            value={path}
                                            onChange={(e) => setPath(e.target.value)}
                                        />
                                    </div>
                                </div>
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">HTTP Methods</label>
                                    <div className="flex gap-2 flex-wrap">
                                        {HTTP_METHODS.map((method) => (
                                            <Button
                                                key={method}
                                                variant="outline"
                                                size="sm"
                                                className={cn(
                                                    "text-sm font-bold",
                                                    methods.includes(method) && `${methodColors[method]} text-white border-0`
                                                )}
                                                onClick={() => toggleMethod(method)}
                                            >
                                                {method}
                                            </Button>
                                        ))}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Upstream */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Upstream Configuration</CardTitle>
                                <CardDescription>Target service for this route</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">Upstream URL</label>
                                    <Input
                                        placeholder="http://backend-service:8080"
                                        value={upstreamUrl}
                                        onChange={(e) => setUpstreamUrl(e.target.value)}
                                    />
                                </div>
                                <div className="grid gap-4 md:grid-cols-2">
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Timeout (ms)</label>
                                        <Input
                                            type="number"
                                            value={timeout}
                                            onChange={(e) => setTimeout(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Retries</label>
                                        <Input
                                            type="number"
                                            value={retries}
                                            onChange={(e) => setRetries(e.target.value)}
                                        />
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Security */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Security</CardTitle>
                                <CardDescription>Authentication and rate limiting</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="grid gap-4 md:grid-cols-2">
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Authentication</label>
                                        <div className="flex gap-2">
                                            {(["none", "jwt", "api_key"] as const).map((type) => (
                                                <Button
                                                    key={type}
                                                    variant={authType === type ? "default" : "outline"}
                                                    size="sm"
                                                    onClick={() => setAuthType(type)}
                                                >
                                                    {type === "none" ? "None" : type === "jwt" ? "JWT" : "API Key"}
                                                </Button>
                                            ))}
                                        </div>
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Rate Limit (req/min)</label>
                                        <Input
                                            type="number"
                                            value={rateLimit}
                                            onChange={(e) => setRateLimit(e.target.value)}
                                        />
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Transformations */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Transformations</CardTitle>
                                <CardDescription>Apply data transformations to requests/responses</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="border-2 border-dashed rounded-lg p-8 text-center text-muted-foreground">
                                    <Plus className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                    <p>No transformations added</p>
                                    <Button variant="link" className="mt-2">
                                        Add Transformation
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    );
}
