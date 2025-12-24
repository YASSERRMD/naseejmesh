import { create } from "zustand";
import {
    Node,
    Edge,
    OnNodesChange,
    OnEdgesChange,
    OnConnect,
    applyNodeChanges,
    applyEdgeChanges,
    addEdge,
    Connection,
} from "@xyflow/react";
import { getLayoutedElements } from "@/lib/flow-layout";

// Service types for the mesh
export type ServiceType =
    | "mqtt"
    | "http"
    | "database"
    | "filter"
    | "transform"
    | "gateway"
    | "ai"         // AI/LLM node (Cohere)
    | "mcp"        // Model Context Protocol
    | "splitter"   // Split flow into parallel branches
    | "aggregator" // Aggregate multiple inputs
    | "logic";     // If/Else conditional logic

export type ServiceStatus = "healthy" | "warning" | "error" | "offline";

export interface ServiceNodeData {
    label: string;
    serviceType: ServiceType;
    status: ServiceStatus;
    address?: string;
    topic?: string;
    requestsPerSec?: number;
    description?: string;
    // AI Node specific
    model?: string;         // e.g., 'command-r-plus', 'gpt-4'
    prompt?: string;        // System prompt
    // MCP Node specific
    mcpServerUrl?: string;  // MCP server endpoint
    mcpTool?: string;       // Selected tool from server
    // Logic Node specific
    condition?: string;     // e.g., 'input.value > 100'
    // Index signature for React Flow compatibility
    [key: string]: unknown;
}

export type ServiceNode = Node<ServiceNodeData, "service">;

export interface MeshEdgeData {
    animated?: boolean;
    label?: string;
    // Index signature for React Flow compatibility
    [key: string]: unknown;
}

export type MeshEdge = Edge<MeshEdgeData>;

interface MeshState {
    nodes: ServiceNode[];
    edges: MeshEdge[];
    selectedNodeId: string | null;
    onNodesChange: OnNodesChange<ServiceNode>;
    onEdgesChange: OnEdgesChange<MeshEdge>;
    onConnect: OnConnect;
    setSelectedNode: (id: string | null) => void;
    addNode: (node: ServiceNode) => void;
    removeNode: (id: string) => void;
    updateNode: (id: string, data: Partial<ServiceNodeData>) => void;
    addEdgeConnection: (edge: MeshEdge) => void;
    removeEdge: (id: string) => void;
    layout: () => void;
    reset: () => void;
}

// Initial demo state with 3 connected nodes
const initialNodes: ServiceNode[] = [
    {
        id: "mqtt-source",
        type: "service",
        position: { x: 0, y: 0 },
        data: {
            label: "MQTT Broker",
            serviceType: "mqtt",
            status: "healthy",
            address: "mqtt://broker.naseej.io:1883",
            topic: "sensors/#",
            requestsPerSec: 142,
        },
    },
    {
        id: "filter-node",
        type: "service",
        position: { x: 250, y: 0 },
        data: {
            label: "Data Filter",
            serviceType: "filter",
            status: "healthy",
            description: "Filter by device_id",
            requestsPerSec: 89,
        },
    },
    {
        id: "transform-node",
        type: "service",
        position: { x: 500, y: 0 },
        data: {
            label: "JSON Transform",
            serviceType: "transform",
            status: "healthy",
            description: "Rhai script",
            requestsPerSec: 89,
        },
    },
    {
        id: "postgres-sink",
        type: "service",
        position: { x: 750, y: 0 },
        data: {
            label: "PostgreSQL",
            serviceType: "database",
            status: "healthy",
            address: "postgres://db.naseej.io:5432/iot",
            requestsPerSec: 67,
        },
    },
    {
        id: "http-api",
        type: "service",
        position: { x: 500, y: 150 },
        data: {
            label: "REST API",
            serviceType: "http",
            status: "warning",
            address: "https://api.naseej.io/v1",
            requestsPerSec: 234,
        },
    },
];

const initialEdges: MeshEdge[] = [
    {
        id: "e-mqtt-filter",
        source: "mqtt-source",
        target: "filter-node",
        animated: true,
        type: "smoothstep",
    },
    {
        id: "e-filter-transform",
        source: "filter-node",
        target: "transform-node",
        animated: true,
        type: "smoothstep",
    },
    {
        id: "e-transform-postgres",
        source: "transform-node",
        target: "postgres-sink",
        animated: true,
        type: "smoothstep",
    },
    {
        id: "e-transform-http",
        source: "transform-node",
        target: "http-api",
        animated: false,
        type: "smoothstep",
    },
];

export const useMeshStore = create<MeshState>((set, get) => ({
    nodes: initialNodes,
    edges: initialEdges,
    selectedNodeId: null,

    onNodesChange: (changes) => {
        set({
            nodes: applyNodeChanges(changes, get().nodes),
        });
    },

    onEdgesChange: (changes) => {
        set({
            edges: applyEdgeChanges(changes, get().edges),
        });
    },

    onConnect: (connection: Connection) => {
        // Validate connection - prevent connecting output to output
        const sourceNode = get().nodes.find((n) => n.id === connection.source);
        const targetNode = get().nodes.find((n) => n.id === connection.target);

        if (!sourceNode || !targetNode) return;

        // Don't allow self-connections
        if (connection.source === connection.target) return;

        // Check if edge already exists
        const existingEdge = get().edges.find(
            (e) => e.source === connection.source && e.target === connection.target
        );
        if (existingEdge) return;

        const newEdge: MeshEdge = {
            ...connection,
            id: `e-${connection.source}-${connection.target}`,
            animated: true,
            type: "smoothstep",
        };

        set({
            edges: addEdge(newEdge, get().edges),
        });
    },

    setSelectedNode: (id) => {
        set({ selectedNodeId: id });
    },

    addNode: (node) => {
        set({
            nodes: [...get().nodes, node],
        });
    },

    removeNode: (id) => {
        set({
            nodes: get().nodes.filter((n) => n.id !== id),
            edges: get().edges.filter((e) => e.source !== id && e.target !== id),
        });
    },

    updateNode: (id, data) => {
        set({
            nodes: get().nodes.map((node) =>
                node.id === id ? { ...node, data: { ...node.data, ...data } } : node
            ),
        });
    },

    addEdgeConnection: (edge) => {
        set({
            edges: [...get().edges, edge],
        });
    },

    removeEdge: (id) => {
        set({
            edges: get().edges.filter((e) => e.id !== id),
        });
    },

    layout: () => {
        const { nodes, edges } = get();
        const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
            nodes,
            edges
        );
        set({
            nodes: layoutedNodes,
            edges: layoutedEdges,
        });
    },

    reset: () => {
        set({
            nodes: initialNodes,
            edges: initialEdges,
            selectedNodeId: null,
        });
    },
}));
