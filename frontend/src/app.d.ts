// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
	// Story 10.3 : version Kesh injectée au build par Vite `define`
	// (cf. vite.config.ts). Lue depuis `package.json:version` via
	// `process.env.npm_package_version`, fallback `'dev'`.
	const __APP_VERSION__: string;
}

export {};
