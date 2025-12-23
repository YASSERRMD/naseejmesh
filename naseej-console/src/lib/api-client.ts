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
    try {
        return await fetchApi<GatewayStatus>("/api/status");
    } catch {
        // Return mock data if API is not available
        return {
            healthy: true,
            version: "0.1.0",
            uptime: 9240,
            routes: 12,
            requests: {
                total: 45892,
                perSecond: 127,
                avgLatencyMs: 45,
                errorRate: 0.02,
            },
        };
    }
}

// Routes
export async function getRoutes(): Promise<RouteConfig[]> {
    try {
        return await fetchApi<RouteConfig[]>("/api/routes");
    } catch {
        // Return mock data
        return [
            {
                id: "1",
                name: "User API",
                path: "/api/users",
                methods: ["GET", "POST"],
                upstream: { url: "http://users-service:8080" },
                enabled: true,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
            {
                id: "2",
                name: "Orders API",
                path: "/api/orders",
                methods: ["GET", "POST", "PUT"],
                upstream: { url: "http://orders-service:8080" },
                enabled: true,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
            {
                id: "3",
                name: "Products API",
                path: "/api/products",
                methods: ["GET"],
                upstream: { url: "http://products-service:8080" },
                enabled: false,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
        ];
    }
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
    try {
        return await fetchApi<TransformationScript[]>("/api/transformations");
    } catch {
        return [
            {
                id: "1",
                name: "Celsius to Fahrenheit",
                description: "Converts temperature values",
                language: "rhai",
                script: 'let temp = input.temperature;\noutput.fahrenheit = (temp * 9/5) + 32;',
                inputType: "json",
                outputType: "json",
                usedBy: ["route-1", "route-2"],
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
            {
                id: "2",
                name: "XML to JSON",
                description: "Converts XML to JSON format",
                language: "rhai",
                script: 'output = xml_to_json(input);',
                inputType: "xml",
                outputType: "json",
                usedBy: ["route-3"],
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
        ];
    }
}

// Security Events
export async function getSecurityEvents(limit = 50): Promise<SecurityEvent[]> {
    try {
        return await fetchApi<SecurityEvent[]>(`/api/security/events?limit=${limit}`);
    } catch {
        return [
            {
                id: "1",
                type: "blocked",
                category: "waf",
                message: "SQL Injection attempt detected",
                source: "192.168.1.45",
                timestamp: new Date(Date.now() - 120000).toISOString(),
            },
            {
                id: "2",
                type: "warning",
                category: "rate_limit",
                message: "Rate limit exceeded",
                source: "10.0.0.23",
                timestamp: new Date(Date.now() - 300000).toISOString(),
            },
            {
                id: "3",
                type: "blocked",
                category: "waf",
                message: "XSS payload in request body",
                source: "192.168.1.89",
                timestamp: new Date(Date.now() - 600000).toISOString(),
            },
        ];
    }
}

// Schemas
export async function getSchemas(): Promise<ApiSchema[]> {
    try {
        return await fetchApi<ApiSchema[]>("/api/schemas");
    } catch {
        return [
            {
                id: "1",
                name: "User Service API",
                type: "openapi",
                version: "3.0.1",
                content: "",
                endpoints: 12,
                status: "valid",
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
            {
                id: "2",
                name: "Orders Schema",
                type: "jsonschema",
                version: "draft-07",
                content: "",
                endpoints: 5,
                status: "valid",
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
            },
        ];
    }
}

// Metrics
export async function getMetrics(): Promise<RequestMetrics> {
    try {
        return await fetchApi<RequestMetrics>("/api/metrics");
    } catch {
        return {
            total: 45892,
            perSecond: 127,
            avgLatencyMs: 45,
            errorRate: 0.02,
        };
    }
}

export { ApiError };
