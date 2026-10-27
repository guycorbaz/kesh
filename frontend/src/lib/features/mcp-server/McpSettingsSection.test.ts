/**
 * Story 32-7 (refs #723) — la section « Réglages du serveur » (AC-5, AC-6, AC-7).
 *
 * Un cas par lettre de l'AC-5, le corps envoyé comparé EN ENTIER ; chaque
 * avertissement de l'AC-6 présent quand sa condition l'est, absent sinon (témoin) ;
 * l'encadré et ses quatre phrases dans tous les états ; l'AC-7.
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./mcp-server.api', () => ({
	getMcpServerSettings: vi.fn(),
	updateMcpServerSettings: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import * as api from './mcp-server.api';
import McpSettingsSection from './McpSettingsSection.svelte';
import type { McpServerSettings } from './mcp-server.types';

const mockApi = vi.mocked(api);

const DEFAUTS: McpServerSettings = {
	enabled: false,
	addressFilterEnabled: true,
	allowedNetworks: ['127.0.0.0/8', '::1/128', '10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16', 'fc00::/7', 'fe80::/10'],
	trustedProxies: [],
	allowedOrigins: [],
	version: 0,
	updatedAt: null,
};

function reglages(over: Partial<McpServerSettings> = {}): McpServerSettings {
	return { ...DEFAUTS, ...over };
}

/** Les quatre phrases de l'encadré, cherchées par leur repli fr-CH. */
const ENCADRE = [
	'Derrière un mandataire non déclaré parmi les mandataires de confiance',
	'Il ne remplace ni le pare-feu ni le VPN',
	'La plage 100.64.0.0/10 (VPN de type Tailscale) n’est pas admise par défaut',
	'Aucune limite de débit n’est posée sur /mcp.',
];

beforeEach(() => {
	vi.clearAllMocks();
	vi.spyOn(console, 'error').mockImplementation(() => {});
});
afterEach(() => cleanup());

async function monter(r: McpServerSettings = reglages()) {
	mockApi.getMcpServerSettings.mockResolvedValueOnce(r);
	const vue = render(McpSettingsSection);
	await vue.findByTestId('mcp-server-settings-form');
	// le chargement terminé (le formulaire est rendu dès le montage)
	await waitFor(() => expect(vue.queryByTestId('mcp-server-settings-loading')).toBeNull());
	return vue;
}

function zone(vue: ReturnType<typeof render>, id: string): HTMLTextAreaElement {
	return vue.getByTestId(id) as HTMLTextAreaElement;
}

