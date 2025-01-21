import ReactDOMServer from 'react-dom/server'
import { createInertiaApp } from '@inertiajs/react'
import createServer from "@inertiajs/react/server";
import type {Page} from "@inertiajs/core/types"

const portArgIdx = process.argv.indexOf("--port");
const port = portArgIdx >= 0 ? Number(process.argv[portArgIdx + 1]) : 1000;

export const appName = 'Inertia Test'
export const titleResolver = (title: string) => (title ? `${appName} - ${title}` : title);

createServer((page: Page) => {
    return createInertiaApp({
        page,

        title: titleResolver,
        
        render: ReactDOMServer.renderToString,

        resolve: async (component) => {
            const pages = import.meta.glob('./pages/**/*.tsx', { eager: true });
            return pages[`./pages/${component}.tsx`];
        },
        
        setup: ({ App, props }) => {
            return <App {...props} />
        },
    })},

    port
)
