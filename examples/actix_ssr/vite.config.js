import react from '@vitejs/plugin-react';
import laravel from 'laravel-vite-plugin';
import { defineConfig } from 'vite';

export default defineConfig(() => {
    return {
        plugins: [    
            laravel({
                input: 'www/app.tsx',
                buildDirectory: 'bundle',
                refresh: 'www/**',
                ssrOutputDirectory: "dist/ssr",
                ssr: "www/ssr.tsx",
            }),
            react(),
        ],
        // important to serve statics from public dir directly from "localhost:5173/" instead of "localhost:5173/public"
        // just "public" without slash prefix won't work
        publicDir: "/public"
    }
});
