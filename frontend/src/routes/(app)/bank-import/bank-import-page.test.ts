// Story 27-3 (#626), revue de code P1, E-7 — test Vitest de la page d'import
// bancaire : l'indicateur de chargement de la PAGE passe par le catalogue
// (`common-loading`, AC17 H11). Aucun autre test ne rendait cette page : seule
// la borne `sitesTotal` protégeait ce site.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe
// `+` dans `src/routes/` (patron des autres tests de page).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';

// Mode du mock i18n : la CLÉ au lieu du repli, pour voir quelle clé est rendue.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string) => key,
}));

const companyGet = vi.hoisted(() => vi.fn());
vi.mock('$lib/shared/utils/api-client', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/shared/utils/api-client')>()),
	apiClient: { get: companyGet },
}));

vi.mock('$lib/features/bank-import/bank-import.api', () => ({
	listBankImports: vi.fn(async () => ({ items: [], total: 0 })),
	previewBankImport: vi.fn(),
	createBankImport: vi.fn(),
}));

vi.mock('$lib/features/bank-import/bank-profile.api', () => ({
	listBankProfiles: vi.fn(async () => ({ items: [], total: 0, page: 1, perPage: 50 })),
}));

import Page from './+page.svelte';

function deferred<T>() {
	let resolve!: (v: T) => void;
	const promise = new Promise<T>((res) => {
		resolve = res;
	});
	return { promise, resolve };
}

async function flush(): Promise<void> {
	for (let i = 0; i < 3; i++) {
		await new Promise((r) => setTimeout(r, 0));
		await tick();
	}
}

beforeEach(() => {
	companyGet.mockReset();
});

afterEach(() => {
	cleanup();
});

describe('Page d’import bancaire — chargement (Story 27-3, AC17 H11)', () => {
	it('pendant le chargement, la page affiche la clé common-loading ; ensuite, l’écran d’import', async () => {
		const pending = deferred<unknown>();
		companyGet.mockReturnValueOnce(pending.promise);
		const { container } = render(Page);
		await flush();

		expect(container.textContent).toContain('common-loading');
		expect(container.textContent).not.toContain('Chargement');
		expect(container.querySelector('[data-testid="bank-import-upload"]')).toBeNull();

		pending.resolve({ bankAccounts: [] });
		await flush();
		expect(container.textContent).not.toContain('common-loading');
		expect(container.querySelector('[data-testid="bank-import-upload"]')).not.toBeNull();
	});
});
