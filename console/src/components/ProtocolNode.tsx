'use client';

import { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { Badge, Text } from '@mantine/core';

interface ProtocolNodeData {
    label: string;
    protocol: 'http' | 'mqtt' | 'grpc' | 'soap';
    config?: Record<string, string>;
}

const protocolColors = {
    http: { bg: 'linear-gradient(135deg, #228be6 0%, #1971c2 100%)', badge: 'blue' },
    mqtt: { bg: 'linear-gradient(135deg, #40c057 0%, #2f9e44 100%)', badge: 'green' },
    grpc: { bg: 'linear-gradient(135deg, #be4bdb 0%, #9c36b5 100%)', badge: 'grape' },
    soap: { bg: 'linear-gradient(135deg, #fd7e14 0%, #e8590c 100%)', badge: 'orange' },
};

const protocolIcons = {
    http: '🌐',
    mqtt: '📡',
    grpc: '⚡',
    soap: '📄',
};

function ProtocolNode({ data }: NodeProps<ProtocolNodeData>) {
    const colors = protocolColors[data.protocol] || protocolColors.http;
    const icon = protocolIcons[data.protocol] || '🔗';

    return (
        <div
            style={{
                padding: '12px 16px',
                borderRadius: 8,
                minWidth: 160,
                background: colors.bg,
                boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
            }}
        >
            <Handle type="target" position={Position.Left} style={{ background: '#fff' }} />

            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                <span style={{ fontSize: 18 }}>{icon}</span>
                <Text size="sm" fw={600} c="white">
                    {data.label}
                </Text>
            </div>

            <Badge size="xs" variant="light" color={colors.badge as any}>
                {data.protocol.toUpperCase()}
            </Badge>

            {data.config && Object.entries(data.config).map(([key, value]) => (
                <Text key={key} size="xs" c="rgba(255,255,255,0.7)" mt={4}>
                    {key}: {value}
                </Text>
            ))}

            <Handle type="source" position={Position.Right} style={{ background: '#fff' }} />
        </div>
    );
}

export default memo(ProtocolNode);
