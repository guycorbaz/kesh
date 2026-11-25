// Story 1582 (#1582) — le store du nom de la comptabilité ouverte.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';

const fetchCompanyCurrentMock = vi.fn<() => Promise<CompanyCurrentResponse>>();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
}));

import { companyIdentity } from './company-identity.svelte';

function reponse(name: string): CompanyCurrentResponse {
	return {
		company: { id: 1, name, version: 1 } as unknown as CompanyCurrentResponse['company'],
		bankAccounts: [],
	};
}

/** Une lecture en vol qu'on libère à la main. */
function enVol(): { promise: Promise<CompanyCurrentResponse>; resolve: (r: CompanyCurrentResponse) => void } {
	let resolve!: (r: CompanyCurrentResponse) => void;
	const promise = new Promise<CompanyCurrentResponse>((r) => (resolve = r));
	return { promise, resolve };
}

describe('companyIdentity (Story 1582)', () => {
	let errorSpy: ReturnType<typeof vi.spyOn>;
	beforeEach(() => {
		// Singleton de module : rien n'est hérité du test précédent.
		companyIdentity.clear();
		fetchCompanyCurrentMock.mockReset();
		errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
	});
	afterEach(() => errorSpy.mockRestore());

	it('lecture réussie : le nom, rogné', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse('  Dupont Sàrl  '));
		await companyIdentity.load();
		expect(companyIdentity.name).toBe('Dupont Sàrl');
	});

	it('nom vide ou d’espaces : null (D2)', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse('   '));
		await companyIdentity.load();
		expect(companyIdentity.name).toBeNull();
		companyIdentity.set('X');
		companyIdentity.set('  ');
		expect(companyIdentity.name).toBeNull();
	});

	it('échec : null, aucune exception, console.error appelé (D9)', async () => {
		companyIdentity.set('Avant');
		fetchCompanyCurrentMock.mockRejectedValue(new Error('500'));
		await expect(companyIdentity.load()).resolves.toBeUndefined();
		expect(companyIdentity.name).toBeNull();
		expect(errorSpy).toHaveBeenCalled();
	});

	it('set("A") puis load() en vol : null jusqu’à la réponse (AC8)', async () => {
		companyIdentity.set('A');
		const v = enVol();
		fetchCompanyCurrentMock.mockReturnValue(v.promise);
		const p = companyIdentity.load();
		expect(companyIdentity.name).toBeNull();
		v.resolve(reponse('B'));
		await p;
		expect(companyIdentity.name).toBe('B');
	});

	it('set() pendant une lecture en vol l’emporte sur la réponse tardive', async () => {
		const v = enVol();
		fetchCompanyCurrentMock.mockReturnValue(v.promise);
		const p = companyIdentity.load();
		companyIdentity.set('Corrigé SA');
		v.resolve(reponse('Ancien SA'));
		await p;
		expect(companyIdentity.name).toBe('Corrigé SA');
	});

	it('clear() pendant une lecture en vol : la réponse tardive est sans effet', async () => {
		const v = enVol();
		fetchCompanyCurrentMock.mockReturnValue(v.promise);
		const p = companyIdentity.load();
		companyIdentity.clear();
		v.resolve(reponse('Tardif SA'));
		await p;
		expect(companyIdentity.name).toBeNull();
	});

	it('une lecture tardive en échec ne touche pas un set() intervenu entre-temps', async () => {
		let rejeter!: (e: unknown) => void;
		fetchCompanyCurrentMock.mockReturnValue(new Promise((_, rej) => (rejeter = rej)));
		const p = companyIdentity.load();
		companyIdentity.set('Corrigé SA');
		rejeter(new Error('boom'));
		await p;
		expect(companyIdentity.name).toBe('Corrigé SA');
	});

	it('load() rejoué remplace le nom', async () => {
		fetchCompanyCurrentMock.mockResolvedValueOnce(reponse('Un SA'));
		await companyIdentity.load();
		expect(companyIdentity.name).toBe('Un SA');
		fetchCompanyCurrentMock.mockResolvedValueOnce(reponse('Deux SA'));
		await companyIdentity.load();
		expect(companyIdentity.name).toBe('Deux SA');
	});

	// M11a : une séquence en `$state` ferait dépendre l'effet de ce que `load()` écrit
	// (`effect_update_depth_exceeded`). Le layout appelle `load()` depuis un `$effect`.
	it('load() appelé depuis un $effect ne le relance pas (séquence non réactive)', () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse('Effet SA'));
		let lancements = 0;
		const stop = $effect.root(() => {
			$effect(() => {
				lancements++;
				void companyIdentity.load();
			});
		});
		try {
			flushSync();
			expect(lancements).toBe(1);
			expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(1);
		} finally {
			stop();
		}
	});
});
