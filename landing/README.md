# harness-lint landing page

A static single-page marketing site for harness-lint, built with **Vite + React +
TypeScript**. No backend, no router, no CSS framework — just one component tree and
one stylesheet, so it builds to plain static files and is easy to maintain.

## Develop

```sh
npm install
npm run dev      # local dev server with HMR
```

## Build

```sh
npm run build    # type-check + emit static site to dist/
npm run preview  # serve the production build locally
```

The contents of `dist/` are fully static — deploy them to GitHub Pages, Netlify,
Cloudflare Pages, or any file host. `vite.config.ts` uses a relative `base`, so the
build works from a subpath without extra config.

## Edit the copy

All wording, links, features, and command lists live in
[`src/content.ts`](src/content.ts). The components in `src/` only render that data,
so updating the page rarely means touching JSX. Theme colours and spacing are CSS
variables at the top of [`src/styles.css`](src/styles.css).
