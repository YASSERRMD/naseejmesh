import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { NextIntlClientProvider } from "next-intl";
import { getMessages } from "next-intl/server";
import { notFound } from "next/navigation";
import { routing } from "@/i18n/routing";
import { ToastProvider } from "@/components/toast-provider";
import "../globals.css";

const inter = Inter({
    subsets: ["latin"],
    variable: "--font-inter",
});

export const metadata: Metadata = {
    title: "Naseej Console",
    description: "Enterprise Integration Dashboard",
};

interface RootLayoutProps {
    children: React.ReactNode;
    params: Promise<{ locale: string }>;
}

export function generateStaticParams() {
    return routing.locales.map((locale) => ({ locale }));
}

export default async function RootLayout({
    children,
    params,
}: RootLayoutProps) {
    const { locale } = await params;

    if (!routing.locales.includes(locale as any)) {
        notFound();
    }

    const messages = await getMessages();
    const direction = locale === "ar" ? "rtl" : "ltr";
    const fontClass = locale === "ar" ? "font-arabic" : "";

    return (
        <html lang={locale} dir={direction} suppressHydrationWarning>
            <head>
                {locale === "ar" && (
                    <link
                        href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans+Arabic:wght@400;500;600;700&display=swap"
                        rel="stylesheet"
                    />
                )}
            </head>
            <body className={`${inter.variable} ${fontClass} antialiased`}>
                <NextIntlClientProvider messages={messages}>
                    {children}
                    <ToastProvider />
                </NextIntlClientProvider>
            </body>
        </html>
    );
}
