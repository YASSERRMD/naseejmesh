'use client';

import {
    Stack,
    Title,
    NavLink,
    Text,
    Badge,
    Divider,
    Group,
} from '@mantine/core';

export default function Sidebar() {
    return (
        <Stack gap="md">
            <Title order={5} c="dimmed">Navigation</Title>

            <NavLink
                label="Flow Canvas"
                leftSection="🕸️"
                active
                variant="filled"
                style={{ borderRadius: 8 }}
            />

            <NavLink
                label="Routes"
                leftSection="🛣️"
                rightSection={<Badge size="xs" color="blue">3</Badge>}
                style={{ borderRadius: 8 }}
            />

            <NavLink
                label="Transformations"
                leftSection="⚙️"
                rightSection={<Badge size="xs" color="yellow">1</Badge>}
                style={{ borderRadius: 8 }}
            />

            <NavLink
                label="API Schemas"
                leftSection="📄"
                style={{ borderRadius: 8 }}
            />

            <Divider my="md" />

            <Title order={5} c="dimmed">Quick Actions</Title>

            <NavLink
                label="New Route"
                leftSection="➕"
                description="Create integration"
                style={{ borderRadius: 8 }}
            />

            <NavLink
                label="Import OpenAPI"
                leftSection="📥"
                description="Add to knowledge base"
                style={{ borderRadius: 8 }}
            />

            <NavLink
                label="Test Transform"
                leftSection="🧪"
                description="Dry-run a script"
                style={{ borderRadius: 8 }}
            />

            <Divider my="md" />

            <Title order={5} c="dimmed">Gateway Status</Title>

            <Group gap="xs">
                <Badge color="green" variant="dot">Healthy</Badge>
                <Text size="xs" c="dimmed">v0.1.0</Text>
            </Group>

            <Stack gap={4}>
                <Group justify="space-between">
                    <Text size="xs" c="dimmed">Routes</Text>
                    <Text size="xs" fw={600}>3 active</Text>
                </Group>
                <Group justify="space-between">
                    <Text size="xs" c="dimmed">Uptime</Text>
                    <Text size="xs" fw={600}>2h 34m</Text>
                </Group>
                <Group justify="space-between">
                    <Text size="xs" c="dimmed">Requests/s</Text>
                    <Text size="xs" fw={600}>1,234</Text>
                </Group>
            </Stack>
        </Stack>
    );
}
