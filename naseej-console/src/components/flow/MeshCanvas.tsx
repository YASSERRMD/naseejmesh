"use client";

import { useCallback, useEffect } from "react";
import {
    ReactFlow,
    Background,
    Controls,
    MiniMap,
    BackgroundVariant,
    useReactFlow,
    ReactFlowProvider,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { cn } from "@/lib/utils";
import { useMeshStore } from "@/stores/mesh-store";
import { ServiceNode } from "./ServiceNode";

// Register custom node types
const nodeTypes = {
    service: ServiceNode,
};

// Default edge options
const defaultEdgeOptions = {
    type: "smoothstep",
    animated: true,
    style: {
        strokeWidth: 2,
    },
};

interface MeshCanvasProps {
    locale?: string;
}

function MeshCanvasInner({ locale = "en" }: MeshCanvasProps) {
    const isRTL = locale === "ar";
    const { fitView } = useReactFlow();

    const {
        nodes,
        edges,
        onNodesChange,
        onEdgesChange,
        onConnect,
        setSelectedNode,
        layout,
    } = useMeshStore();

    // Auto-layout on first render
    useEffect(() => {
        layout();
        // Fit view after layout
        setTimeout(() => fitView({ padding: 0.2 }), 100);
    }, []);

    const handleNodeClick = useCallback(
        (_: React.MouseEvent, node: { id: string }) => {
            setSelectedNode(node.id);
        },
        [setSelectedNode]
    );

    const handlePaneClick = useCallback(() => {
        setSelectedNode(null);
    }, [setSelectedNode]);

    return (
        <div className="w-full h-full bg-background">
            <ReactFlow
                nodes={nodes as any}
                edges={edges as any}
                onNodesChange={onNodesChange as any}
                onEdgesChange={onEdgesChange as any}
                onConnect={onConnect}
                onNodeClick={handleNodeClick}
                onPaneClick={handlePaneClick}
                nodeTypes={nodeTypes}
                defaultEdgeOptions={defaultEdgeOptions}
                fitView
                fitViewOptions={{ padding: 0.2 }}
                minZoom={0.1}
                maxZoom={2}
                snapToGrid
                snapGrid={[20, 20]}
                proOptions={{ hideAttribution: true }}
                className="mesh-canvas"
            >
                {/* Dotted background */}
                <Background
                    variant={BackgroundVariant.Dots}
                    gap={20}
                    size={1}
                    className="opacity-30"
                />

                {/* Controls */}
                <Controls
                    position={isRTL ? "bottom-left" : "bottom-right"}
                    className="!bg-card !border !border-border !rounded-lg !shadow-lg"
                />

                {/* MiniMap - RTL aware positioning */}
                <MiniMap
                    position={isRTL ? "bottom-left" : "bottom-right"}
                    className={cn(
                        "!bg-card !border !border-border !rounded-lg !shadow-lg",
                        isRTL ? "!left-4 !right-auto" : "!right-4 !left-auto",
                        "!bottom-32"
                    )}
                    nodeColor={(node) => {
                        const data = node.data as { status?: string };
                        switch (data?.status) {
                            case "healthy":
                                return "#22c55e";
                            case "warning":
                                return "#eab308";
                            case "error":
                                return "#ef4444";
                            default:
                                return "#6b7280";
                        }
                    }}
                    maskColor="rgba(0, 0, 0, 0.3)"
                />
            </ReactFlow>
        </div>
    );
}

export function MeshCanvas(props: MeshCanvasProps) {
    return <MeshCanvasInner {...props} />;
}

export function MeshCanvasWithProvider(props: MeshCanvasProps) {
    return (
        <ReactFlowProvider>
            <MeshCanvasInner {...props} />
        </ReactFlowProvider>
    );
}
