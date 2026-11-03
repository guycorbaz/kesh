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

// Story v016-2b (#816, AC-19) — le rôle courant, pour le lien de saisie à la main.
const role = vi.hoisted(() => ({ value: 'Comptable' as string | null }));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return role.value === null ? null : { role: role.value, username: 'u', userId: '1' };
		},
	},
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

		// Revue de code P2, L2 : le texte EXACT d'un paragraphe, pas une sous-chaîne.
		const paragraphs = Array.from(container.querySelectorAll('p')).map((p) => p.textContent?.trim());
		expect(paragraphs).toContain('common-loading');
		expect(container.textContent).not.toContain('Chargement');
		expect(container.querySelector('[data-testid="bank-import-upload"]')).toBeNull();

		pending.resolve({ bankAccounts: [] });
		await flush();
		expect(container.textContent).not.toContain('common-loading');
		expect(container.querySelector('[data-testid="bank-import-upload"]')).not.toBeNull();
	});
});

// Story v016-2b (#816, AC-19) — « Saisir un relevé à la main » : Admin et Comptable
// seulement (mutation M28 : lien affiché à tout rôle).
describe('Page d’import bancaire — lien de saisie à la main (Story v016-2b, AC-19)', () => {
	for (const r of ['Admin', 'Comptable']) {
		it(`${r} ⇒ le lien vers /bank-import/manual est rendu`, async () => {
			role.value = r;
			companyGet.mockResolvedValue({ bankAccounts: [] });
			const { container } = render(Page);
			await flush();
			const lien = container.querySelector<HTMLAnchorElement>('[data-testid="bank-import-manual-link"]');
			expect(lien?.getAttribute('href')).toBe('/bank-import/manual');
			expect(lien?.textContent?.trim()).toBe('bank-import-manual-link');
		});
	}

	for (const r of ['Consultation', null]) {
		it(`${r ?? 'aucun utilisateur'} ⇒ aucun lien (M28)`, async () => {
			role.value = r;
			companyGet.mockResolvedValue({ bankAccounts: [] });
			const { container } = render(Page);
			await flush();
			// La page est chargée : l'absence du lien n'est pas celle d'une page vide.
			expect(container.querySelector('[data-testid="bank-import-upload"]')).not.toBeNull();
			expect(container.querySelector('[data-testid="bank-import-manual-link"]')).toBeNull();
		});
	}
});
