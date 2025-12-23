"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import {
    Activity,
    Clock,
    TrendingUp,
    TrendingDown,
    AlertCircle,
    CheckCircle2,
    Server,
    Cpu,
    MemoryStick,
    HardDrive,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface MonitoringPageProps {
    params: Promise<{ locale: string }>;
}

export default function MonitoringPage({ params }: MonitoringPageProps) {
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
                        <div>
                            <h1 className="text-3xl font-bold tracking-tight">{t("monitoring")}</h1>
                            <p className="text-muted-foreground mt-1">
                                Real-time metrics and system health
                            </p>
                        </div>

                        {/* Health Status */}
                        <div className="grid gap-4 md:grid-cols-4">
                            <HealthCard
                                icon={Server}
                                title="Gateway"
                                status="healthy"
                                detail="Running v0.1.0"
                            />
                            <HealthCard
                                icon={Cpu}
                                title="CPU Usage"
                                status="healthy"
                                detail="23%"
                            />
                            <HealthCard
                                icon={MemoryStick}
                                title="Memory"
                                status="warning"
                                detail="78% (2.3GB / 3GB)"
                            />
                            <HealthCard
                                icon={HardDrive}
                                title="Disk"
                                status="healthy"
                                detail="45% used"
                            />
                        </div>

                        {/* Request Metrics */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <Card>
                                <CardHeader>
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Activity className="h-4 w-4" />
                                        Requests/sec
                                    </CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="text-3xl font-bold">127</div>
                                    <div className="flex items-center gap-1 mt-1 text-sm text-green-500">
                                        <TrendingUp className="h-4 w-4" />
                                        +12% from last hour
                                    </div>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader>
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Clock className="h-4 w-4" />
                                        Avg Latency
                                    </CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="text-3xl font-bold">45ms</div>
                                    <div className="flex items-center gap-1 mt-1 text-sm text-green-500">
                                        <TrendingDown className="h-4 w-4" />
                                        -8% from last hour
                                    </div>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader>
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <AlertCircle className="h-4 w-4" />
                                        Error Rate
                                    </CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="text-3xl font-bold">0.02%</div>
                                    <div className="flex items-center gap-1 mt-1 text-sm text-muted-foreground">
                                        2 errors in last hour
                                    </div>
                                </CardContent>
                            </Card>
                        </div>

                        {/* Uptime */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Clock className="h-5 w-5" />
                                    Uptime History
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="flex gap-1">
                                    {/* Simulated uptime bars for last 30 days */}
                                    {Array.from({ length: 30 }).map((_, i) => (
                                        <div
                                            key={i}
                                            className={cn(
                                                "flex-1 h-8 rounded",
                                                i === 15 ? "bg-yellow-500" : "bg-green-500"
                                            )}
                                            title={`Day ${30 - i}: ${i === 15 ? "99.5%" : "100%"}`}
                                        />
                                    ))}
                                </div>
                                <div className="flex justify-between mt-2 text-xs text-muted-foreground">
                                    <span>30 days ago</span>
                                    <span>Today</span>
                                </div>
                                <div className="mt-4 text-center">
                                    <span className="text-2xl font-bold">99.98%</span>
                                    <span className="text-muted-foreground ms-2">overall uptime</span>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    );
}

function HealthCard({
    icon: Icon,
    title,
    status,
    detail,
}: {
    icon: React.ElementType;
    title: string;
    status: "healthy" | "warning" | "error";
    detail: string;
}) {
    return (
        <Card>
            <CardContent className="pt-6">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div
                            className={cn(
                                "h-10 w-10 rounded-lg flex items-center justify-center",
                                status === "healthy" && "bg-green-500/10",
                                status === "warning" && "bg-yellow-500/10",
                                status === "error" && "bg-destructive/10"
                            )}
                        >
                            <Icon
                                className={cn(
                                    "h-5 w-5",
                                    status === "healthy" && "text-green-500",
                                    status === "warning" && "text-yellow-500",
                                    status === "error" && "text-destructive"
                                )}
                            />
                        </div>
                        <div>
                            <p className="font-medium">{title}</p>
                            <p className="text-sm text-muted-foreground">{detail}</p>
                        </div>
                    </div>
                    {status === "healthy" && <CheckCircle2 className="h-5 w-5 text-green-500" />}
                    {status === "warning" && <AlertCircle className="h-5 w-5 text-yellow-500" />}
                </div>
            </CardContent>
        </Card>
    );
}
