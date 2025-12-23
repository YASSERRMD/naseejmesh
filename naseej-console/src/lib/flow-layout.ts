import dagre from "dagre";
import { Node, Edge } from "@xyflow/react";

const NODE_WIDTH = 280;
const NODE_HEIGHT = 120;
const HORIZONTAL_SPACING = 100;
const VERTICAL_SPACING = 50;

export function getLayoutedElements<T extends Node, E extends Edge>(
    nodes: T[],
    edges: E[],
    direction: "LR" | "TB" = "LR"
): { nodes: T[]; edges: E[] } {
    const dagreGraph = new dagre.graphlib.Graph();
    dagreGraph.setDefaultEdgeLabel(() => ({}));

    const isHorizontal = direction === "LR";
    dagreGraph.setGraph({
        rankdir: direction,
        nodesep: isHorizontal ? VERTICAL_SPACING : HORIZONTAL_SPACING,
        ranksep: isHorizontal ? HORIZONTAL_SPACING : VERTICAL_SPACING,
        marginx: 50,
        marginy: 50,
    });

    // Add nodes to dagre
    nodes.forEach((node) => {
        dagreGraph.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
    });

    // Add edges to dagre
    edges.forEach((edge) => {
        dagreGraph.setEdge(edge.source, edge.target);
    });

    // Calculate layout
    dagre.layout(dagreGraph);

    // Apply positions to nodes
    const layoutedNodes = nodes.map((node) => {
        const nodeWithPosition = dagreGraph.node(node.id);
        return {
            ...node,
            position: {
                x: nodeWithPosition.x - NODE_WIDTH / 2,
                y: nodeWithPosition.y - NODE_HEIGHT / 2,
            },
        };
    });

    return { nodes: layoutedNodes, edges };
}

// Generate a unique ID for new nodes
export function generateNodeId(prefix = "node"): string {
    return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
