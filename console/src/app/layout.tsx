import '@mantine/core/styles.css';
import './globals.css';

import { ColorSchemeScript, MantineProvider, createTheme, DirectionProvider } from '@mantine/core';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Metadata } from 'next';
import { Providers } from './providers';

export const metadata: Metadata = {
    title: 'Naseej Console',
    description: 'Visual Control Plane for NaseejMesh API Gateway',
};

const theme = createTheme({
    primaryColor: 'blue',
    fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif',
});

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" dir="ltr">
            <head>
                <ColorSchemeScript defaultColorScheme="dark" />
            </head>
            <body>
                <Providers>
                    {children}
                </Providers>
            </body>
        </html>
    );
}
