'use client';

import { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { Badge, Text, Tooltip } from '@mantine/core';

interface TransformNodeData {
    label: string;
    script: string;
}

function TransformNode({ data }: NodeProps<TransformNodeData>) {
    return (
        <Tooltip
            label={<pre style={{ fontSize: 10, margin: 0 }}>{data.script}</pre>}
            position="bottom"
            withArrow
            multiline
            w={300}
        >
            <div
                style={{
                    padding: '12px 16px',
                    borderRadius: 8,
                    minWidth: 140,
                    background: 'linear-gradient(135deg, #fab005 0%, #f59f00 100%)',
                    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    cursor: 'pointer',
                }}
            >
                <Handle type="target" position={Position.Left} style={{ background: '#fff' }} />

                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                    <span style={{ fontSize: 18 }}>⚙️</span>
                    <Text size="sm" fw={600} c="dark.9">
                        {data.label}
                    </Text>
                </div>

                <Badge size="xs" variant="filled" color="dark">
                    Rhai Script
                </Badge>

                <Handle type="source" position={Position.Right} style={{ background: '#fff' }} />
            </div>
        </Tooltip>
    );
}

export default memo(TransformNode);
