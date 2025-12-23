"use client";

import { memo } from "react";
import { Handle, Position, NodeProps } from "@xyflow/react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import {
    Radio,
    Globe,
    Database,
    Filter,
    Workflow,
    Shield,
    Activity,
} from "lucide-react";
import type { ServiceNodeData, ServiceType, ServiceStatus } from "@/stores/mesh-store";

// Icon mapping for service types
const serviceIcons: Record<ServiceType, React.ElementType> = {
    mqtt: Radio,
    http: Globe,
    database: Database,
    filter: Filter,
    transform: Workflow,
    gateway: Shield,
};

// Color mapping for service types
const serviceColors: Record<ServiceType, string> = {
    mqtt: "from-purple-500/20 to-purple-600/10 border-purple-500/30",
    http: "from-blue-500/20 to-blue-600/10 border-blue-500/30",
    database: "from-green-500/20 to-green-600/10 border-green-500/30",
    filter: "from-yellow-500/20 to-yellow-600/10 border-yellow-500/30",
    transform: "from-orange-500/20 to-orange-600/10 border-orange-500/30",
    gateway: "from-cyan-500/20 to-cyan-600/10 border-cyan-500/30",
};

// Status indicator colors
const statusColors: Record<ServiceStatus, string> = {
    healthy: "bg-green-500",
    warning: "bg-yellow-500",
    error: "bg-red-500",
    offline: "bg-gray-500",
};

interface ServiceNodeProps extends NodeProps<ServiceNodeData> {
    selected?: boolean;
}

function ServiceNodeComponent({ data, selected }: ServiceNodeProps) {
    const Icon = serviceIcons[data.serviceType] || Globe;
    const colorClass = serviceColors[data.serviceType] || serviceColors.http;
    const statusClass = statusColors[data.status];

    return (
        <div
            className={cn(
                "relative w-[280px] rounded-lg border bg-gradient-to-br shadow-lg transition-all duration-200",
                colorClass,
                selected && "ring-2 ring-primary ring-offset-2 ring-offset-background",
                "hover:shadow-xl hover:border-primary/50"
            )}
        >
            {/* Input Handle - Left Port */}
            <Handle
                type="target"
                position={Position.Left}
                className={cn(
                    "!w-4 !h-4 !bg-background !border-2 !border-primary",
                    "!rounded-full !-left-2",
                    "hover:!bg-primary hover:!scale-110 transition-all"
                )}
            />

            {/* Header */}
            <div className="flex items-center gap-3 px-4 py-3 border-b border-border/50">
                <div className={cn(
                    "h-10 w-10 rounded-lg flex items-center justify-center",
                    "bg-background/80 backdrop-blur-sm"
                )}>
                    <Icon className="h-5 w-5 text-foreground" />
                </div>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                        <span className="font-semibold text-sm truncate">{data.label}</span>
                        <div className={cn("h-2 w-2 rounded-full animate-pulse", statusClass)} />
                    </div>
                    <span className="text-xs text-muted-foreground uppercase tracking-wider">
                        {data.serviceType}
                    </span>
                </div>
            </div>

            {/* Body */}
            <div className="px-4 py-3 space-y-2">
                {data.address && (
                    <div className="text-xs">
                        <span className="text-muted-foreground">Address: </span>
                        <code className="bg-background/50 px-1.5 py-0.5 rounded text-[10px]">
                            {data.address}
                        </code>
                    </div>
                )}
                {data.topic && (
                    <div className="text-xs">
                        <span className="text-muted-foreground">Topic: </span>
                        <code className="bg-background/50 px-1.5 py-0.5 rounded text-[10px]">
                            {data.topic}
                        </code>
                    </div>
                )}
                {data.description && (
                    <div className="text-xs text-muted-foreground">
                        {data.description}
                    </div>
                )}
            </div>

            {/* Footer */}
            <div className="px-4 py-2 border-t border-border/50 flex items-center justify-between">
                {data.requestsPerSec !== undefined ? (
                    <Badge variant="secondary" className="text-[10px] h-5">
                        <Activity className="h-3 w-3 me-1" />
                        {data.requestsPerSec} req/s
                    </Badge>
                ) : (
                    <div />
                )}
                <Badge
                    variant={data.status === "healthy" ? "success" : data.status === "warning" ? "warning" : "destructive"}
                    className="text-[10px] h-5"
                >
                    {data.status}
                </Badge>
            </div>

            {/* Output Handle - Right Port */}
            <Handle
                type="source"
                position={Position.Right}
                className={cn(
                    "!w-4 !h-4 !bg-background !border-2 !border-primary",
                    "!rounded-full !-right-2",
                    "hover:!bg-primary hover:!scale-110 transition-all"
                )}
            />
        </div>
    );
}

export const ServiceNode = memo(ServiceNodeComponent);
