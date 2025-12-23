"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { Activity, Route, Workflow, Shield } from "lucide-react";
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

    return (
        <div className="min-h-screen bg-background">
            <Sidebar locale={locale} />
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
                                Overview of your API Gateway and integrations
                            </p>
                        </div>

                        {/* Stats Grid */}
                        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                            <StatCard icon={Route} title="Active Routes" value="12" change="+2 from last week" />
                            <StatCard icon={Workflow} title="Transformations" value="5" change="3 active" />
                            <StatCard icon={Activity} title="Requests/min" value="4,231" change="+12% from yesterday" />
                            <StatCard icon={Shield} title="Blocked Threats" value="47" change="Last 24 hours" />
                        </div>

                        {/* Gateway Status */}
                        <div className="rounded-lg border bg-card p-6">
                            <h2 className="text-lg font-semibold mb-4">{tGateway("status")}</h2>
                            <div className="grid gap-4 md:grid-cols-3">
                                <div className="flex items-center gap-3">
                                    <div className="h-3 w-3 rounded-full bg-green-500 animate-pulse" />
                                    <div>
                                        <p className="text-sm text-muted-foreground">Status</p>
                                        <p className="font-medium">{tGateway("healthy")}</p>
                                    </div>
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">{tGateway("version")}</p>
                                    <p className="font-medium">v0.1.0</p>
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">{tGateway("uptime")}</p>
                                    <p className="font-medium">2h 34m</p>
                                </div>
                            </div>
                        </div>
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
    change: string;
}

function StatCard({ icon: Icon, title, value, change }: StatCardProps) {
    return (
        <div className="rounded-lg border bg-card p-6">
            <div className="flex items-center gap-2">
                <Icon className="h-5 w-5 text-muted-foreground" />
                <span className="text-sm font-medium text-muted-foreground">{title}</span>
            </div>
            <p className="text-2xl font-bold mt-2">{value}</p>
            <p className="text-xs text-muted-foreground mt-1">{change}</p>
        </div>
    );
}