describe('McpSettingsSection — lecture et enregistrement (AC-5)', () => {
	it('(a) lignes rognées, lignes vides retirées ; (b) version lue ; corps entier', async () => {
		const vue = await monter();
		mockApi.updateMcpServerSettings.mockResolvedValueOnce(reglages({ version: 1, enabled: true }));
		await fireEvent.click(vue.getByTestId('mcp-server-settings-enabled'));
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), {
			target: { value: '  10.0.0.0/8  \n\n192.168.1.10\n   \n' },
		});
		await fireEvent.input(zone(vue, 'mcp-server-settings-proxies'), { target: { value: '\n' } });
		await fireEvent.input(zone(vue, 'mcp-server-settings-origins'), {
			target: { value: ' http://agent.local:8080 ' },
		});
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		await waitFor(() => expect(mockApi.updateMcpServerSettings).toHaveBeenCalledTimes(1));
		expect(mockApi.updateMcpServerSettings.mock.calls[0][0]).toEqual({
			enabled: true,
			addressFilterEnabled: true,
			allowedNetworks: ['10.0.0.0/8', '192.168.1.10'],
			trustedProxies: [],
			allowedOrigins: ['http://agent.local:8080'],
			version: 0,
		});
	});

	it('(c) succès : les champs remplacés par la réponse (formes canoniques), message, phrase fixe', async () => {
		const vue = await monter(reglages({ version: 3, updatedAt: '2026-10-25T10:00:00.000Z' }));
		expect(vue.getByText('Kesh enregistre chaque entrée sous sa forme canonique.')).toBeTruthy();
		mockApi.updateMcpServerSettings.mockResolvedValueOnce(
			reglages({ version: 4, allowedNetworks: ['192.168.1.10/32', '10.0.0.0/8'], updatedAt: '2026-10-25T11:00:00.000Z' }),
		);
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), {
			target: { value: '192.168.1.10\n010.0.0.0/8\n10.0.0.0/8' },
		});
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		expect((await vue.findByTestId('mcp-server-settings-saved')).textContent).toContain('Réglages enregistrés.');
		expect(zone(vue, 'mcp-server-settings-networks').value).toBe('192.168.1.10/32\n10.0.0.0/8');
		expect(mockApi.updateMcpServerSettings.mock.calls[0][0].version).toBe(3);
	});

	it('date de modification ou « jamais modifiés »', async () => {
		const vue = await monter();
		expect(vue.getByTestId('mcp-server-settings-updated').textContent).toContain(
			'Jamais modifiés — valeurs par défaut.',
		);
		cleanup();
		const vue2 = await monter(reglages({ version: 2, updatedAt: '2026-10-25T10:00:00.000Z' }));
		expect(vue2.getByTestId('mcp-server-settings-updated').textContent).toContain('Dernière modification :');
		expect(vue2.getByTestId('mcp-server-settings-updated').textContent).not.toContain('Jamais');
	});

	it('(d) 400 : le message du serveur, saisies conservées', async () => {
		const vue = await monter();
		mockApi.updateMcpServerSettings.mockRejectedValueOnce({
			code: 'VALIDATION_ERROR',
			status: 400,
			message: 'Réseau à bits d’hôte : 10.0.0.1/8',
		});
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), { target: { value: '10.0.0.1/8' } });
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		const erreur = await vue.findByTestId('mcp-server-settings-error');
		expect(erreur.getAttribute('role')).toBe('alert');
		expect(erreur.textContent).toContain('Réseau à bits d’hôte : 10.0.0.1/8');
		expect(zone(vue, 'mcp-server-settings-networks').value).toBe('10.0.0.1/8');
		expect(mockApi.getMcpServerSettings).toHaveBeenCalledTimes(1);
	});

	it('(e) 409 : message de conflit, puis rechargement', async () => {
		const vue = await monter();
		mockApi.updateMcpServerSettings.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		mockApi.getMcpServerSettings.mockResolvedValueOnce(reglages({ version: 5, enabled: true }));
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		expect((await vue.findByTestId('mcp-server-settings-error')).textContent).toContain(
			'Les réglages ont changé entre-temps ; ils sont rechargés, vos modifications ne sont pas enregistrées.',
		);
		await waitFor(() => expect(mockApi.getMcpServerSettings).toHaveBeenCalledTimes(2));
		await waitFor(() =>
			expect((vue.getByTestId('mcp-server-settings-enabled') as HTMLInputElement).checked).toBe(true),
		);
	});

	it('(f) erreur hors ApiError : le générique, ni le message technique ni [object Object]', async () => {
		const vue = await monter();
		mockApi.updateMcpServerSettings.mockRejectedValueOnce(new Error('boom'));
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		const erreur = await vue.findByTestId('mcp-server-settings-error');
		expect(erreur.textContent?.trim()).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('(g) chargement : Enregistrer désactivé ; échec : message, phrase fixe, Enregistrer désactivé', async () => {
		let resoudre!: (r: McpServerSettings) => void;
		mockApi.getMcpServerSettings.mockReturnValueOnce(new Promise((r) => (resoudre = r)));
		const vue = render(McpSettingsSection);
		expect((vue.getByTestId('mcp-server-settings-save') as HTMLButtonElement).disabled).toBe(true);
		resoudre(reglages());
		await waitFor(() =>
			expect((vue.getByTestId('mcp-server-settings-save') as HTMLButtonElement).disabled).toBe(false),
		);
		// témoin : au succès, la phrase de l'échec de lecture est absente
		expect(vue.queryByTestId('mcp-server-settings-load-failed')).toBeNull();
		cleanup();

		mockApi.getMcpServerSettings.mockRejectedValueOnce({
			code: 'INTERNAL_ERROR',
			status: 500,
			message: 'Erreur interne du serveur.',
		});
		const vue2 = render(McpSettingsSection);
		const erreur = await vue2.findByTestId('mcp-server-settings-load-error');
		expect(erreur.getAttribute('role')).toBe('alert');
		expect(erreur.textContent).toContain('Erreur interne du serveur.');
		expect(vue2.getByTestId('mcp-server-settings-load-failed').textContent).toContain(
			'Les réglages n’ont pas pu être lus : tant qu’ils ne le sont pas, rien ne s’enregistre depuis cet écran.',
		);
		expect((vue2.getByTestId('mcp-server-settings-save') as HTMLButtonElement).disabled).toBe(true);
		await fireEvent.click(vue2.getByTestId('mcp-server-settings-save'));
		expect(mockApi.updateMcpServerSettings).not.toHaveBeenCalled();
		cleanup();

		mockApi.getMcpServerSettings.mockRejectedValueOnce(new Error('réseau'));
		const vue3 = render(McpSettingsSection);
		expect((await vue3.findByTestId('mcp-server-settings-load-error')).textContent?.trim()).toBe('Erreur inattendue.');
	});

	it('(h) la phrase fixe sous Origines admises', async () => {
		const vue = await monter();
		expect(
			vue.getByText(
				'Une origine admise n’ouvre le serveur à aucune page web : elle ne sert qu’aux clients qui envoient un en-tête Origin sans être des navigateurs.',
			),
		).toBeTruthy();
	});
});

