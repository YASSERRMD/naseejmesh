"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";
import { Search, Sun, Moon, Languages, Command } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useThemeStore } from "@/stores/ui-store";
import { ConnectionStatus } from "@/components/connection-status";

interface HeaderProps {
    locale: string;
}

export function Header({ locale }: HeaderProps) {
    const pathname = usePathname();
    const t = useTranslations("common");
    const tHeader = useTranslations("header");
    const { theme, setTheme } = useThemeStore();
    const isRTL = locale === "ar";

    // Generate breadcrumbs from pathname
    const segments = pathname.split("/").filter((s) => s && s !== locale);
    const breadcrumbs = segments.map((segment, index) => ({
        label: segment.charAt(0).toUpperCase() + segment.slice(1),
        href: `/${locale}/${segments.slice(0, index + 1).join("/")}`,
        isLast: index === segments.length - 1,
    }));

    const toggleTheme = () => {
        if (theme === "light") {
            setTheme("dark");
            document.documentElement.classList.add("dark");
        } else {
            setTheme("light");
            document.documentElement.classList.remove("dark");
        }
    };

    const toggleLocale = () => {
        const newLocale = locale === "en" ? "ar" : "en";
        const newPath = pathname.replace(`/${locale}`, `/${newLocale}`);
        window.location.href = newPath;
    };

    return (
        <header className="sticky top-0 z-30 h-16 glass border-b border-border">
            <div className="h-full flex items-center justify-between px-6">
                {/* Breadcrumbs */}
                <nav className="flex items-center gap-2 text-sm">
                    <Link href={`/${locale}/dashboard`} className="text-muted-foreground hover:text-foreground">
                        {t("appName")}
                    </Link>
                    {breadcrumbs.map((crumb) => (
                        <React.Fragment key={crumb.href}>
                            <span className="text-muted-foreground">/</span>
                            {crumb.isLast ? (
                                <span className="font-medium">{crumb.label}</span>
                            ) : (
                                <Link href={crumb.href} className="text-muted-foreground hover:text-foreground">
                                    {crumb.label}
                                </Link>
                            )}
                        </React.Fragment>
                    ))}
                </nav>

                {/* Actions */}
                <div className="flex items-center gap-2">
                    {/* Connection Status */}
                    <ConnectionStatus />

                    {/* Search Trigger */}
                    <Button
                        variant="outline"
                        size="sm"
                        className="hidden md:flex items-center gap-2 text-muted-foreground"
                    >
                        <Search className="h-4 w-4" />
                        <span>{t("search")}</span>
                        <kbd className="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
                            <Command className="h-3 w-3" />K
                        </kbd>
                    </Button>

                    {/* Language Toggle */}
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={toggleLocale}
                        title={tHeader("language")}
                    >
                        <Languages className="h-5 w-5" />
                    </Button>

                    {/* Theme Toggle */}
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={toggleTheme}
                        title={tHeader("theme")}
                    >
                        {theme === "dark" ? (
                            <Sun className="h-5 w-5" />
                        ) : (
                            <Moon className="h-5 w-5" />
                        )}
                    </Button>
                </div>
            </div>
        </header>
    );
}
