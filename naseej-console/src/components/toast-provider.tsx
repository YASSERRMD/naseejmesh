"use client";

import { Toaster } from "sonner";

export function ToastProvider() {
    return (
        <Toaster
            position="bottom-right"
            toastOptions={{
                style: {
                    background: "hsl(var(--color-card))",
                    color: "hsl(var(--color-card-foreground))",
                    border: "1px solid hsl(var(--color-border))",
                },
            }}
            richColors
        />
    );
}