describe('McpSettingsSection — ce que le filtre ne garantit pas (AC-6)', () => {
	it('l’encadré et ses quatre phrases, dans l’état initial comme après échec de lecture', async () => {
		const vue = await monter();
		const texte = vue.getByTestId('mcp-server-guarantee').textContent ?? '';
		for (const phrase of ENCADRE) expect(texte).toContain(phrase);
		expect(texte).toContain('100.64.0.0/10');
		expect(texte).toContain('Traefik');
		expect(texte).toContain('Docker');
		cleanup();
		mockApi.getMcpServerSettings.mockRejectedValueOnce(new Error('x'));
		const vue2 = render(McpSettingsSection);
		await vue2.findByTestId('mcp-server-settings-load-error');
		const texte2 = vue2.getByTestId('mcp-server-guarantee').textContent ?? '';
		for (const phrase of ENCADRE) expect(texte2).toContain(phrase);
	});

	it('(a) filtre désactivé → « toute adresse est admise » ; témoin : absent filtre activé', async () => {
		const vue = await monter();
		expect(vue.queryByTestId('mcp-server-warning-filter-off')).toBeNull();
		await fireEvent.click(vue.getByTestId('mcp-server-settings-filter'));
		expect(vue.getByTestId('mcp-server-warning-filter-off').textContent).toContain('toute adresse est admise');
	});

	it('(b) une plage /0 nomme sa famille ; témoin : absent sans /0', async () => {
		const vue = await monter();
		expect(vue.queryAllByTestId('mcp-server-warning-network-any')).toHaveLength(0);
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), { target: { value: '0.0.0.0/0' } });
		let avis = vue.getAllByTestId('mcp-server-warning-network-any').map((e) => e.textContent ?? '');
		expect(avis).toHaveLength(1);
		expect(avis[0]).toContain('0.0.0.0/0 admet tout client IPv4');
		expect(avis[0]).not.toContain('IPv6');
		expect(avis[0]).toContain('pour admettre les deux familles, il faut les deux entrées');
		expect(avis[0]).not.toContain('toute adresse');
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), { target: { value: ' ::/0 ' } });
		avis = vue.getAllByTestId('mcp-server-warning-network-any').map((e) => e.textContent ?? '');
		expect(avis).toHaveLength(1);
		expect(avis[0]).toContain('::/0 admet tout client IPv6');
		expect(avis[0]).not.toContain('IPv4');
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), { target: { value: '192.168.1.10\n10.0.0.0/8' } });
		expect(vue.queryAllByTestId('mcp-server-warning-network-any')).toHaveLength(0);
	});

	it('(c) un mandataire /0, filtre activé, nomme sa famille ; absent filtre désactivé', async () => {
		const vue = await monter();
		expect(vue.queryAllByTestId('mcp-server-warning-proxy-any')).toHaveLength(0);
		await fireEvent.input(zone(vue, 'mcp-server-settings-proxies'), { target: { value: '0.0.0.0/0\n::/0' } });
		const avis = vue.getAllByTestId('mcp-server-warning-proxy-any').map((e) => e.textContent ?? '');
		expect(avis).toHaveLength(2);
		expect(avis[0]).toContain('0.0.0.0/0 : tout pair IPv4 est cru sur son en-tête X-Forwarded-For');
		expect(avis[0]).not.toContain('IPv6');
		expect(avis[1]).toContain('::/0 : tout pair IPv6 est cru sur son en-tête X-Forwarded-For');
		expect(avis[1]).not.toContain('IPv4');
		await fireEvent.click(vue.getByTestId('mcp-server-settings-filter'));
		expect(vue.queryAllByTestId('mcp-server-warning-proxy-any')).toHaveLength(0);
	});

	it('(d) filtre activé et aucune plage → « personne n’est admis » ; témoin', async () => {
		const vue = await monter();
		expect(vue.queryByTestId('mcp-server-warning-nobody')).toBeNull();
		await fireEvent.input(zone(vue, 'mcp-server-settings-networks'), { target: { value: '  \n' } });
		expect(vue.getByTestId('mcp-server-warning-nobody').textContent).toContain('personne n’est admis');
		await fireEvent.click(vue.getByTestId('mcp-server-settings-filter'));
		expect(vue.queryByTestId('mcp-server-warning-nobody')).toBeNull();
	});
});

