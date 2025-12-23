"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";
import {
    LayoutDashboard,
    Route,
    Workflow,
    FileJson,
    Shield,
    Activity,
    ChevronLeft,
    ChevronRight,
    Plus,
    Upload,
    TestTube,
    Zap,
    Settings,
    GitBranch,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useSidebarStore } from "@/stores/ui-store";

interface NavItem {
    href: string;
    icon: React.ElementType;
    labelKey: string;
    badge?: number;
}

const navItems: NavItem[] = [
    { href: "/dashboard", icon: LayoutDashboard, labelKey: "dashboard" },
    { href: "/design", icon: GitBranch, labelKey: "design" },
    { href: "/routes", icon: Route, labelKey: "routes", badge: 3 },
    { href: "/transformations", icon: Workflow, labelKey: "transformations", badge: 1 },
    { href: "/schemas", icon: FileJson, labelKey: "schemas" },
    { href: "/security", icon: Shield, labelKey: "security" },
    { href: "/monitoring", icon: Activity, labelKey: "monitoring" },
    { href: "/settings", icon: Settings, labelKey: "settings" },
];

interface SidebarProps {
    locale: string;
}

export function Sidebar({ locale }: SidebarProps) {
    const pathname = usePathname();
    const t = useTranslations("nav");
    const tSidebar = useTranslations("sidebar");
    const tActions = useTranslations("actions");
    const tGateway = useTranslations("gateway");
    const { isCollapsed, toggle } = useSidebarStore();
    const isRTL = locale === "ar";

    return (
        <aside
            className={cn(
                "fixed top-0 h-screen bg-card border-e border-border transition-all duration-300 z-40",
                isCollapsed ? "w-16" : "w-64",
                isRTL ? "end-0" : "start-0"
            )}
        >
            {/* Logo */}
            <div className="h-16 flex items-center justify-center border-b border-border">
                {isCollapsed ? (
                    <Zap className="h-8 w-8 text-primary" />
                ) : (
                    <div className="flex items-center gap-2">
                        <Zap className="h-8 w-8 text-primary" />
                        <span className="font-semibold text-lg">Naseej</span>
                    </div>
                )}
            </div>

            {/* Navigation */}
            <nav className="p-2 space-y-1">
                {navItems.map((item) => {
                    const isActive = pathname.includes(item.href);
                    const Icon = item.icon;

                    return (
                        <Link
                            key={item.href}
                            href={`/${locale}${item.href}`}
                            className={cn(
                                "flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors",
                                isActive
                                    ? "bg-primary/10 text-primary"
                                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                            )}
                        >
                            <Icon className="h-5 w-5 shrink-0" />
                            {!isCollapsed && (
                                <>
                                    <span className="flex-1">{t(item.labelKey)}</span>
                                    {item.badge && (
                                        <span className="bg-primary/20 text-primary text-xs px-2 py-0.5 rounded-full">
                                            {item.badge}
                                        </span>
                                    )}
                                </>
                            )}
                        </Link>
                    );
                })}
            </nav>

            {/* Quick Actions */}
            {!isCollapsed && (
                <div className="px-3 py-4 border-t border-border">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">
                        Quick Actions
                    </p>
                    <div className="space-y-1">
                        <Link href={`/${locale}/routes/new`}>
                            <Button variant="ghost" size="sm" className="w-full justify-start gap-2">
                                <Plus className="h-4 w-4" />
                                {tActions("newRoute")}
                            </Button>
                        </Link>
                        <Button variant="ghost" size="sm" className="w-full justify-start gap-2">
                            <Upload className="h-4 w-4" />
                            {tActions("importOpenAPI")}
                        </Button>
                        <Button variant="ghost" size="sm" className="w-full justify-start gap-2">
                            <TestTube className="h-4 w-4" />
                            {tActions("testTransform")}
                        </Button>
                    </div>
                </div>
            )}

            {/* Gateway Status */}
            {!isCollapsed && (
                <div className="absolute bottom-16 left-0 right-0 px-3 py-4 border-t border-border">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">
                        {tGateway("status")}
                    </p>
                    <div className="bg-accent/50 rounded-lg p-3 space-y-2">
                        <div className="flex items-center gap-2">
                            <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                            <span className="text-sm font-medium">{tGateway("healthy")}</span>
                            <span className="text-xs text-muted-foreground ms-auto">v0.1.0</span>
                        </div>
                        <div className="grid grid-cols-2 gap-2 text-xs">
                            <div>
                                <span className="text-muted-foreground">{tGateway("uptime")}</span>
                                <p className="font-medium">2h 34m</p>
                            </div>
                            <div>
                                <span className="text-muted-foreground">{tGateway("requests")}</span>
                                <p className="font-medium">1,234</p>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Collapse Toggle */}
            <button
                onClick={toggle}
                className={cn(
                    "absolute bottom-4 p-2 rounded-lg bg-accent hover:bg-accent/80 transition-colors",
                    isCollapsed ? "start-1/2 -translate-x-1/2" : isRTL ? "start-3" : "end-3"
                )}
                title={isCollapsed ? tSidebar("expand") : tSidebar("collapse")}
            >
                {isCollapsed ? (
                    isRTL ? <ChevronLeft className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />
                ) : (
                    isRTL ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />
                )}
            </button>
        </aside>
    );
}
