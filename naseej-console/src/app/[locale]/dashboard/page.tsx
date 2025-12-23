"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { useGatewayStatus, useMetrics, useSecurityEvents } from "@/lib/hooks";
import { cn } from "@/lib/utils";
import { use } from "react";
import { Activity, Route, Workflow, Shield, TrendingUp, TrendingDown, AlertCircle } from "lucide-react";
import { useTranslations } from "next-intl";

interface DashboardPageProps {
    params: Promise<{ locale: string }>;
}

export default function DashboardPage({ params }: DashboardPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");
    const tGateway = useTranslations("gateway");

    // Real-time data hooks
    const { data: gatewayStatus, isLoading: statusLoading } = useGatewayStatus();
    const { data: metrics } = useMetrics();
    const { data: securityEvents } = useSecurityEvents(5);

    const blockedCount = securityEvents?.filter(e => e.type === "blocked").length || 0;

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
                        <div>
                            <h1 className="text-3xl font-bold tracking-tight">{t("dashboard")}</h1>
                            <p className="text-muted-foreground mt-1">
                                Real-time overview of your API Gateway
                            </p>
                        </div>

                        {/* Stats Grid */}
                        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                            <StatCard
                                icon={Route}
                                title="Active Routes"
                                value={gatewayStatus?.routes?.toString() || "12"}
                                trend={+2}
                                loading={statusLoading}
                            />
                            <StatCard
                                icon={Workflow}
                                title="Transformations"
                                value="5"
                                subtext="3 active"
                            />
                            <StatCard
                                icon={Activity}
                                title="Requests/sec"
                                value={metrics?.perSecond?.toLocaleString() || "127"}
                                trend={+12}
                                loading={!metrics}
                            />
                            <StatCard
                                icon={Shield}
                                title="Blocked Today"
                                value={blockedCount.toString()}
                                variant="warning"
                            />
                        </div>

                        {/* Gateway Status */}
                        <div className="rounded-lg border bg-card p-6">
                            <h2 className="text-lg font-semibold mb-4">{tGateway("status")}</h2>
                            <div className="grid gap-4 md:grid-cols-4">
                                <div className="flex items-center gap-3">
                                    <div className={cn(
                                        "h-3 w-3 rounded-full animate-pulse",
                                        gatewayStatus?.healthy ? "bg-green-500" : "bg-red-500"
                                    )} />
                                    <div>
                                        <p className="text-sm text-muted-foreground">Status</p>
                                        <p className="font-medium">
                                            {gatewayStatus?.healthy ? tGateway("healthy") : tGateway("unhealthy")}
                                        </p>
                                    </div>
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">{tGateway("version")}</p>
                                    <p className="font-medium">{gatewayStatus?.version || "v0.1.0"}</p>
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">{tGateway("uptime")}</p>
                                    <p className="font-medium">{formatUptime(gatewayStatus?.uptime || 0)}</p>
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">Avg Latency</p>
                                    <p className="font-medium">{metrics?.avgLatencyMs || 45}ms</p>
                                </div>
                            </div>
                        </div>

                        {/* Recent Security Events */}
                        {securityEvents && securityEvents.length > 0 && (
                            <div className="rounded-lg border bg-card p-6">
                                <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
                                    <AlertCircle className="h-5 w-5" />
                                    Recent Security Events
                                </h2>
                                <div className="space-y-2">
                                    {securityEvents.slice(0, 5).map((event) => (
                                        <div
                                            key={event.id}
                                            className="flex items-center justify-between p-3 rounded-lg border hover:bg-muted/50"
                                        >
                                            <div className="flex items-center gap-3">
                                                <div className={cn(
                                                    "h-2 w-2 rounded-full",
                                                    event.type === "blocked" ? "bg-destructive" :
                                                        event.type === "warning" ? "bg-yellow-500" : "bg-green-500"
                                                )} />
                                                <div>
                                                    <p className="text-sm font-medium">{event.message}</p>
                                                    <p className="text-xs text-muted-foreground">From {event.source}</p>
                                                </div>
                                            </div>
                                            <span className="text-xs text-muted-foreground">
                                                {formatTimeAgo(event.timestamp)}
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                </main>
            </div>
        </div>
    );
}

interface StatCardProps {
    icon: React.ElementType;
    title: string;
    value: string;
    trend?: number;
    subtext?: string;
    loading?: boolean;
    variant?: "default" | "warning";
}

function StatCard({ icon: Icon, title, value, trend, subtext, loading, variant = "default" }: StatCardProps) {
    return (
        <div className="rounded-lg border bg-card p-6">
            <div className="flex items-center gap-2">
                <Icon className={cn(
                    "h-5 w-5",
                    variant === "warning" ? "text-yellow-500" : "text-muted-foreground"
                )} />
                <span className="text-sm font-medium text-muted-foreground">{title}</span>
            </div>
            <p className={cn("text-2xl font-bold mt-2", loading && "animate-pulse")}>
                {loading ? "..." : value}
            </p>
            {trend !== undefined && (
                <div className={cn(
                    "flex items-center gap-1 text-xs mt-1",
                    trend > 0 ? "text-green-500" : "text-red-500"
                )}>
                    {trend > 0 ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
                    {trend > 0 ? "+" : ""}{trend}% from last hour
                </div>
            )}
            {subtext && (
                <p className="text-xs text-muted-foreground mt-1">{subtext}</p>
            )}
        </div>
    );
}

function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
}

function formatTimeAgo(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
}
