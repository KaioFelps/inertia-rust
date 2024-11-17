import ReactDOMServer from 'react-dom/server'
import { createInertiaApp } from '@inertiajs/react'
import server from "@inertiajs/core/server";

const appName = 'Inertia Test'

function render(page: any) {
    return createInertiaApp({
        page,
        title: (title) => (title ? `${appName} - ${title}` : title),
        render: ReactDOMServer.renderToString,
        resolve: (name: string) => {
            const pages = import.meta.glob('./pages/**/*.tsx', { eager: true })
            const page: any = pages[`./pages/${name}.tsx`]
            return page
        },
        setup: ({ App, props }) => <App {...props} />,
    })
}

const portArgIdx = process.argv.indexOf("--port");
const port = portArgIdx >= 0 ? Number(process.argv[portArgIdx + 1]) : 1000;

server(async (page) => await render(page), port)