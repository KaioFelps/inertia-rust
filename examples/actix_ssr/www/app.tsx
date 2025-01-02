import "./index.css"

import { hydrateRoot } from 'react-dom/client'
import { createInertiaApp } from '@inertiajs/react'
import { resolvePageComponent } from "laravel-vite-plugin/inertia-helpers";

export const appName = 'Inertia Test'
export const titleResolver = (title: string) => (title ? `${appName} - ${title}` : title);

createInertiaApp({
  progress: { color: '#eedcff', includeCSS: true },

  title: titleResolver,

  resolve: async (component) => {
    return await resolvePageComponent(
        `./pages/${component}.tsx`,
        import.meta.glob('./pages/**/*.tsx', { eager: false })
    );
  },

  setup({ el, App, props }) {
    // createRoot(el).render(<App {...props} />);
    hydrateRoot(el, <App {...props} />);
  },
})
