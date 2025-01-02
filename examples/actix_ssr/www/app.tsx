import "./index.css"

import { hydrateRoot } from 'react-dom/client'
import { createInertiaApp } from '@inertiajs/react'

export const appName = 'Inertia Test'
export const titleResolver = (title: string) => (title ? `${appName} - ${title}` : title);

createInertiaApp({
  progress: { color: '#eedcff', includeCSS: true },

  title: titleResolver,

  resolve: async (component) => {
    const pages = import.meta.glob('./pages/**/*.tsx', { eager: true });
    return pages[`./pages/${component}.tsx`];
  },

  setup({ el, App, props }) {
    // createRoot(el).render(<App {...props} />);
    hydrateRoot(el, <App {...props} />);
  },
})
