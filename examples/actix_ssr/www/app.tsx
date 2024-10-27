import "./index.css"

import { createRoot } from 'react-dom/client'
import { createInertiaApp } from '@inertiajs/react'
import { resolvePageComponent } from "laravel-vite-plugin/inertia-helpers"

const appName = 'Inertia Test'

createInertiaApp({
  progress: { color: '#eedcff', includeCSS: true },

  title: (title) => (title ? `${appName} - ${title}` : title),

  resolve: async (name) => {
    const page: any = await resolvePageComponent(
      `./pages/${name}.tsx`,
      import.meta.glob('./pages/**/*.tsx')
    )

    return page
  },

  setup({ el, App, props }) {
    // hydrateRoot(el, <App {...props} />)
    createRoot(el).render(<App {...props} />)
  },
})
