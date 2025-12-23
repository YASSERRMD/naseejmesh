// API types matching the Rust backend

export interface RouteConfig {
    id: string;
    name: string;
    path: string;
    methods: HttpMethod[];
    upstream: UpstreamConfig;
    transformations?: TransformationRef[];
    auth?: AuthConfig;
    rateLimit?: RateLimitConfig;
    cache?: CacheConfig;
    enabled: boolean;
    createdAt: string;
    updatedAt: string;
}

export type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

export interface UpstreamConfig {
    url: string;
    timeout?: number;
    retries?: number;
    loadBalancer?: "round_robin" | "random" | "least_conn";
    healthCheck?: HealthCheckConfig;
}

export interface HealthCheckConfig {
    path: string;
    interval: number;
    timeout: number;
    healthyThreshold: number;
    unhealthyThreshold: number;
}

export interface TransformationRef {
    id: string;
    name: string;
    type: "request" | "response";
}

export interface AuthConfig {
    type: "jwt" | "api_key" | "basic" | "none";
    required: boolean;
    config?: Record<string, unknown>;
}

export interface RateLimitConfig {
    requestsPerMinute: number;
    burstSize: number;
    keyBy: "ip" | "api_key" | "user";
}

export interface CacheConfig {
    enabled: boolean;
    ttl: number;
    varyBy?: string[];
}

export interface GatewayStatus {
    healthy: boolean;
    version: string;
    uptime: number;
    routes: number;
    requests: RequestMetrics;
}

export interface RequestMetrics {
    total: number;
    perSecond: number;
    avgLatencyMs: number;
    errorRate: number;
}

export interface SecurityEvent {
    id: string;
    type: "blocked" | "warning" | "allowed";
    category: "waf" | "rate_limit" | "auth";
    message: string;
    source: string;
    details?: Record<string, unknown>;
    timestamp: string;
}

export interface TransformationScript {
    id: string;
    name: string;
    description: string;
    language: "rhai";
    script: string;
    inputType: "json" | "xml" | "text";
    outputType: "json" | "xml" | "text";
    usedBy: string[];
    createdAt: string;
    updatedAt: string;
}

export interface ApiSchema {
    id: string;
    name: string;
    type: "openapi" | "jsonschema" | "graphql";
    version: string;
    content: string;
    endpoints: number;
    status: "valid" | "warning" | "error";
    errors?: string[];
    createdAt: string;
    updatedAt: string;
}
