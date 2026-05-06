# Kesh marketing website

Static HTML site published to GitHub Pages from `website/` on every push to `main`.

## Stack

- **HTML5** — no framework, no build step.
- **Tailwind CSS** via [Play CDN](https://tailwindcss.com/docs/installation/play-cdn) — modify classes directly in HTML, refresh.
- **Inter** + **JetBrains Mono** via Google Fonts.
- Custom CSS in `css/style.css` for hero gradient, animations, focus rings, reduced-motion support.

## Pages

| File | Path | Purpose |
|------|------|---------|
| `index.html` | `/` | Homepage — hero, features, tech stack, CTA |
| `about.html` | `/about.html` | Mission, architecture, Swiss compliance, BMAD methodology |
| `roadmap.html` | `/roadmap.html` | Visual epic timeline (v0.1 + v0.2) |
| `issues.html` | `/issues.html` | Issue templates and label-based deep links |
| `404.html` | served on unknown path | GitHub Pages renders this automatically on 404 |

## Local preview

Any static server works:

```sh
cd website
python3 -m http.server 8000
# → http://localhost:8000
```

## Deployment

Triggered by `.github/workflows/deploy-pages.yml` on every push to `main` that touches `website/**`. The workflow uploads `website/` as a Pages artifact via `actions/upload-pages-artifact@v3`.

### One-time setup (repo admin)

In **Settings → Pages** of the GitHub repository:

1. **Source**: select **GitHub Actions** (not the legacy "Deploy from a branch").
2. The first successful workflow run publishes the site to `https://guycorbaz.github.io/kesh/`.

A custom domain can be configured later by adding a `CNAME` file with the domain inside this `website/` directory, plus a DNS record at the registrar.

## Editing

- **Copy / wording** → edit the relevant `.html` file directly.
- **Visual tweaks** → modify Tailwind utility classes in HTML. For repeated patterns, add a class to `css/style.css`.
- **New page** → copy `about.html` as a template (preserves nav, footer, and font setup).

The `.nojekyll` file disables Jekyll preprocessing so file paths starting with `_` work as expected.
