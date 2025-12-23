"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore, useThemeStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use } from "react";
import { useTranslations } from "next-intl";
import {
    Settings,
    Globe,
    Moon,
    Sun,
    Server,
    Shield,
    Bell,
    Database,
    Key,
    RefreshCcw,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface SettingsPageProps {
    params: Promise<{ locale: string }>;
}

export default function SettingsPage({ params }: SettingsPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const { theme, setTheme } = useThemeStore();
    const isRTL = locale === "ar";

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
                        <div>
                            <h1 className="text-3xl font-bold tracking-tight flex items-center gap-2">
                                <Settings className="h-8 w-8" />
                                Settings
                            </h1>
                            <p className="text-muted-foreground mt-1">
                                Configure your gateway and console preferences
                            </p>
                        </div>

                        {/* Appearance */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Moon className="h-5 w-5" />
                                    Appearance
                                </CardTitle>
                                <CardDescription>Customize the console appearance</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="font-medium">Theme</p>
                                        <p className="text-sm text-muted-foreground">
                                            Switch between light and dark mode
                                        </p>
                                    </div>
                                    <div className="flex gap-2">
                                        <Button
                                            variant={theme === "light" ? "default" : "outline"}
                                            size="sm"
                                            onClick={() => setTheme("light")}
                                        >
                                            <Sun className="h-4 w-4 me-1" />
                                            Light
                                        </Button>
                                        <Button
                                            variant={theme === "dark" ? "default" : "outline"}
                                            size="sm"
                                            onClick={() => setTheme("dark")}
                                        >
                                            <Moon className="h-4 w-4 me-1" />
                                            Dark
                                        </Button>
                                    </div>
                                </div>
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="font-medium">Language</p>
                                        <p className="text-sm text-muted-foreground">
                                            Current: {locale === "en" ? "English" : "العربية"}
                                        </p>
                                    </div>
                                    <Badge variant="outline">
                                        <Globe className="h-3 w-3 me-1" />
                                        {locale.toUpperCase()}
                                    </Badge>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Gateway Connection */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Server className="h-5 w-5" />
                                    Gateway Connection
                                </CardTitle>
                                <CardDescription>Configure the gateway API endpoint</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">Gateway URL</label>
                                    <div className="flex gap-2">
                                        <Input
                                            defaultValue="http://localhost:3001"
                                            className="flex-1"
                                        />
                                        <Button variant="outline">
                                            <RefreshCcw className="h-4 w-4 me-1" />
                                            Test Connection
                                        </Button>
                                    </div>
                                </div>
                                <div className="flex items-center justify-between p-3 rounded-lg border bg-muted/50">
                                    <div className="flex items-center gap-3">
                                        <div className="h-3 w-3 rounded-full bg-green-500 animate-pulse" />
                                        <span className="font-medium">Connected</span>
                                    </div>
                                    <span className="text-sm text-muted-foreground">v0.1.0</span>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Security */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Shield className="h-5 w-5" />
                                    Security
                                </CardTitle>
                                <CardDescription>API keys and authentication</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">Console API Key</label>
                                    <div className="flex gap-2">
                                        <Input
                                            type="password"
                                            defaultValue="nsk_1234567890abcdef"
                                            className="flex-1 font-mono"
                                        />
                                        <Button variant="outline">
                                            <Key className="h-4 w-4 me-1" />
                                            Regenerate
                                        </Button>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Database */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Database className="h-5 w-5" />
                                    Database
                                </CardTitle>
                                <CardDescription>SurrealDB connection settings</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="grid gap-4 md:grid-cols-2">
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Host</label>
                                        <Input defaultValue="localhost:8000" />
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-sm font-medium">Namespace</label>
                                        <Input defaultValue="naseej" />
                                    </div>
                                </div>
                                <div className="flex items-center justify-between p-3 rounded-lg border bg-muted/50">
                                    <div className="flex items-center gap-3">
                                        <div className="h-3 w-3 rounded-full bg-green-500 animate-pulse" />
                                        <span className="font-medium">Database Connected</span>
                                    </div>
                                    <span className="text-sm text-muted-foreground">SurrealDB v1.5</span>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Notifications */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Bell className="h-5 w-5" />
                                    Notifications
                                </CardTitle>
                                <CardDescription>Alert preferences</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="space-y-3">
                                    {[
                                        { label: "Security alerts", enabled: true },
                                        { label: "Error notifications", enabled: true },
                                        { label: "Rate limit warnings", enabled: false },
                                        { label: "Route changes", enabled: true },
                                    ].map((item) => (
                                        <div
                                            key={item.label}
                                            className="flex items-center justify-between py-2"
                                        >
                                            <span>{item.label}</span>
                                            <Badge variant={item.enabled ? "success" : "secondary"}>
                                                {item.enabled ? "On" : "Off"}
                                            </Badge>
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
