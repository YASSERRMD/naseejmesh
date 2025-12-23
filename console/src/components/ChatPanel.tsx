'use client';

import { useState, useRef, useEffect } from 'react';
import {
    Stack,
    TextInput,
    Button,
    ScrollArea,
    Paper,
    Text,
    Group,
    Title,
    Loader,
    Code,
} from '@mantine/core';

interface Message {
    id: string;
    role: 'user' | 'assistant';
    content: string;
    timestamp: Date;
}

export default function ChatPanel() {
    const [messages, setMessages] = useState<Message[]>([
        {
            id: '1',
            role: 'assistant',
            content: 'Hello! I\'m the Naseej Architect. I can help you create integration routes. Try saying:\n\n• "Create a route from MQTT to HTTP"\n• "Search for user APIs"\n• "Validate this script: let x = 1 + 2;"',
            timestamp: new Date(),
        },
    ]);
    const [input, setInput] = useState('');
    const [loading, setLoading] = useState(false);
    const scrollRef = useRef<HTMLDivElement>(null);

    const sendMessage = async () => {
        if (!input.trim()) return;

        const userMessage: Message = {
            id: Date.now().toString(),
            role: 'user',
            content: input,
            timestamp: new Date(),
        };

        setMessages((prev) => [...prev, userMessage]);
        setInput('');
        setLoading(true);

        try {
            const response = await fetch('http://localhost:3001/api/chat', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ message: input }),
            });

            if (response.ok) {
                const data = await response.json();
                const assistantMessage: Message = {
                    id: (Date.now() + 1).toString(),
                    role: 'assistant',
                    content: data.response,
                    timestamp: new Date(),
                };
                setMessages((prev) => [...prev, assistantMessage]);
            } else {
                throw new Error('Failed to get response');
            }
        } catch (error) {
            const errorMessage: Message = {
                id: (Date.now() + 1).toString(),
                role: 'assistant',
                content: 'Sorry, I couldn\'t connect to the server. Make sure the API is running on port 3001.',
                timestamp: new Date(),
            };
            setMessages((prev) => [...prev, errorMessage]);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'smooth' });
    }, [messages]);

    return (
        <Stack h="100%" gap="md">
            <Group justify="space-between">
                <Title order={5}>🤖 AI Architect</Title>
            </Group>

            <ScrollArea
                flex={1}
                viewportRef={scrollRef}
                style={{ background: '#141517', borderRadius: 8, padding: 12 }}
            >
                <Stack gap="md">
                    {messages.map((msg) => (
                        <Paper
                            key={msg.id}
                            p="sm"
                            radius="md"
                            style={{
                                background: msg.role === 'user' ? '#228be6' : '#25262B',
                                alignSelf: msg.role === 'user' ? 'flex-end' : 'flex-start',
                                maxWidth: '90%',
                            }}
                        >
                            <Text
                                size="sm"
                                c={msg.role === 'user' ? 'white' : 'gray.3'}
                                style={{ whiteSpace: 'pre-wrap' }}
                            >
                                {msg.content}
                            </Text>
                            <Text size="xs" c="dimmed" mt={4}>
                                {msg.timestamp.toLocaleTimeString()}
                            </Text>
                        </Paper>
                    ))}
                    {loading && (
                        <Group gap="xs">
                            <Loader size="xs" />
                            <Text size="sm" c="dimmed">Thinking...</Text>
                        </Group>
                    )}
                </Stack>
            </ScrollArea>

            <Group gap="xs">
                <TextInput
                    flex={1}
                    placeholder="Ask the Architect..."
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
                    disabled={loading}
                    styles={{
                        input: {
                            background: '#25262B',
                            border: '1px solid #373A40',
                        },
                    }}
                />
                <Button onClick={sendMessage} loading={loading}>
                    Send
                </Button>
            </Group>
        </Stack>
    );
}
