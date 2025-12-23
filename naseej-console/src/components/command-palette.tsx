"use client";

import * as React from "react";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
    LayoutDashboard,
    Route,
    Workflow,
    FileJson,
    Shield,
    Activity,
    Plus,
    Upload,
    TestTube,
    Settings,
    Search,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface CommandItem {
    id: string;
    label: string;
    icon: React.ElementType;
    action: () => void;
    category: "navigation" | "action" | "settings";
    shortcut?: string;
}

interface CommandPaletteProps {
    locale: string;
}

export function CommandPalette({ locale }: CommandPaletteProps) {
    const [open, setOpen] = React.useState(false);
    const [search, setSearch] = React.useState("");
    const [selectedIndex, setSelectedIndex] = React.useState(0);
    const router = useRouter();
    const t = useTranslations("nav");
    const tActions = useTranslations("actions");

    const commands: CommandItem[] = React.useMemo(
        () => [
            // Navigation
            {
                id: "dashboard",
                label: t("dashboard"),
                icon: LayoutDashboard,
                action: () => router.push(`/${locale}/dashboard`),
                category: "navigation",
            },
            {
                id: "routes",
                label: t("routes"),
                icon: Route,
                action: () => router.push(`/${locale}/routes`),
                category: "navigation",
            },
            {
                id: "transformations",
                label: t("transformations"),
                icon: Workflow,
                action: () => router.push(`/${locale}/transformations`),
                category: "navigation",
            },
            {
                id: "schemas",
                label: t("schemas"),
                icon: FileJson,
                action: () => router.push(`/${locale}/schemas`),
                category: "navigation",
            },
            {
                id: "security",
                label: t("security"),
                icon: Shield,
                action: () => router.push(`/${locale}/security`),
                category: "navigation",
            },
            {
                id: "monitoring",
                label: t("monitoring"),
                icon: Activity,
                action: () => router.push(`/${locale}/monitoring`),
                category: "navigation",
            },
            // Actions
            {
                id: "new-route",
                label: tActions("newRoute"),
                icon: Plus,
                action: () => {
                    router.push(`/${locale}/routes/new`);
                },
                category: "action",
                shortcut: "⌘N",
            },
            {
                id: "import-openapi",
                label: tActions("importOpenAPI"),
                icon: Upload,
                action: () => {
                    // TODO: Open import modal
                    console.log("Import OpenAPI");
                },
                category: "action",
                shortcut: "⌘I",
            },
            {
                id: "test-transform",
                label: tActions("testTransform"),
                icon: TestTube,
                action: () => {
                    // TODO: Open test panel
                    console.log("Test Transform");
                },
                category: "action",
                shortcut: "⌘T",
            },
        ],
        [locale, router, t, tActions]
    );

    const filteredCommands = React.useMemo(() => {
        if (!search) return commands;
        const lower = search.toLowerCase();
        return commands.filter(
            (cmd) =>
                cmd.label.toLowerCase().includes(lower) ||
                cmd.id.toLowerCase().includes(lower)
        );
    }, [commands, search]);

    // Keyboard shortcut to open
    React.useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                setOpen((open) => !open);
            }
        };

        document.addEventListener("keydown", down);
        return () => document.removeEventListener("keydown", down);
    }, []);

    // Keyboard navigation
    React.useEffect(() => {
        if (!open) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            switch (e.key) {
                case "ArrowDown":
                    e.preventDefault();
                    setSelectedIndex((i) =>
                        i < filteredCommands.length - 1 ? i + 1 : 0
                    );
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    setSelectedIndex((i) =>
                        i > 0 ? i - 1 : filteredCommands.length - 1
                    );
                    break;
                case "Enter":
                    e.preventDefault();
                    if (filteredCommands[selectedIndex]) {
                        filteredCommands[selectedIndex].action();
                        setOpen(false);
                        setSearch("");
                    }
                    break;
                case "Escape":
                    setOpen(false);
                    setSearch("");
                    break;
            }
        };

        document.addEventListener("keydown", handleKeyDown);
        return () => document.removeEventListener("keydown", handleKeyDown);
    }, [open, filteredCommands, selectedIndex]);

    // Reset selection when search changes
    React.useEffect(() => {
        setSelectedIndex(0);
    }, [search]);

    const categories = {
        navigation: "Navigation",
        action: "Actions",
        settings: "Settings",
    };

    const groupedCommands = React.useMemo(() => {
        const groups: Record<string, CommandItem[]> = {};
        filteredCommands.forEach((cmd) => {
            if (!groups[cmd.category]) {
                groups[cmd.category] = [];
            }
            groups[cmd.category].push(cmd);
        });
        return groups;
    }, [filteredCommands]);

    let flatIndex = 0;

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent className="max-w-2xl p-0 gap-0">
                <DialogHeader className="px-4 py-3 border-b">
                    <div className="flex items-center gap-2">
                        <Search className="h-5 w-5 text-muted-foreground" />
                        <Input
                            placeholder="Search commands..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            className="border-0 focus-visible:ring-0 text-base"
                            autoFocus
                        />
                    </div>
                </DialogHeader>
                <div className="max-h-96 overflow-y-auto py-2">
                    {Object.entries(groupedCommands).map(([category, items]) => (
                        <div key={category}>
                            <div className="px-4 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                                {categories[category as keyof typeof categories]}
                            </div>
                            {items.map((cmd) => {
                                const currentIndex = flatIndex++;
                                const Icon = cmd.icon;
                                return (
                                    <button
                                        key={cmd.id}
                                        onClick={() => {
                                            cmd.action();
                                            setOpen(false);
                                            setSearch("");
                                        }}
                                        className={cn(
                                            "w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors",
                                            currentIndex === selectedIndex
                                                ? "bg-accent text-accent-foreground"
                                                : "hover:bg-accent/50"
                                        )}
                                    >
                                        <Icon className="h-4 w-4 text-muted-foreground" />
                                        <span className="flex-1 text-start">{cmd.label}</span>
                                        {cmd.shortcut && (
                                            <kbd className="px-2 py-0.5 bg-muted rounded text-xs font-mono">
                                                {cmd.shortcut}
                                            </kbd>
                                        )}
                                    </button>
                                );
                            })}
                        </div>
                    ))}
                    {filteredCommands.length === 0 && (
                        <div className="px-4 py-8 text-center text-muted-foreground">
                            No commands found
                        </div>
                    )}
                </div>
            </DialogContent>
        </Dialog>
    );
}
