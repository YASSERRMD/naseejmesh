"use client";

import { use } from "react";
import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { MeshCanvasWithProvider } from "@/components/flow/MeshCanvas";
import { useSidebarStore } from "@/stores/ui-store";
import { useMeshStore, type ServiceType } from "@/stores/mesh-store";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import {
    Plus,
    Layout,
    Save,
    Radio,
    Globe,
    Database,
    Filter,
    Workflow,
    Shield,
    RotateCcw,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { generateNodeId } from "@/lib/flow-layout";

interface DesignPageProps {
    params: Promise<{ locale: string }>;
}

const serviceOptions: { type: ServiceType; label: string; icon: React.ElementType }[] = [
    { type: "mqtt", label: "MQTT Broker", icon: Radio },
    { type: "http", label: "HTTP/REST API", icon: Globe },
    { type: "database", label: "Database", icon: Database },
    { type: "filter", label: "Filter", icon: Filter },
    { type: "transform", label: "Transform", icon: Workflow },
    { type: "gateway", label: "Gateway", icon: Shield },
];

export default function DesignPage({ params }: DesignPageProps) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";

    const { nodes, edges, addNode, layout, reset } = useMeshStore();

    const handleAddService = (type: ServiceType) => {
        const option = serviceOptions.find((o) => o.type === type);
        const id = generateNodeId(type);

        const newNode = {
            id,
            type: "service" as const,
            position: { x: Math.random() * 400, y: Math.random() * 300 },
            data: {
                label: option?.label || "New Service",
                serviceType: type,
                status: "healthy" as const,
                description: "New service node",
                requestsPerSec: 0,
            },
        };

        addNode(newNode);
        toast.success(`Added ${option?.label || type} node`);
    };

    const handleAutoLayout = () => {
        layout();
        toast.success("Auto-layout applied");
    };

    const handleSave = () => {
        const config = {
            nodes: nodes.map((n) => ({
                id: n.id,
                type: n.data.serviceType,
                data: n.data,
            })),
            edges: edges.map((e) => ({
                source: e.source,
                target: e.target,
            })),
        };
        console.log("Mesh Configuration:", JSON.stringify(config, null, 2));
        toast.success("Configuration saved to console");
    };

    const handleReset = () => {
        reset();
        toast.info("Canvas reset to initial state");
    };

    return (
        <div className="min-h-screen h-screen bg-background flex flex-col overflow-hidden">
            <div className="flex flex-1 overflow-hidden">
                <Sidebar locale={locale} />
                <CommandPalette locale={locale} />
                <div
                    className={cn(
                        "flex-1 flex flex-col transition-all duration-300 overflow-hidden",
                        isCollapsed ? (isRTL ? "me-16" : "ms-16") : (isRTL ? "me-64" : "ms-64")
                    )}
                >
                    <Header locale={locale} />

                    {/* Main Canvas Area */}
                    <div className="flex-1 relative overflow-hidden">
                        {/* Floating Toolbar */}
                        <div className={cn(
                            "absolute top-4 z-10 flex items-center gap-2 bg-card/95 backdrop-blur-sm rounded-lg border shadow-lg p-2",
                            isRTL ? "right-4" : "left-4"
                        )}>
                            {/* Add Service Dropdown */}
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button size="sm">
                                        <Plus className="h-4 w-4 me-2" />
                                        Add Service
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align={isRTL ? "end" : "start"}>
                                    {serviceOptions.map((option) => {
                                        const Icon = option.icon;
                                        return (
                                            <DropdownMenuItem
                                                key={option.type}
                                                onClick={() => handleAddService(option.type)}
                                            >
                                                <Icon className="h-4 w-4 me-2" />
                                                {option.label}
                                            </DropdownMenuItem>
                                        );
                                    })}
                                </DropdownMenuContent>
                            </DropdownMenu>

                            <div className="w-px h-6 bg-border" />

                            <Button variant="outline" size="sm" onClick={handleAutoLayout}>
                                <Layout className="h-4 w-4 me-2" />
                                Auto Layout
                            </Button>

                            <Button variant="outline" size="sm" onClick={handleSave}>
                                <Save className="h-4 w-4 me-2" />
                                Save
                            </Button>

                            <div className="w-px h-6 bg-border" />

                            <Button variant="ghost" size="sm" onClick={handleReset}>
                                <RotateCcw className="h-4 w-4 me-2" />
                                Reset
                            </Button>
                        </div>

                        {/* Stats Badge */}
                        <div className={cn(
                            "absolute top-4 z-10 flex items-center gap-2",
                            isRTL ? "left-4" : "right-4"
                        )}>
                            <Badge variant="outline" className="bg-card/95 backdrop-blur-sm">
                                {nodes.length} nodes
                            </Badge>
                            <Badge variant="outline" className="bg-card/95 backdrop-blur-sm">
                                {edges.length} connections
                            </Badge>
                        </div>

                        {/* React Flow Canvas */}
                        <MeshCanvasWithProvider locale={locale} />
                    </div>
                </div>
            </div>
        </div>
    );
}
