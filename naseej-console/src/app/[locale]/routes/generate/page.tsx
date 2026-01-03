"use client";

import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { CommandPalette } from "@/components/command-palette";
import { useSidebarStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";
import { use, useEffect, useState } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import {
    ArrowLeft,
    Loader2,
    Check,
    AlertCircle,
    Play,
    Code,
    Settings,
    Database,
    MessageSquare,
    Activity
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";

// Mock API client - should be real imports
async function generateRoutes(schemaId: string, partial: boolean) {
    const res = await fetch(`/api/schemas/${schemaId}/routes`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ partial }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
}

export default function GenerateRoutesPage({ params }: { params: Promise<{ locale: string }> }) {
    const { locale } = use(params);
    const { isCollapsed } = useSidebarStore();
    const isRTL = locale === "ar";
    const t = useTranslations("nav");
    const searchParams = useSearchParams();
    const router = useRouter();
    const schemaId = searchParams.get("schema");

    const [loading, setLoading] = useState(false);
    const [success, setSuccess] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [partialExport, setPartialExport] = useState(false);
    const [generatedCount, setGeneratedCount] = useState(0);

    // Tester State
    const [testTab, setTestTab] = useState("rest");
    const [testUrl, setTestUrl] = useState("http://localhost:8080/api/v1/resource");
    const [testMethod, setTestMethod] = useState("GET");
    const [testBody, setTestBody] = useState("");
    const [testResponse, setTestResponse] = useState("");

    if (!schemaId) {
        return <div className="p-10 text-center">Missing Schema ID</div>;
    }

    const handleGenerate = async () => {
        setLoading(true);
        setError(null);
        setSuccess(null);
        try {
            const res = await generateRoutes(schemaId, partialExport);
            setGeneratedCount(res.routes_created);
            setSuccess(res.message);
        } catch (err: any) {
            setError(err.message || "Failed to generate routes");
        } finally {
            setLoading(false);
        }
    };

    const handleTestRequest = async () => {
        if (testTab === "rest") {
            setTestResponse("Sending request...");
            try {
                const options: RequestInit = {
                    method: testMethod,
                    headers: { "Content-Type": "application/json" },
                };
                if (["POST", "PUT", "PATCH"].includes(testMethod) && testBody) {
                    options.body = testBody;
                }

                const res = await fetch(testUrl, options);
                const text = await res.text();
                let display = `Status: ${res.status} ${res.statusText}\n`;
                display += `Headers: ${JSON.stringify(Object.fromEntries(res.headers.entries()), null, 2)}\n\n`;

                try {
                    const json = JSON.parse(text);
                    display += JSON.stringify(json, null, 2);
                } catch {
                    display += text;
                }
                setTestResponse(display);
            } catch (err: any) {
                setTestResponse(`Error: ${err.message}\nMake sure CORS is enabled on the target or use a proxy.`);
            }
        } else {
            // Placeholder for other protocols requiring backend proxy
            setTestResponse(`[${testTab.toUpperCase()}] Simulation:\nSending payload to ${testUrl}...\n\n(Backend proxy required for non-HTTP protocols from browser)\nSuccess!`);
        }
    };

    return (
        <div className="min-h-screen bg-background">
            <Sidebar locale={locale} />
            <CommandPalette locale={locale} />
            <div
                className={cn(
                    "transition-all duration-300",
                    isCollapsed ? (isRTL ? "me-16" : "ms-16") : (isRTL ? "me-64" : "ms-64")
                )}
            >
                <Header locale={locale} />
                <main className="p-6">
                    <div className="max-w-4xl mx-auto space-y-6">
                        {/* Header */}
                        <div className="flex items-center gap-4">
                            <Button variant="ghost" size="icon" onClick={() => router.back()}>
                                <ArrowLeft className="h-4 w-4" />
                            </Button>
                            <div>
                                <h1 className="text-2xl font-bold">Generate Routes from Schema</h1>
                                <p className="text-muted-foreground">Convert your API schema into actionable gateway routes.</p>
                            </div>
                        </div>

                        <div className="grid gap-6 md:grid-cols-3">
                            {/* Configuration Config */}
                            <div className="md:col-span-1 space-y-4">
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="text-lg">Configuration</CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-4">
                                        <div className="flex items-center justify-between">
                                            <div className="space-y-0.5">
                                                <Label>Partial Export</Label>
                                                <p className="text-xs text-muted-foreground">Select subset of endpoints (Auto for now)</p>
                                            </div>
                                            <Switch checked={partialExport} onCheckedChange={setPartialExport} />
                                        </div>
                                        <Separator />
                                        <Button className="w-full" onClick={handleGenerate} disabled={loading}>
                                            {loading && <Loader2 className="h-4 w-4 me-2 animate-spin" />}
                                            Generate Routes
                                        </Button>
                                    </CardContent>
                                </Card>

                                {success && (
                                    <div className="p-4 rounded-lg bg-green-500/10 text-green-600 border border-green-500/20 flex items-start gap-3">
                                        <Check className="h-5 w-5 mt-0.5" />
                                        <div>
                                            <p className="font-medium">Success!</p>
                                            <p className="text-sm">{success}</p>
                                        </div>
                                    </div>
                                )}

                                {error && (
                                    <div className="p-4 rounded-lg bg-destructive/10 text-destructive border border-destructive/20 flex items-start gap-3">
                                        <AlertCircle className="h-5 w-5 mt-0.5" />
                                        <div>
                                            <p className="font-medium">Error</p>
                                            <p className="text-sm">{error}</p>
                                        </div>
                                    </div>
                                )}
                            </div>

                            {/* Testing Playground (Visible always but relevant after generation) */}
                            <div className="md:col-span-2">
                                <Card className="h-full flex flex-col">
                                    <CardHeader>
                                        <CardTitle className="flex items-center gap-2">
                                            <Activity className="h-5 w-5 text-primary" />
                                            API Tester
                                        </CardTitle>
                                        <CardDescription>
                                            Test your import credentials and protocols.
                                        </CardDescription>
                                    </CardHeader>
                                    <CardContent className="flex-1">
                                        <Tabs value={testTab} onValueChange={setTestTab} className="h-full flex flex-col">
                                            <TabsList className="grid w-full grid-cols-5 mb-4">
                                                <TabsTrigger value="rest">REST</TabsTrigger>
                                                <TabsTrigger value="soap">SOAP</TabsTrigger>
                                                <TabsTrigger value="grpc">gRPC</TabsTrigger>
                                                <TabsTrigger value="mcp">MCP</TabsTrigger>
                                                <TabsTrigger value="mqtt">MQTT</TabsTrigger>
                                            </TabsList>

                                            {/* REST Tester */}
                                            <TabsContent value="rest" className="space-y-4 flex-1">
                                                <div className="flex gap-2">
                                                    <select
                                                        className="h-10 rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                                        value={testMethod}
                                                        onChange={(e) => setTestMethod(e.target.value)}
                                                    >
                                                        <option>GET</option>
                                                        <option>POST</option>
                                                        <option>PUT</option>
                                                        <option>DELETE</option>
                                                    </select>
                                                    <Input value={testUrl} onChange={(e) => setTestUrl(e.target.value)} placeholder="https://api.example.com/..." className="flex-1" />
                                                    <Button onClick={handleTestRequest}>Send</Button>
                                                </div>
                                                <div className="space-y-2">
                                                    <Label>Body (JSON)</Label>
                                                    <Textarea
                                                        value={testBody}
                                                        onChange={(e) => setTestBody(e.target.value)}
                                                        className="font-mono text-xs h-32"
                                                        placeholder="{}"
                                                    />
                                                </div>
                                            </TabsContent>

                                            {/* SOAP Tester */}
                                            <TabsContent value="soap" className="space-y-4 flex-1">
                                                <div className="flex gap-2">
                                                    <Input value={testUrl} onChange={(e) => setTestUrl(e.target.value)} placeholder="WSDL Endpoint URL" className="flex-1" />
                                                    <Button onClick={handleTestRequest}>
                                                        <Play className="h-4 w-4 me-2" /> Call Action
                                                    </Button>
                                                </div>
                                                <div className="space-y-2">
                                                    <Label>XML Envelope</Label>
                                                    <Textarea
                                                        className="font-mono text-xs h-64"
                                                        placeholder='<soap:Envelope ...>...</soap:Envelope>'
                                                    />
                                                </div>
                                            </TabsContent>

                                            {/* gRPC Tester */}
                                            <TabsContent value="grpc" className="space-y-4 flex-1">
                                                <div className="flex gap-2">
                                                    <Input placeholder="localhost:50051" className="flex-1" />
                                                    <Input placeholder="Service/Method" className="flex-1" />
                                                    <Button onClick={handleTestRequest}>Invoke</Button>
                                                </div>
                                                <div className="space-y-2">
                                                    <Label>Protobuf Message (JSON format)</Label>
                                                    <Textarea
                                                        className="font-mono text-xs h-64"
                                                        placeholder='{ "id": 123 }'
                                                    />
                                                </div>
                                            </TabsContent>

                                            {/* MCP Tester */}
                                            <TabsContent value="mcp" className="space-y-4 flex-1">
                                                <div className="bg-slate-900 text-slate-50 p-4 rounded-md mb-4 text-sm">
                                                    <div className="flex items-center gap-2 mb-2 text-blue-400">
                                                        <MessageSquare className="h-4 w-4" />
                                                        <span>MCP Context Protocol</span>
                                                    </div>
                                                    <p>Test interactions with AI Models via MCP.</p>
                                                </div>
                                                <div className="space-y-2">
                                                    <Label>Prompt / Context</Label>
                                                    <Textarea
                                                        className="font-mono text-xs h-40"
                                                        placeholder='Describe the context or task...'
                                                    />
                                                </div>
                                                <Button className="w-full" onClick={handleTestRequest}>Send to Model</Button>
                                            </TabsContent>

                                            {/* MQTT Tester */}
                                            <TabsContent value="mqtt" className="space-y-4 flex-1">
                                                <div className="grid grid-cols-2 gap-4">
                                                    <div className="space-y-2">
                                                        <Label>Topic</Label>
                                                        <Input placeholder="sensors/temp" />
                                                    </div>
                                                    <div className="space-y-2">
                                                        <Label>QoS</Label>
                                                        <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50">
                                                            <option>0 - At most once</option>
                                                            <option>1 - At least once</option>
                                                            <option>2 - Exactly once</option>
                                                        </select>
                                                    </div>
                                                </div>
                                                <div className="space-y-2">
                                                    <Label>Payload</Label>
                                                    <Textarea
                                                        className="font-mono text-xs h-32"
                                                        placeholder='{"temp": 24}'
                                                    />
                                                </div>
                                                <Button className="w-full" onClick={handleTestRequest}>Publish Message</Button>
                                            </TabsContent>


                                            <div className="mt-4 border-t pt-4">
                                                <Label>Response / Output</Label>
                                                <div className="mt-2 rounded-md bg-muted p-4 font-mono text-xs h-40 overflow-auto whitespace-pre-wrap">
                                                    {testResponse || "// Waiting for request..."}
                                                </div>
                                            </div>
                                        </Tabs>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    );
}
