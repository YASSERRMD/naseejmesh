import type {
    RouteConfig,
    GatewayStatus,
    SecurityEvent,
    TransformationScript,
    ApiSchema,
    RequestMetrics,
} from "./api-types";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

class ApiError extends Error {
    constructor(public status: number, message: string) {
        super(message);
        this.name = "ApiError";
    }
}

async function fetchApi<T>(
    endpoint: string,
    options?: RequestInit
): Promise<T> {
    const url = `${API_BASE}${endpoint}`;

    const response = await fetch(url, {
        ...options,
        headers: {
            "Content-Type": "application/json",
            ...options?.headers,
        },
    });

    if (!response.ok) {
        throw new ApiError(response.status, await response.text());
    }

    return response.json();
}

// Gateway Status
export async function getGatewayStatus(): Promise<GatewayStatus> {
    return fetchApi<GatewayStatus>("/api/status");
}

// Routes
export async function getRoutes(): Promise<RouteConfig[]> {
    return fetchApi<RouteConfig[]>("/api/routes");
}

export async function createRoute(route: Partial<RouteConfig>): Promise<RouteConfig> {
    return fetchApi<RouteConfig>("/api/routes", {
        method: "POST",
        body: JSON.stringify(route),
    });
}

export async function updateRoute(id: string, route: Partial<RouteConfig>): Promise<RouteConfig> {
    return fetchApi<RouteConfig>(`/api/routes/${id}`, {
        method: "PUT",
        body: JSON.stringify(route),
    });
}

export async function deleteRoute(id: string): Promise<void> {
    await fetchApi(`/api/routes/${id}`, { method: "DELETE" });
}

// Transformations
export async function getTransformations(): Promise<TransformationScript[]> {
    return fetchApi<TransformationScript[]>("/api/transformations");
}

// Security Events
export async function getSecurityEvents(limit = 50): Promise<SecurityEvent[]> {
    return fetchApi<SecurityEvent[]>(`/api/security/events?limit=${limit}`);
}

// Schemas
export async function getSchemas(): Promise<ApiSchema[]> {
    return fetchApi<ApiSchema[]>("/api/schemas");
}

export async function createSchema(schema: Partial<ApiSchema>): Promise<ApiSchema> {
    return fetchApi<ApiSchema>("/api/schemas", {
        method: "POST",
        body: JSON.stringify(schema),
    });
}

// Metrics
export async function getMetrics(): Promise<RequestMetrics> {
    return fetchApi<RequestMetrics>("/api/metrics");
}


export { ApiError };
