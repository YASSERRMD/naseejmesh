"use client";

import useSWR from "swr";
import {
    getGatewayStatus,
    getRoutes,
    getTransformations,
    getSecurityEvents,
    getSchemas,
    getMetrics,
} from "./api-client";
import type {
    GatewayStatus,
    RouteConfig,
    TransformationScript,
    SecurityEvent,
    ApiSchema,
    RequestMetrics,
} from "./api-types";

// Gateway Status - refreshes every 5 seconds
export function useGatewayStatus() {
    return useSWR<GatewayStatus>("gateway-status", getGatewayStatus, {
        refreshInterval: 5000,
        revalidateOnFocus: true,
    });
}

// Routes
export function useRoutes() {
    return useSWR<RouteConfig[]>("routes", getRoutes, {
        revalidateOnFocus: true,
    });
}

// Transformations
export function useTransformations() {
    return useSWR<TransformationScript[]>("transformations", getTransformations, {
        revalidateOnFocus: true,
    });
}

// Security Events - refreshes every 10 seconds
export function useSecurityEvents(limit = 50) {
    return useSWR<SecurityEvent[]>(
        ["security-events", limit],
        () => getSecurityEvents(limit),
        {
            refreshInterval: 10000,
            revalidateOnFocus: true,
        }
    );
}

// Schemas
export function useSchemas() {
    return useSWR<ApiSchema[]>("schemas", getSchemas, {
        revalidateOnFocus: true,
    });
}

// Metrics - refreshes every 3 seconds
export function useMetrics() {
    return useSWR<RequestMetrics>("metrics", getMetrics, {
        refreshInterval: 3000,
        revalidateOnFocus: true,
    });
}
