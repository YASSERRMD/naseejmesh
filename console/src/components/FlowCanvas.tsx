'use client';

import { useCallback, useState } from 'react';
import ReactFlow, {
    MiniMap,
    Controls,
    Background,
    useNodesState,
    useEdgesState,
    addEdge,
    BackgroundVariant,
    Node,
    Edge,
    Connection,
    NodeTypes,
} from 'reactflow';
import 'reactflow/dist/style.css';
import ProtocolNode from './ProtocolNode';
import TransformNode from './TransformNode';

// Define custom node types
const nodeTypes: NodeTypes = {
    protocol: ProtocolNode,
    transform: TransformNode,
};

// Initial demo nodes
const initialNodes: Node[] = [
    {
        id: 'mqtt-1',
        type: 'protocol',
        position: { x: 100, y: 100 },
        data: {
            label: 'MQTT Sensors',
            protocol: 'mqtt',
            config: { topic: 'sensors/+/temp' },
        },
    },
    {
        id: 'transform-1',
        type: 'transform',
        position: { x: 350, y: 100 },
        data: {
            label: 'Celsius → Fahrenheit',
            script: 'let data = parse_json(input);\ndata["temp_f"] = celsius_to_fahrenheit(data["temp"]);\noutput = to_json(data);',
        },
    },
    {
        id: 'http-1',
        type: 'protocol',
        position: { x: 600, y: 100 },
        data: {
            label: 'REST API',
            protocol: 'http',
            config: { upstream: 'http://api.example.com/readings' },
        },
    },
    {
        id: 'grpc-1',
        type: 'protocol',
        position: { x: 100, y: 300 },
        data: {
            label: 'gRPC Service',
            protocol: 'grpc',
            config: { service: 'user.UserService' },
        },
    },
    {
        id: 'http-2',
        type: 'protocol',
        position: { x: 350, y: 300 },
        data: {
            label: 'JSON Gateway',
            protocol: 'http',
            config: { path: '/api/users' },
        },
    },
];

const initialEdges: Edge[] = [
    { id: 'e1-2', source: 'mqtt-1', target: 'transform-1', animated: true },
    { id: 'e2-3', source: 'transform-1', target: 'http-1', animated: true },
    { id: 'e4-5', source: 'grpc-1', target: 'http-2' },
];

export default function FlowCanvas() {
    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge({ ...params, animated: true }, eds)),
        [setEdges]
    );

    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            fitView
            style={{ background: '#141517' }}
        >
            <Controls
                style={{
                    background: '#25262B',
                    borderRadius: 8,
                    border: '1px solid #373A40',
                }}
            />
            <MiniMap
                style={{
                    background: '#25262B',
                    borderRadius: 8,
                    border: '1px solid #373A40',
                }}
                nodeColor={(node) => {
                    const protocol = node.data?.protocol;
                    switch (protocol) {
                        case 'http': return '#228be6';
                        case 'mqtt': return '#40c057';
                        case 'grpc': return '#be4bdb';
                        case 'soap': return '#fd7e14';
                        default: return '#fab005';
                    }
                }}
            />
            <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#373A40" />
        </ReactFlow>
    );
}
