"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use, useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import {
    Route,
    Plus,
    Play,
    Pause,
    Trash2,
    ExternalLink,
    RefreshCw,
    Loader2,
    AlertCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
import { getRoutes, createRoute, deleteRoute } from "@/lib/api-client";

interface RoutesPageProps {
    params: Promise<{ locale: string }>;
}

interface RouteItem {
    id: string;
    name: string;
    path: string;
    method: string;
    upstream: { url: string } | string;
    enabled: boolean;
    requests?: number;
    avgLatencyMs?: number;
}

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

    const [routes, setRoutes] = useState<RouteItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [creating, setCreating] = useState(false);

    // Form state
    const [newRoute, setNewRoute] = useState({
        name: "",
        path: "",
        method: "GET",
        upstream: "",
    });

    const fetchRoutes = async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await getRoutes();
            setRoutes(data.map((r: any) => ({
                id: r.id,
                name: r.name || r.path,
                path: r.path,
                method: r.methods?.[0] || "GET",
                upstream: r.upstream,
                enabled: r.enabled ?? true,
                requests: r.requests || 0,
                avgLatencyMs: r.avgLatencyMs || 0,
            })));
        } catch (err: any) {
            setError(err.message || "Failed to load routes");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchRoutes();
    }, []);

    const handleCreateRoute = async () => {
        if (!newRoute.path || !newRoute.upstream) return;

        setCreating(true);
        try {
            await createRoute({
                name: newRoute.name || newRoute.path,
                path: newRoute.path,
                methods: [newRoute.method as any],
                upstream: { url: newRoute.upstream },
                enabled: true,
            });
            setDialogOpen(false);
            setNewRoute({ name: "", path: "", method: "GET", upstream: "" });
            await fetchRoutes();
        } catch (err: any) {
            setError(err.message || "Failed to create route");
        } finally {
            setCreating(false);
        }
    };

    const handleDeleteRoute = async (id: string) => {
        if (!confirm("Are you sure you want to delete this route?")) return;
        try {
            await deleteRoute(id);
            await fetchRoutes();
        } catch (err: any) {
            setError(err.message || "Failed to delete route");
        }
    };

    const getUpstreamUrl = (upstream: any): string => {
        if (typeof upstream === "string") return upstream;
        return upstream?.url || "-";
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
                                <h1 className="text-3xl font-bold tracking-tight">{t("routes")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Manage your API routes and upstream services
                                </p>
                            </div>
                            <div className="flex gap-2">
                                <Button variant="outline" onClick={fetchRoutes} disabled={loading}>
                                    <RefreshCw className={cn("h-4 w-4 me-2", loading && "animate-spin")} />
                                    Refresh
                                </Button>
                                <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                                    <DialogTrigger asChild>
                                        <Button>
                                            <Plus className="h-4 w-4 me-2" />
                                            {tActions("newRoute")}
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent className="sm:max-w-[500px]">
                                        <DialogHeader>
                                            <DialogTitle>Create New Route</DialogTitle>
                                            <DialogDescription>
                                                Add a new API route to your gateway configuration.
                                            </DialogDescription>
                                        </DialogHeader>
                                        <div className="grid gap-4 py-4">
                                            <div className="grid gap-2">
                                                <Label htmlFor="name">Name</Label>
                                                <Input
                                                    id="name"
                                                    placeholder="User API"
                                                    value={newRoute.name}
                                                    onChange={(e) => setNewRoute({ ...newRoute, name: e.target.value })}
                                                />
                                            </div>
                                            <div className="grid gap-2">
                                                <Label htmlFor="path">Path *</Label>
                                                <Input
                                                    id="path"
                                                    placeholder="/api/users"
                                                    value={newRoute.path}
                                                    onChange={(e) => setNewRoute({ ...newRoute, path: e.target.value })}
                                                />
                                            </div>
                                            <div className="grid gap-2">
                                                <Label htmlFor="method">Method</Label>
                                                <Select
                                                    value={newRoute.method}
                                                    onValueChange={(value) => setNewRoute({ ...newRoute, method: value })}
                                                >
                                                    <SelectTrigger>
                                                        <SelectValue placeholder="Select method" />
                                                    </SelectTrigger>
                                                    <SelectContent>
                                                        <SelectItem value="GET">GET</SelectItem>
                                                        <SelectItem value="POST">POST</SelectItem>
                                                        <SelectItem value="PUT">PUT</SelectItem>
                                                        <SelectItem value="DELETE">DELETE</SelectItem>
                                                        <SelectItem value="PATCH">PATCH</SelectItem>
                                                    </SelectContent>
                                                </Select>
                                            </div>
                                            <div className="grid gap-2">
                                                <Label htmlFor="upstream">Upstream URL *</Label>
                                                <Input
                                                    id="upstream"
                                                    placeholder="http://backend-service:8080"
                                                    value={newRoute.upstream}
                                                    onChange={(e) => setNewRoute({ ...newRoute, upstream: e.target.value })}
                                                />
                                            </div>
                                        </div>
                                        <DialogFooter>
                                            <Button variant="outline" onClick={() => setDialogOpen(false)}>
                                                Cancel
                                            </Button>
                                            <Button onClick={handleCreateRoute} disabled={creating || !newRoute.path || !newRoute.upstream}>
                                                {creating && <Loader2 className="h-4 w-4 me-2 animate-spin" />}
                                                Create Route
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
                            <StatsCard title="Total Routes" value={routes.length.toString()} />
                            <StatsCard
                                title="Active"
                                value={routes.filter((r) => r.enabled).length.toString()}
                            />
                            <StatsCard
                                title="Total Requests"
                                value={routes.reduce((acc, r) => acc + (r.requests || 0), 0).toLocaleString()}
                            />
                            <StatsCard
                                title="Avg Latency"
                                value={routes.length > 0
                                    ? `${Math.round(routes.reduce((acc, r) => acc + (r.avgLatencyMs || 0), 0) / Math.max(routes.length, 1))}ms`
                                    : "-"
                                }
                            />
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
                                {loading ? (
                                    <div className="flex items-center justify-center py-12">
                                        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                                    </div>
                                ) : routes.length === 0 ? (
                                    <div className="text-center py-12 text-muted-foreground">
                                        <Route className="h-12 w-12 mx-auto mb-4 opacity-50" />
                                        <p>No routes configured yet.</p>
                                        <p className="text-sm">Click &quot;New Route&quot; to create your first route.</p>
                                    </div>
                                ) : (
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
                                                {routes.map((route) => (
                                                    <tr key={route.id} className="border-b hover:bg-muted/50">
                                                        <td className="py-3 px-4">
                                                            <span className="font-medium">{route.name}</span>
                                                        </td>
                                                        <td className="py-3 px-4">
                                                            <span
                                                                className={cn(
                                                                    "px-2 py-1 rounded text-xs font-bold text-white",
                                                                    methodColors[route.method] || "bg-gray-500"
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
                                                            {getUpstreamUrl(route.upstream)}
                                                        </td>
                                                        <td className="py-3 px-4">
                                                            <Badge
                                                                variant={route.enabled ? "success" : "secondary"}
                                                            >
                                                                {route.enabled ? "active" : "inactive"}
                                                            </Badge>
                                                        </td>
                                                        <td className="py-3 px-4 text-sm">
                                                            {(route.requests || 0).toLocaleString()}
                                                        </td>
                                                        <td className="py-3 px-4 text-sm">
                                                            {route.avgLatencyMs ? `${route.avgLatencyMs}ms` : "-"}
                                                        </td>
                                                        <td className="py-3 px-4">
                                                            <div className="flex items-center gap-1">
                                                                <Button variant="ghost" size="icon" title="Test">
                                                                    <ExternalLink className="h-4 w-4" />
                                                                </Button>
                                                                {route.enabled ? (
                                                                    <Button variant="ghost" size="icon" title="Pause">
                                                                        <Pause className="h-4 w-4" />
                                                                    </Button>
                                                                ) : (
                                                                    <Button variant="ghost" size="icon" title="Activate">
                                                                        <Play className="h-4 w-4" />
                                                                    </Button>
                                                                )}
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    title="Delete"
                                                                    onClick={() => handleDeleteRoute(route.id)}
                                                                >
                                                                    <Trash2 className="h-4 w-4 text-destructive" />
                                                                </Button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                )}
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
