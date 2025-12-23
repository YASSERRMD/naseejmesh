"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import {
    Shield,
    Key,
    Lock,
    AlertTriangle,
    CheckCircle,
    Ban,
    Eye,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface SecurityPageProps {
    params: Promise<{ locale: string }>;
}

interface SecurityEvent {
    id: string;
    type: "blocked" | "warning" | "allowed";
    message: string;
    source: string;
    timestamp: string;
}

const mockEvents: SecurityEvent[] = [
    {
        id: "1",
        type: "blocked",
        message: "SQL Injection attempt detected",
        source: "192.168.1.45",
        timestamp: "2 minutes ago",
    },
    {
        id: "2",
        type: "warning",
        message: "Rate limit exceeded",
        source: "10.0.0.23",
        timestamp: "5 minutes ago",
    },
    {
        id: "3",
        type: "blocked",
        message: "XSS payload in request body",
        source: "192.168.1.89",
        timestamp: "10 minutes ago",
    },
    {
        id: "4",
        type: "allowed",
        message: "Valid JWT token verified",
        source: "10.0.0.15",
        timestamp: "15 minutes ago",
    },
    {
        id: "5",
        type: "warning",
        message: "Unusual request pattern detected",
        source: "192.168.1.112",
        timestamp: "30 minutes ago",
    },
];

export default function SecurityPage({ params }: SecurityPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");

    const blocked = mockEvents.filter((e) => e.type === "blocked").length;
    const warnings = mockEvents.filter((e) => e.type === "warning").length;

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
                                <h1 className="text-3xl font-bold tracking-tight">{t("security")}</h1>
                                <p className="text-muted-foreground mt-1">
                                    Monitor security events and configure WAF rules
                                </p>
                            </div>
                            <Button variant="outline">
                                <Key className="h-4 w-4 me-2" />
                                Configure WAF
                            </Button>
                        </div>

                        {/* Stats */}
                        <div className="grid gap-4 md:grid-cols-4">
                            <StatsCard
                                icon={Ban}
                                title="Blocked Today"
                                value={blocked.toString()}
                                color="text-destructive"
                            />
                            <StatsCard
                                icon={AlertTriangle}
                                title="Warnings"
                                value={warnings.toString()}
                                color="text-yellow-500"
                            />
                            <StatsCard
                                icon={CheckCircle}
                                title="Requests Allowed"
                                value="12,847"
                                color="text-green-500"
                            />
                            <StatsCard
                                icon={Shield}
                                title="WAF Rules Active"
                                value="24"
                                color="text-primary"
                            />
                        </div>

                        {/* Security Features */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <FeatureCard
                                icon={Shield}
                                title="WAF Protection"
                                description="SQL Injection, XSS, and path traversal protection"
                                status="active"
                            />
                            <FeatureCard
                                icon={Lock}
                                title="Rate Limiting"
                                description="100 requests per minute per IP"
                                status="active"
                            />
                            <FeatureCard
                                icon={Key}
                                title="JWT Validation"
                                description="RS256 token verification enabled"
                                status="active"
                            />
                        </div>

                        {/* Recent Events */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Eye className="h-5 w-5" />
                                    Recent Security Events
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="space-y-3">
                                    {mockEvents.map((event) => (
                                        <div
                                            key={event.id}
                                            className="flex items-center justify-between p-3 rounded-lg border hover:bg-muted/50"
                                        >
                                            <div className="flex items-center gap-3">
                                                {event.type === "blocked" && (
                                                    <Ban className="h-5 w-5 text-destructive" />
                                                )}
                                                {event.type === "warning" && (
                                                    <AlertTriangle className="h-5 w-5 text-yellow-500" />
                                                )}
                                                {event.type === "allowed" && (
                                                    <CheckCircle className="h-5 w-5 text-green-500" />
                                                )}
                                                <div>
                                                    <p className="font-medium">{event.message}</p>
                                                    <p className="text-sm text-muted-foreground">
                                                        From {event.source}
                                                    </p>
                                                </div>
                                            </div>
                                            <div className="flex items-center gap-3">
                                                <Badge
                                                    variant={
                                                        event.type === "blocked"
                                                            ? "destructive"
                                                            : event.type === "warning"
                                                                ? "warning"
                                                                : "success"
                                                    }
                                                >
                                                    {event.type}
                                                </Badge>
                                                <span className="text-sm text-muted-foreground">
                                                    {event.timestamp}
                                                </span>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    );
}

function StatsCard({
    icon: Icon,
    title,
    value,
    color,
}: {
    icon: React.ElementType;
    title: string;
    value: string;
    color: string;
}) {
    return (
        <div className="rounded-lg border bg-card p-4">
            <div className="flex items-center gap-2">
                <Icon className={cn("h-5 w-5", color)} />
                <p className="text-sm text-muted-foreground">{title}</p>
            </div>
            <p className="text-2xl font-bold mt-1">{value}</p>
        </div>
    );
}

function FeatureCard({
    icon: Icon,
    title,
    description,
    status,
}: {
    icon: React.ElementType;
    title: string;
    description: string;
    status: "active" | "inactive";
}) {
    return (
        <Card>
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                            <Icon className="h-5 w-5 text-primary" />
                        </div>
                        <CardTitle className="text-base">{title}</CardTitle>
                    </div>
                    <Badge variant={status === "active" ? "success" : "secondary"}>
                        {status}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent>
                <CardDescription>{description}</CardDescription>
            </CardContent>
        </Card>
    );
}
