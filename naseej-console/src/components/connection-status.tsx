"use client";

import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { Wifi, WifiOff, Loader2 } from "lucide-react";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

type ConnectionState = "connecting" | "connected" | "disconnected";

interface ConnectionStatusProps {
    className?: string;
}

export function ConnectionStatus({ className }: ConnectionStatusProps) {
    const [status, setStatus] = useState<ConnectionState>("connecting");
    const [version, setVersion] = useState<string | null>(null);
    const [retryCount, setRetryCount] = useState(0);

    useEffect(() => {
        let isMounted = true;
        let timeoutId: NodeJS.Timeout;

        const checkConnection = async () => {
            try {
                const response = await fetch(`${API_BASE}/api/status`, {
                    method: "GET",
                    headers: { "Content-Type": "application/json" },
                    signal: AbortSignal.timeout(5000),
                });

                if (!isMounted) return;

                if (response.ok) {
                    const data = await response.json();
                    setStatus("connected");
                    setVersion(data.version || null);
                    setRetryCount(0);
                } else {
                    throw new Error("Backend returned error");
                }
            } catch {
                if (!isMounted) return;
                setStatus("disconnected");
                setVersion(null);
                setRetryCount((prev) => prev + 1);
            }

            // Schedule next check
            const interval = status === "connected" ? 10000 : 3000;
            timeoutId = setTimeout(checkConnection, interval);
        };

        checkConnection();

        return () => {
            isMounted = false;
            clearTimeout(timeoutId);
        };
    }, [status]);

    const statusConfig = {
        connecting: {
            icon: Loader2,
            color: "text-yellow-500",
            bgColor: "bg-yellow-500/10",
            label: "Connecting...",
            animate: true,
        },
        connected: {
            icon: Wifi,
            color: "text-green-500",
            bgColor: "bg-green-500/10",
            label: `Connected${version ? ` (v${version})` : ""}`,
            animate: false,
        },
        disconnected: {
            icon: WifiOff,
            color: "text-destructive",
            bgColor: "bg-destructive/10",
            label: `Disconnected${retryCount > 0 ? ` (retry ${retryCount})` : ""}`,
            animate: false,
        },
    };

    const config = statusConfig[status];
    const Icon = config.icon;

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <div
                        className={cn(
                            "flex items-center gap-2 px-2 py-1 rounded-md cursor-default transition-colors",
                            config.bgColor,
                            className
                        )}
                    >
                        <Icon
                            className={cn(
                                "h-4 w-4",
                                config.color,
                                config.animate && "animate-spin"
                            )}
                        />
                        <span className={cn("text-xs font-medium", config.color)}>
                            {status === "connected" ? "Backend" : config.label}
                        </span>
                    </div>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                    <div className="text-xs space-y-1">
                        <p className="font-medium">{config.label}</p>
                        <p className="text-muted-foreground">{API_BASE}</p>
                        {status === "disconnected" && (
                            <p className="text-muted-foreground">
                                Start backend: cargo run -p naseej-console
                            </p>
                        )}
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}
