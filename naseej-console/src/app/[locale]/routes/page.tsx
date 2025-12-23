"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import {
    Route,
    Plus,
    MoreHorizontal,
    Play,
    Pause,
    Trash2,
    ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface RoutesPageProps {
    params: Promise<{ locale: string }>;
}

interface RouteItem {
    id: string;
    name: string;
    path: string;
    method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
    upstream: string;
    status: "active" | "inactive" | "error";
    requests: number;
    latency: string;
}

const mockRoutes: RouteItem[] = [
    {
        id: "1",
        name: "User API",
        path: "/api/users",
        method: "GET",
        upstream: "http://users-service:8080",
        status: "active",
        requests: 1234,
        latency: "45ms",
    },
    {
        id: "2",
        name: "Orders API",
        path: "/api/orders",
        method: "POST",
        upstream: "http://orders-service:8080",
        status: "active",
        requests: 567,
        latency: "89ms",
    },
    {
        id: "3",
        name: "Products API",
        path: "/api/products",
        method: "GET",
        upstream: "http://products-service:8080",
        status: "inactive",
        requests: 0,
        latency: "-",
    },
    {
        id: "4",
        name: "Auth API",
        path: "/api/auth/login",
        method: "POST",
        upstream: "http://auth-service:8080",
        status: "active",
        requests: 890,
        latency: "23ms",
    },
    {
        id: "5",
        name: "Legacy SOAP",
        path: "/api/legacy/soap",
        method: "POST",
        upstream: "http://legacy:8080/soap",
        status: "error",
        requests: 12,
        latency: "500ms",
    },
];

const methodColors: Record<string, string> = {
    GET: "bg-green-500",
    POST: "bg-blue-500",
    PUT: "bg-yellow-500",
    DELETE: "bg-red-500",
    PATCH: "bg-purple-500",
};

export default function RoutesPage({ params }: RoutesPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");
    const tActions = useTranslations("actions");

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
                                <h1 className="text-3xl font-bold tracking-tight">{t("routes")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Manage your API routes and upstream services
                                </p>
                            </div>
                            <Button>
                                <Plus className="h-4 w-4 me-2" />
                                {tActions("newRoute")}
                            </Button>
                        </div>

                        {/* Stats */}
                        <div className="grid gap-4 md:grid-cols-4">
                            <StatsCard title="Total Routes" value={mockRoutes.length.toString()} />
                            <StatsCard
                                title="Active"
                                value={mockRoutes.filter((r) => r.status === "active").length.toString()}
                            />
                            <StatsCard
                                title="Total Requests"
                                value={mockRoutes.reduce((acc, r) => acc + r.requests, 0).toLocaleString()}
                            />
                            <StatsCard title="Avg Latency" value="52ms" />
                        </div>

                        {/* Routes Table */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Route className="h-5 w-5" />
                                    All Routes
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="overflow-x-auto">
                                    <table className="w-full">
                                        <thead>
                                            <tr className="border-b">
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Name
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Method
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Path
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Upstream
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Status
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Requests
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Latency
                                                </th>
                                                <th className="text-start py-3 px-4 text-sm font-medium text-muted-foreground">
                                                    Actions
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {mockRoutes.map((route) => (
                                                <tr key={route.id} className="border-b hover:bg-muted/50">
                                                    <td className="py-3 px-4">
                                                        <span className="font-medium">{route.name}</span>
                                                    </td>
                                                    <td className="py-3 px-4">
                                                        <span
                                                            className={cn(
                                                                "px-2 py-1 rounded text-xs font-bold text-white",
                                                                methodColors[route.method]
                                                            )}
                                                        >
                                                            {route.method}
                                                        </span>
                                                    </td>
                                                    <td className="py-3 px-4">
                                                        <code className="text-sm bg-muted px-2 py-1 rounded">
                                                            {route.path}
                                                        </code>
                                                    </td>
                                                    <td className="py-3 px-4 text-sm text-muted-foreground">
                                                        {route.upstream}
                                                    </td>
                                                    <td className="py-3 px-4">
                                                        <Badge
                                                            variant={
                                                                route.status === "active"
                                                                    ? "success"
                                                                    : route.status === "error"
                                                                        ? "destructive"
                                                                        : "secondary"
                                                            }
                                                        >
                                                            {route.status}
                                                        </Badge>
                                                    </td>
                                                    <td className="py-3 px-4 text-sm">
                                                        {route.requests.toLocaleString()}
                                                    </td>
                                                    <td className="py-3 px-4 text-sm">{route.latency}</td>
                                                    <td className="py-3 px-4">
                                                        <div className="flex items-center gap-1">
                                                            <Button variant="ghost" size="icon" title="Test">
                                                                <ExternalLink className="h-4 w-4" />
                                                            </Button>
                                                            {route.status === "active" ? (
                                                                <Button variant="ghost" size="icon" title="Pause">
                                                                    <Pause className="h-4 w-4" />
                                                                </Button>
                                                            ) : (
                                                                <Button variant="ghost" size="icon" title="Activate">
                                                                    <Play className="h-4 w-4" />
                                                                </Button>
                                                            )}
                                                            <Button variant="ghost" size="icon" title="Delete">
                                                                <Trash2 className="h-4 w-4 text-destructive" />
                                                            </Button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            </CardContent>
                        </Card>
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
