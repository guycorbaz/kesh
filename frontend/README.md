# sv

Everything you need to build a Svelte project, powered by [`sv`](https://github.com/sveltejs/cli).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```sh
# create a new project
npx sv create my-app
```

To recreate this project with the same configuration:

```sh
# recreate this project
npx sv@0.13.2 create --template minimal --types ts --no-install frontend
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```sh
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```sh
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://svelte.dev/docs/kit/adapters) for your target environment.

## Internationalization (i18n)

The frontend uses Fluent message files (`.ftl`) for multilingual support. Messages are organized by locale in `crates/kesh-i18n/locales/`:

```
crates/kesh-i18n/locales/
  fr-CH/messages.ftl
  de-CH/messages.ftl
  it-CH/messages.ftl
  en-CH/messages.ftl
```

### Using i18n Keys

To add translatable text to a component:

1. **Define the message** in all 4 locale files under `crates/kesh-i18n/locales/*/messages.ftl`
2. **Use in code** with the `i18nMsg` helper:
   ```svelte
   <script>
     import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
   </script>
   <button>{i18nMsg('my-key', 'Fallback Label')}</button>
   ```

### Key Ownership Pattern

The **Key Ownership Pattern** enforces that feature-specific keys (`feature-name-*`) are only used within their feature folder. Global namespaces (`error-*`, `common-*`, `nav-*`, etc.) can be used everywhere.

Run the linter to validate key scoping:

```bash
npm run lint-i18n-ownership
```

For detailed patterns, examples, and troubleshooting, see [`docs/i18n-key-ownership-pattern.md`](../docs/i18n-key-ownership-pattern.md).