describe('McpSettingsSection — activation, adresse, restauration, réinitialisation (AC-7)', () => {
	it('(a) suit la valeur ENREGISTRÉE, non la case cochée', async () => {
		const vue = await monter();
		expect(vue.getByTestId('mcp-server-state').textContent).toContain('/mcp répond comme une adresse qui n’existe pas');
		await fireEvent.click(vue.getByTestId('mcp-server-settings-enabled'));
		// case cochée, aucun envoi : la phrase reste celle du serveur désactivé
		expect(vue.getByTestId('mcp-server-state').textContent).toContain('/mcp répond comme une adresse qui n’existe pas');
		mockApi.updateMcpServerSettings.mockResolvedValueOnce(reglages({ enabled: true, version: 1 }));
		await fireEvent.click(vue.getByTestId('mcp-server-settings-save'));
		await waitFor(() =>
			expect(vue.getByTestId('mcp-server-state').textContent).toContain(
				'le serveur répond aux clés MCP de cette société, depuis les adresses admises',
			),
		);
	});

	it('(b) l’adresse du serveur, (c) la restauration, (d) la réinitialisation', async () => {
		const vue = await monter();
		expect(vue.getByTestId('mcp-server-address').textContent).toBe(new URL('/mcp', window.location.origin).href);
		expect(vue.getByTestId('mcp-server-address').tagName).toBe('CODE');
		expect(vue.getByTestId('mcp-server-address-note').textContent).toContain('exactement /mcp, sans barre finale');
		expect(vue.getByTestId('mcp-server-restore-note').textContent).toContain(
			'un serveur actif dans la sauvegarde l’est après l’import',
		);
		expect(vue.getByTestId('mcp-server-reset-note').textContent).toContain(
			'efface ces réglages (serveur désactivé) et conserve les clés',
		);
	});

	it('(b) sous une origine simulée : http://localhost:3000/mcp', async () => {
		const avant = window.location;
		Object.defineProperty(window, 'location', {
			configurable: true,
			value: { ...avant, origin: 'http://localhost:3000' },
		});
		try {
			const vue = await monter();
			expect(vue.getByTestId('mcp-server-address').textContent).toBe('http://localhost:3000/mcp');
		} finally {
			Object.defineProperty(window, 'location', { configurable: true, value: avant });
		}
	});
});
