'use client';

import {
    AppShell,
    Group,
    Title,
    ActionIcon,
    Tooltip,
    Text,
    Badge,
} from '@mantine/core';
import { useState } from 'react';
import FlowCanvas from '@/components/FlowCanvas';
import ChatPanel from '@/components/ChatPanel';
import Sidebar from '@/components/Sidebar';

export default function Home() {
    const [chatOpen, setChatOpen] = useState(true);

    return (
        <AppShell
            header={{ height: 60 }}
            navbar={{ width: 280, breakpoint: 'sm' }}
            aside={{ width: chatOpen ? 400 : 0, breakpoint: 'md' }}
            padding={0}
        >
            <AppShell.Header
                style={{
                    background: 'rgba(26, 27, 30, 0.95)',
                    backdropFilter: 'blur(10px)',
                    borderBottom: '1px solid #373A40',
                }}
            >
                <Group h="100%" px="md" justify="space-between">
                    <Group>
                        <Title order={3} c="blue.4">
                            🕸️ Naseej Console
                        </Title>
                        <Badge variant="light" color="green" size="sm">
                            Connected
                        </Badge>
                    </Group>
                    <Group>
                        <Text size="sm" c="dimmed">
                            Press ⌘K for Command Bar
                        </Text>
                        <Tooltip label={chatOpen ? 'Hide Chat' : 'Show Chat'}>
                            <ActionIcon
                                variant="subtle"
                                onClick={() => setChatOpen(!chatOpen)}
                                size="lg"
                            >
                                💬
                            </ActionIcon>
                        </Tooltip>
                    </Group>
                </Group>
            </AppShell.Header>

            <AppShell.Navbar
                p="md"
                style={{
                    background: 'rgba(26, 27, 30, 0.95)',
                    borderRight: '1px solid #373A40',
                }}
            >
                <Sidebar />
            </AppShell.Navbar>

            <AppShell.Main>
                <div style={{ height: 'calc(100vh - 60px)' }}>
                    <FlowCanvas />
                </div>
            </AppShell.Main>

            {chatOpen && (
                <AppShell.Aside
                    p="md"
                    style={{
                        background: 'rgba(26, 27, 30, 0.95)',
                        borderLeft: '1px solid #373A40',
                    }}
                >
                    <ChatPanel />
                </AppShell.Aside>
            )}
        </AppShell>
    );
}
