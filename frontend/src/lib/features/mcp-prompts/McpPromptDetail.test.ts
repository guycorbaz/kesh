/**
 * Story 32-10b (refs #723) — le détail d'un prompt du serveur MCP (AC-4, AC-7 à
 * AC-10, AC-12, AC-14 ; amendement de l'AC-12 : la saisie survit au 409).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M1, M4, M6, M7, M10, M11, M12,
 * M15, M17, M18 de la fiche ; M19, la saisie vidée au rechargement).
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor, within } from '@testing-library/svelte';

vi.mock('./mcp-prompts.api', async (original) => ({
	...(await original<typeof import('./mcp-prompts.api')>()),
	getMcpPrompt: vi.fn(),
	addMcpPromptVersion: vi.fn(),
	approveMcpPromptVersion: vi.fn(),
	refuseMcpPromptVersion: vi.fn(),
	restoreMcpPromptStandard: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifySuccess: vi.fn() }));

import * as api from './mcp-prompts.api';
import McpPromptDetail from './McpPromptDetail.svelte';
import type { McpPromptDetail as Detail, McpPromptVersion as Version } from './mcp-prompts.types';

const mockApi = vi.mocked(api);

const tr = (s: string) => ({ exact: s, visible: s });

/** Une version de texte d'administrateur approuvée. */
function v(numero: number, over: Partial<Version> = {}): Version {
	return {
		numero,
		origine: 'admin',
		etat: 'approved',
		nature: 'text',
		description: tr(`description ${numero}`),
		texte: tr(`texte ${numero}`),
		servable: true,
		descriptionServie: tr(`description ${numero}`),
		texteServi: tr(`texte ${numero}`),
		auteurNom: 'admin',
		decideurNom: null,
		cleAuteurId: null,
		cleNom: null,
		cleRevoquee: null,
		creeLe: '2026-10-27T10:00:00',
		decideLe: null,
		...over,
	};
}

/** Une proposition d'agent à valider ; texte servi marqué. */
function agent(numero: number, over: Partial<Version> = {}): Version {
	const marque = `[Prompt écrit par un agent — version ${numero}]`;
	return v(numero, {
		origine: 'agent',
		etat: 'pending',
		auteurNom: null,
		cleAuteurId: 5,
		cleNom: 'agent de test',
		cleRevoquee: false,
		description: tr(`description agent ${numero}`),
		texte: tr(`texte agent ${numero}`),
		descriptionServie: tr(`${marque}\n\ndescription agent ${numero}`),
		texteServi: tr(`${marque}\n\ntexte agent ${numero}`),
		...over,
	});
}

const retrait = (numero: number, over: Partial<Version> = {}) =>
	v(numero, {
		nature: 'withdrawal',
		description: null,
		texte: null,
		servable: false,
		descriptionServie: null,
		texteServi: null,
		...over,
	});

const CODE = { description: tr('description du code'), texte: tr('texte du code\n\n  avec retrait') };

function detail(versions: Version[], over: Partial<Detail> = {}): Detail {
	return {
		id: 4,
		nom: 'mon-prompt',
		standard: false,
		version: 7,
		numeroEnVigueur: 1,
		standardDiffere: false,
		standardDuCode: null,
		modifieLe: '2026-10-27T11:00:00',
		versions,
		...over,
	};
}

const ECRITE = { promptId: 4, nom: 'mon-prompt', numero: 9, origine: 'admin', nature: 'text', etat: 'approved', version: 8 } as const;

async function monter(d: Detail) {
	mockApi.getMcpPrompt.mockResolvedValueOnce(d);
	const r = render(McpPromptDetail, { id: d.id });
	await r.findByTestId('mcp-prompt-title');
	return r;
}

/** Une promesse laissée en attente, et de quoi la libérer. */
function enAttente<T>() {
	let liberer: (v: T) => void = () => {};
	const p = new Promise<T>((res) => {
		liberer = res;
	});
	return { p, liberer };
}

beforeEach(() => {
	vi.clearAllMocks();
	// Une réponse « une fois » restée en file ne doit pas fuir dans le test suivant.
	mockApi.getMcpPrompt.mockReset();
	mockApi.addMcpPromptVersion.mockReset();
	mockApi.approveMcpPromptVersion.mockReset();
	mockApi.refuseMcpPromptVersion.mockReset();
	mockApi.restoreMcpPromptStandard.mockReset();
});
afterEach(() => cleanup());

describe('McpPromptDetail — le texte servi (AC-4)', () => {
	it('avis_permanent_exact — au-dessus du seul bloc servi', async () => {
		const r = await monter(detail([v(1)]));
		const avis = r.getByTestId('mcp-prompt-notice');
		expect(avis.getAttribute('role')).toBe('note');
		expect(avis.textContent?.trim()).toBe(
			'Le texte est montré tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme \\u{…}.',
		);
		expect(r.getByTestId('servi-mcp-prompt-served-text-1').textContent).toBe('texte 1');
		expect(r.queryAllByTestId('mcp-prompt-notice')).toHaveLength(1);
	});

	it('bloc_servi_seulement_si_servable — en vigueur non servable : la phrase, aucun élément servi-', async () => {
		const r = await monter(
			detail([v(1, { servable: false, descriptionServie: null, texteServi: null })]),
		);
		expect(r.getByTestId('mcp-prompt-not-served').textContent).toContain(
			'Ce prompt n’est servi à aucun agent',
		);
		expect(r.getByTestId('mcp-prompt-not-served-reason').textContent?.trim()).toBe(
			'Sa version en vigueur n’est pas valide : Kesh refuse de la servir (elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh).',
		);
		expect(r.container.querySelectorAll('[data-testid^="servi-"]')).toHaveLength(0);
		expect(r.getByTestId('non-servi-mcp-prompt-stored-text-1').textContent).toBe('texte 1');
		expect(r.queryByTestId('mcp-prompt-notice')).toBeNull();
	});

	it('retiré, ou aucune version approuvée : la phrase et sa raison', async () => {
		const r = await monter(detail([v(1), retrait(2)], { numeroEnVigueur: 2 }));
		expect(r.getByTestId('mcp-prompt-not-served-reason').textContent?.trim()).toBe('Il est retiré.');
		expect(r.queryByTestId('mcp-prompt-notice')).toBeNull();
		cleanup();
		const r2 = await monter(detail([agent(1)], { numeroEnVigueur: null }));
		expect(r2.getByTestId('mcp-prompt-not-served-reason').textContent?.trim()).toBe(
			'Aucune version n’est approuvée.',
		);
		expect(r2.getByTestId('mcp-prompt-header-in-force').textContent?.trim()).toBe('Non servi');
	});

	it('l’avis est absent d’une version refusée, d’une version à valider et de l’éditeur', async () => {
		const r = await monter(detail([v(1), agent(2, { etat: 'refused' }), agent(3)]));
		const avis = r.getAllByTestId('mcp-prompt-notice');
		expect(avis).toHaveLength(1);
		const bloc = r.getByTestId('mcp-prompt-served-block');
		expect(bloc.contains(avis[0])).toBe(true);
		expect(within(r.getByTestId('mcp-prompt-pending')).queryByTestId('mcp-prompt-notice')).toBeNull();
		expect(within(r.getByTestId('mcp-prompt-editor')).queryByTestId('mcp-prompt-notice')).toBeNull();
	});
});

describe('McpPromptDetail — la décision (AC-7)', () => {
	// M15
	it('version_a_valider_jamais_repliee', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		const pre = r.getByTestId('a-valider-mcp-prompt-served-text-2');
		expect(pre.textContent).toBe('[Prompt écrit par un agent — version 2]\n\ntexte agent 2');
		expect(pre.closest('details')).toBeNull();
		expect(pre.closest('[hidden]')).toBeNull();
		expect(r.queryByText(/voir plus/i)).toBeNull();
		expect(r.getByTestId('mcp-prompt-pending-key-2').textContent).toContain('clé n° 5 (agent de test)');
	});

	// M1
	it('approbation_porte_la_version_lue — un seul GET avant le POST', async () => {
		const r = await monter(detail([v(1), v(2), agent(3)], { version: 7 }));
		// Le nombre de GET au moment du POST est RELEVÉ, non asserté dans la simulation :
		// une assertion levée ici serait avalée par le `catch` de l'écran (M1 survivait).
		let getAuPost = -1;
		mockApi.approveMcpPromptVersion.mockImplementationOnce(async () => {
			getAuPost = mockApi.getMcpPrompt.mock.calls.length;
			return { ...ECRITE, numero: 3 };
		});
		// Toute relecture rend une `version` différente : une relecture avant le POST se verrait.
		// (Deux réponses « une fois » : `clearAllMocks` ne retire pas une réponse permanente.)
		const relu = detail([v(1), v(2), agent(3, { etat: 'approved' })], { version: 8, numeroEnVigueur: 3 });
		mockApi.getMcpPrompt.mockResolvedValueOnce(relu).mockResolvedValueOnce(relu);
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-3'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-approve-confirm'));
		await waitFor(() => expect(mockApi.approveMcpPromptVersion).toHaveBeenCalledTimes(1));
		expect(mockApi.approveMcpPromptVersion).toHaveBeenCalledWith(4, 3, 7);
		expect(getAuPost, 'un seul GET avant le POST').toBe(1);
		// La relecture vient APRÈS la décision.
		await waitFor(() => expect(mockApi.getMcpPrompt).toHaveBeenCalledTimes(2));
		await r.findByTestId('mcp-prompt-in-force-badge-3');
	});

	// M7
	it('approbation_apres_confirmation — sans confirmation, aucun POST', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-2'));
		await r.findByTestId('mcp-prompt-approve-dialog');
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-cancel'));
		expect(mockApi.approveMcpPromptVersion).not.toHaveBeenCalled();
	});

	it('dialogue_restate_numero_cle_et_marque', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-2'));
		const dialogue = await r.findByTestId('mcp-prompt-approve-dialog');
		const texte = dialogue.textContent?.replace(/\s+/g, ' ') ?? '';
		expect(texte).toContain('mon-prompt');
		expect(texte).toContain('version n° 2, proposée par la clé n° 5 (agent de test)');
		expect(texte).toContain(
			'Une fois approuvé, ce texte sera servi à tout agent dont la clé porte books:read, précédé de la marque.',
		);
		expect(within(dialogue).getByTestId('mcp-prompt-approve-confirm').textContent?.trim()).toBe(
			'Approuver la version n° 2',
		);
	});

	// M6
	it('approuver_seulement_la_derniere — n° 3 à valider, n° 4 standard', async () => {
		const r = await monter(
			detail([v(1), v(2), agent(3), v(4, { origine: 'standard' })], { numeroEnVigueur: 4 }),
		);
		expect(r.queryByTestId('mcp-prompt-approve-3')).toBeNull();
		expect(r.getByTestId('mcp-prompt-refuse-3')).toBeTruthy();
		expect(r.getByTestId('mcp-prompt-not-latest-3').textContent?.trim()).toBe(
			'Une version plus récente existe : celle-ci ne peut plus être approuvée, seulement refusée.',
		);
	});

	// M4
	it('pas_d_approbation_d_une_version_non_servable', async () => {
		const r = await monter(
			detail([v(1), agent(2, { servable: false, descriptionServie: null, texteServi: null })]),
		);
		expect(r.queryByTestId('mcp-prompt-approve-2')).toBeNull();
		expect(r.getByTestId('mcp-prompt-refuse-2')).toBeTruthy();
		expect(r.getByTestId('a-valider-mcp-prompt-not-servable-2')).toBeTruthy();
	});

	it('aucun_bouton_sur_une_version_decidee', async () => {
		const r = await monter(
			detail([v(1), agent(2, { etat: 'refused' }), agent(3, { etat: 'approved' })], { numeroEnVigueur: 3 }),
		);
		for (const n of [1, 2, 3]) {
			expect(r.queryByTestId(`mcp-prompt-approve-${n}`)).toBeNull();
			expect(r.queryByTestId(`mcp-prompt-refuse-${n}`)).toBeNull();
		}
	});

	// M11
	it('refus_apres_confirmation — n° 3 à valider, n° 4 dernière : …/versions/3/refuse', async () => {
		const r = await monter(detail([v(1), v(2), agent(3), v(4)], { numeroEnVigueur: 4, version: 9 }));
		mockApi.refuseMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 3 });
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), v(2), agent(3, { etat: 'refused' }), v(4)], { numeroEnVigueur: 4 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-3'));
		const dialogue = await r.findByTestId('mcp-prompt-refuse-dialog');
		expect(dialogue.textContent).toContain('Un refus est définitif ; la version reste à l’historique.');
		expect(mockApi.refuseMcpPromptVersion).not.toHaveBeenCalled();
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-confirm'));
		await waitFor(() => expect(mockApi.refuseMcpPromptVersion).toHaveBeenCalledWith(4, 3, 9));
	});
});

describe('McpPromptDetail — l’historique (AC-8)', () => {
	it('chaque origine, état et nature ; la plus récente en tête', async () => {
		const r = await monter(
			detail(
				[
					v(1, { origine: 'standard', auteurNom: null }),
					v(2, { auteurNom: 'alice' }),
					agent(3, { etat: 'approved', decideurNom: 'alice', decideLe: '2026-10-27T12:00:00' }),
					agent(4, { etat: 'refused', decideurNom: 'bob', decideLe: '2026-10-27T13:00:00' }),
					retrait(5),
				],
				{ numeroEnVigueur: 5 },
			),
		);
		const lignes = r.getAllByTestId(/^mcp-prompt-history-row-/);
		expect(lignes.map((l) => l.getAttribute('data-testid'))).toEqual([
			'mcp-prompt-history-row-5',
			'mcp-prompt-history-row-4',
			'mcp-prompt-history-row-3',
			'mcp-prompt-history-row-2',
			'mcp-prompt-history-row-1',
		]);
		expect(r.getByTestId('mcp-prompt-history-origin-1').textContent?.trim()).toBe('Standard');
		expect(r.getByTestId('mcp-prompt-history-origin-2').textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Administrateur — alice',
		);
		expect(r.getByTestId('mcp-prompt-history-origin-3').textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Agent — clé n° 5',
		);
		expect(r.getByTestId('mcp-prompt-history-state-3').textContent?.trim()).toBe('Approuvée');
		expect(r.getByTestId('mcp-prompt-history-state-4').textContent?.trim()).toBe('Refusée');
		expect(r.getByTestId('mcp-prompt-history-nature-5').textContent?.trim()).toBe('Retrait');
		expect(r.getByTestId('mcp-prompt-history-nature-2').textContent?.trim()).toBe('Texte');
		expect(r.getByTestId('mcp-prompt-history-decision-4').textContent).toContain('bob');
		expect(r.queryByTestId('mcp-prompt-history-decision-2')).toBeNull();
	});

	it('marque_visible_dans_l_historique — la pastille Agent sur chaque version d’agent', async () => {
		const r = await monter(
			detail([v(1), agent(2, { etat: 'refused' }), agent(3, { etat: 'approved' }), agent(4)], {
				numeroEnVigueur: 3,
			}),
		);
		for (const n of [2, 3, 4]) {
			const cellule = r.getByTestId(`mcp-prompt-history-origin-${n}`);
			expect(cellule.querySelector('.pastille-avertissement'), String(n)).not.toBeNull();
		}
		expect(r.getByTestId('mcp-prompt-history-origin-1').querySelector('.pastille-avertissement')).toBeNull();
	});

	it('en_vigueur_selon_le_serveur — la pastille sur numeroEnVigueur et sur elle seule', async () => {
		const r = await monter(
			detail([v(1), retrait(2), agent(3), agent(4, { etat: 'refused' })], { numeroEnVigueur: 2 }),
		);
		expect(r.getAllByTestId(/^mcp-prompt-in-force-badge-/).map((e) => e.getAttribute('data-testid'))).toEqual([
			'mcp-prompt-in-force-badge-2',
		]);
		cleanup();
		const r2 = await monter(detail([agent(1)], { numeroEnVigueur: null }));
		expect(r2.queryAllByTestId(/^mcp-prompt-in-force-badge-/)).toHaveLength(0);
	});

	it('une version de texte se déplie ; Reprendre cette version pré-remplit l’éditeur (texte ENREGISTRÉ)', async () => {
		const r = await monter(detail([v(1), agent(2, { etat: 'refused' })]));
		const details = r.getByTestId('mcp-prompt-history-details-2');
		expect(details.tagName).toBe('DETAILS');
		expect(within(details).getByTestId('historique-mcp-prompt-served-text-2')).toBeTruthy();
		await fireEvent.click(r.getByTestId('mcp-prompt-history-reuse-2'));
		expect((r.getByTestId('mcp-prompt-editor-description') as HTMLInputElement).value).toBe('description agent 2');
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte agent 2');
		expect(r.getByTestId('mcp-prompt-editor-agent-origin')).toBeTruthy();
		// Revue P1, E7 : l'assertion précédente visait un identifiant inexistant. Reprendre
		// la v1 (d'administrateur) remet son texte et retire l'avis P26.
		await fireEvent.click(r.getByTestId('mcp-prompt-history-reuse-1'));
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte 1');
		expect(r.queryByTestId('mcp-prompt-editor-agent-origin')).toBeNull();
	});
});

describe('McpPromptDetail — l’éditeur, le retrait, le standard (AC-9, AC-10)', () => {
	it('pré-rempli d’office par la version en vigueur, texte ENREGISTRÉ ; avis P26 si d’agent', async () => {
		const r = await monter(detail([v(1), agent(2, { etat: 'approved' })], { numeroEnVigueur: 2 }));
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte agent 2');
		expect(r.getByTestId('mcp-prompt-editor-agent-origin')).toBeTruthy();
		cleanup();
		const r2 = await monter(detail([v(1)]));
		expect((r2.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte 1');
		expect(r2.queryByTestId('mcp-prompt-editor-agent-origin')).toBeNull();
	});

	it('enregistrer : corpsTexte(version lue, description, texte), puis relecture', async () => {
		const r = await monter(detail([v(1)], { version: 7 }));
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		t.value = ' nouveau\ttexte\n\n';
		await fireEvent.input(t);
		mockApi.addMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 2 });
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), v(2)], { numeroEnVigueur: 2, version: 8 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		await waitFor(() =>
			expect(mockApi.addMcpPromptVersion).toHaveBeenCalledWith(4, {
				version: 7,
				nature: 'text',
				description: 'description 1',
				texte: ' nouveau\ttexte\n\n',
			}),
		);
		await waitFor(() => expect(mockApi.getMcpPrompt).toHaveBeenCalledTimes(2));
	});

	it('editeur_reinitialise_apres_relecture — après l’approbation de la v4 d’agent', async () => {
		const r = await monter(detail([v(1), v(3), agent(4)], { numeroEnVigueur: 3 }));
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		expect(t.value).toBe('texte 3');
		t.value = 'modifié';
		await fireEvent.input(t);
		mockApi.approveMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 4 });
		mockApi.getMcpPrompt.mockResolvedValueOnce(
			detail([v(1), v(3), agent(4, { etat: 'approved' })], { numeroEnVigueur: 4, version: 8 }),
		);
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-4'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-approve-confirm'));
		await waitFor(() =>
			expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte agent 4'),
		);
		expect((r.getByTestId('mcp-prompt-editor-description') as HTMLInputElement).value).toBe('description agent 4');
		expect(r.getByTestId('mcp-prompt-editor-agent-origin')).toBeTruthy();
	});

	it('retrait_apres_confirmation — corpsRetrait(version lue)', async () => {
		const r = await monter(detail([v(1)], { version: 7 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-withdraw'));
		const dialogue = await r.findByTestId('mcp-prompt-withdraw-dialog');
		expect(dialogue.textContent?.replace(/\s+/g, ' ')).toContain(
			'Le prompt cessera d’être servi. Une version n° 2 de retrait est ajoutée ; l’historique reste. Une nouvelle version le servira de nouveau.',
		);
		expect(mockApi.addMcpPromptVersion).not.toHaveBeenCalled();
		mockApi.addMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 2, nature: 'withdrawal' });
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), retrait(2)], { numeroEnVigueur: 2 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-withdraw-confirm'));
		await waitFor(() =>
			expect(mockApi.addMcpPromptVersion).toHaveBeenCalledWith(4, { version: 7, nature: 'withdrawal' }),
		);
	});

	it('Retirer absent si la version en vigueur est déjà un retrait', async () => {
		const r = await monter(detail([v(1), retrait(2)], { numeroEnVigueur: 2 }));
		expect(r.queryByTestId('mcp-prompt-withdraw')).toBeNull();
	});

	// M10
	it('retour_au_standard_seulement_si_utile', async () => {
		const std = { standard: true, nom: 'kesh-assistant' };
		const presents: Detail[] = [
			detail([v(1, { origine: 'standard' }), v(2)], { ...std, numeroEnVigueur: 2, standardDiffere: true, standardDuCode: CODE }),
			detail([v(1, { origine: 'standard' }), retrait(2)], { ...std, numeroEnVigueur: 2, standardDiffere: true, standardDuCode: CODE }),
			detail([v(1, { origine: 'standard' })], { ...std, standardDiffere: true, standardDuCode: CODE }),
		];
		for (const d of presents) {
			const r = await monter(d);
			expect(r.getByTestId('mcp-prompt-restore-standard')).toBeTruthy();
			cleanup();
		}
		const absents: Detail[] = [
			detail([v(1)]),
			detail([v(1, { origine: 'standard' })], { ...std, standardDiffere: false, standardDuCode: CODE }),
			detail([v(1, { origine: 'standard' })], { ...std, standardDiffere: true, standardDuCode: null }),
		];
		for (const d of absents) {
			const r = await monter(d);
			expect(r.queryByTestId('mcp-prompt-restore-standard')).toBeNull();
			cleanup();
		}
	});

	// M12
	it('dialogue_standard_montre_le_texte_du_code — avant tout POST', async () => {
		const r = await monter(
			detail([v(1, { origine: 'standard' }), v(2)], {
				standard: true,
				nom: 'kesh-assistant',
				numeroEnVigueur: 2,
				standardDiffere: true,
				standardDuCode: CODE,
				version: 6,
			}),
		);
		await fireEvent.click(r.getByTestId('mcp-prompt-restore-standard'));
		await r.findByTestId('mcp-prompt-standard-dialog');
		expect(r.getByTestId('mcp-prompt-standard-text').textContent).toBe(CODE.texte.visible);
		expect(r.getByTestId('mcp-prompt-standard-description').textContent).toBe(CODE.description.visible);
		expect(mockApi.restoreMcpPromptStandard).not.toHaveBeenCalled();
		expect(r.getByTestId('mcp-prompt-standard-dialog').textContent?.replace(/\s+/g, ' ')).toContain(
			'Une nouvelle version n° 3 reprend ce texte, le standard de cette version de Kesh.',
		);
	});

	it('retour_au_standard_apres_confirmation — corpsStandard(version lue)', async () => {
		const r = await monter(
			detail([v(1, { origine: 'standard' }), v(2)], {
				standard: true,
				numeroEnVigueur: 2,
				standardDiffere: true,
				standardDuCode: CODE,
				version: 6,
			}),
		);
		mockApi.restoreMcpPromptStandard.mockResolvedValueOnce({ ...ECRITE, numero: 3, origine: 'standard' });
		mockApi.getMcpPrompt.mockResolvedValueOnce(
			detail([v(1, { origine: 'standard' }), v(2), v(3, { origine: 'standard' })], { numeroEnVigueur: 3 }),
		);
		await fireEvent.click(r.getByTestId('mcp-prompt-restore-standard'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-standard-confirm'));
		await waitFor(() => expect(mockApi.restoreMcpPromptStandard).toHaveBeenCalledWith(4, 6));
	});
});

describe('McpPromptDetail — les erreurs (AC-12)', () => {
	it('409 verrou périmé : le message de l’écran, Recharger ; aucune_relance_automatique_sur_409', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		mockApi.approveMcpPromptVersion.mockRejectedValueOnce({
			status: 409,
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'x',
		});
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-approve-confirm'));
		const erreur = await r.findByTestId('mcp-prompt-error');
		expect(erreur.textContent).toContain('Ce prompt a changé depuis son affichage : rechargez-le avant d’agir.');
		expect(r.getByTestId('mcp-prompt-reload')).toBeTruthy();
		await new Promise((res) => setTimeout(res, 20));
		expect(mockApi.approveMcpPromptVersion).toHaveBeenCalledTimes(1);
		expect(mockApi.getMcpPrompt).toHaveBeenCalledTimes(1);
	});

	it('409 d’état : le message du serveur', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		mockApi.refuseMcpPromptVersion.mockRejectedValueOnce({
			status: 409,
			code: 'ILLEGAL_STATE_TRANSITION',
			message: 'Cette version a déjà été approuvée ou refusée.',
		});
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-refuse-confirm'));
		expect((await r.findByTestId('mcp-prompt-error')).textContent).toContain(
			'Cette version a déjà été approuvée ou refusée.',
		);
	});

	it('400 : le message du serveur, saisie conservée', async () => {
		const r = await monter(detail([v(1)]));
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		t.value = 'avec ‮ caché';
		await fireEvent.input(t);
		mockApi.addMcpPromptVersion.mockRejectedValueOnce({
			status: 400,
			code: 'VALIDATION_ERROR',
			message: 'Le texte du prompt contient un caractère invisible.',
		});
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		expect((await r.findByTestId('mcp-prompt-error')).textContent).toContain(
			'Le texte du prompt contient un caractère invisible.',
		);
		expect(r.queryByTestId('mcp-prompt-reload')).toBeNull();
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('avec ‮ caché');
	});

	it('404 au chargement : Prompt introuvable ; 403 : Réservé à l’Administrateur', async () => {
		mockApi.getMcpPrompt.mockRejectedValueOnce({ status: 404, code: 'NOT_FOUND', message: 'x' });
		const r = render(McpPromptDetail, { id: 4 });
		expect((await r.findByTestId('mcp-prompt-load-error')).textContent?.trim()).toBe('Prompt introuvable.');
		cleanup();
		mockApi.getMcpPrompt.mockRejectedValueOnce({ status: 403, code: 'FORBIDDEN', message: 'x' });
		const r2 = render(McpPromptDetail, { id: 4 });
		expect((await r2.findByTestId('mcp-prompt-load-error')).textContent?.trim()).toBe(
			'Réservé à l’Administrateur.',
		);
	});

	it('un numéro illisible : introuvable, sans appel', async () => {
		const r = render(McpPromptDetail, { id: Number.NaN });
		expect((await r.findByTestId('mcp-prompt-load-error')).textContent?.trim()).toBe('Prompt introuvable.');
		expect(mockApi.getMcpPrompt).not.toHaveBeenCalled();
	});

	// Amendement de l'AC-12 (KF-055, #796) — M19 : la saisie vidée au rechargement.
	it('saisie_conservee_apres_409 — Recharger relit le détail et garde la saisie ; aucun envoi automatique', async () => {
		const r = await monter(detail([v(1)], { version: 7 }));
		const d = r.getByTestId('mcp-prompt-editor-description') as HTMLInputElement;
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		d.value = 'ma description';
		await fireEvent.input(d);
		t.value = 'ma saisie\nsur deux lignes';
		await fireEvent.input(t);
		mockApi.addMcpPromptVersion.mockRejectedValueOnce({
			status: 409,
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'x',
		});
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		await r.findByTestId('mcp-prompt-error');
		// Une proposition d'agent est arrivée entre-temps : la v2, et la `version` 8.
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), agent(2)], { version: 8 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		await r.findByTestId('a-valider-mcp-prompt-served-text-2');
		expect(d.isConnected ? d.value : (r.getByTestId('mcp-prompt-editor-description') as HTMLInputElement).value).toBe(
			'ma description',
		);
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe(
			'ma saisie\nsur deux lignes',
		);
		expect(r.getByTestId('mcp-prompt-editor-kept').textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Votre saisie est conservée dans l’éditeur. Relisez l’historique rechargé avant de l’enregistrer de nouveau.',
		);
		expect(mockApi.addMcpPromptVersion).toHaveBeenCalledTimes(1);
		// L'Administrateur réenregistre lui-même : la `version` relue part.
		mockApi.addMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 3 });
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), agent(2), v(3)], { numeroEnVigueur: 3, version: 9 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		await waitFor(() =>
			expect(mockApi.addMcpPromptVersion).toHaveBeenLastCalledWith(4, {
				version: 8,
				nature: 'text',
				description: 'ma description',
				texte: 'ma saisie\nsur deux lignes',
			}),
		);
	});

	it('un 409 d’un geste sans saisie : Recharger remet l’éditeur sur la version en vigueur', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		t.value = 'brouillon';
		await fireEvent.input(t);
		mockApi.refuseMcpPromptVersion.mockRejectedValueOnce({ status: 409, code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x' });
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-refuse-confirm'));
		await r.findByTestId('mcp-prompt-error');
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), agent(2)], { version: 8 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		await waitFor(() =>
			expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('texte 1'),
		);
		expect(r.queryByTestId('mcp-prompt-editor-kept')).toBeNull();
	});
});

describe('McpPromptDetail — un seul envoi à la fois (AC-7, C-32-10b-18)', () => {
	/** Les boutons de geste visibles de la page, hors dialogue. */
	function gestes(r: Awaited<ReturnType<typeof monter>>): HTMLButtonElement[] {
		return [
			...r.container.querySelectorAll<HTMLButtonElement>(
				'[data-testid^="mcp-prompt-approve-"], [data-testid^="mcp-prompt-refuse-"], [data-testid="mcp-prompt-withdraw"], [data-testid="mcp-prompt-restore-standard"], [data-testid="mcp-prompt-editor-save"], [data-testid^="mcp-prompt-history-reuse-"]',
			),
		].filter((b) => b.tagName === 'BUTTON');
	}

	const STANDARD_PERSO = () =>
		detail([v(1, { origine: 'standard' }), v(2), agent(3)], {
			standard: true,
			nom: 'kesh-assistant',
			numeroEnVigueur: 2,
			standardDiffere: true,
			standardDuCode: CODE,
		});

	type Cas = {
		nom: string;
		ouvrir?: string;
		confirmer: string;
		appel: () => ReturnType<typeof vi.fn>;
	};
	const CAS: Cas[] = [
		{ nom: 'Approuver', ouvrir: 'mcp-prompt-approve-3', confirmer: 'mcp-prompt-approve-confirm', appel: () => mockApi.approveMcpPromptVersion as unknown as ReturnType<typeof vi.fn> },
		{ nom: 'Refuser', ouvrir: 'mcp-prompt-refuse-3', confirmer: 'mcp-prompt-refuse-confirm', appel: () => mockApi.refuseMcpPromptVersion as unknown as ReturnType<typeof vi.fn> },
		{ nom: 'Enregistrer', confirmer: 'mcp-prompt-editor-save', appel: () => mockApi.addMcpPromptVersion as unknown as ReturnType<typeof vi.fn> },
		{ nom: 'Retirer', ouvrir: 'mcp-prompt-withdraw', confirmer: 'mcp-prompt-withdraw-confirm', appel: () => mockApi.addMcpPromptVersion as unknown as ReturnType<typeof vi.fn> },
		{ nom: 'Revenir au standard', ouvrir: 'mcp-prompt-restore-standard', confirmer: 'mcp-prompt-standard-confirm', appel: () => mockApi.restoreMcpPromptStandard as unknown as ReturnType<typeof vi.fn> },
	];

	// M17, M18
	for (const c of CAS) {
		it(`gestes_desactives_pendant_l_envoi — ${c.nom}`, async () => {
			const r = await monter(STANDARD_PERSO());
			const attente = enAttente<unknown>();
			c.appel().mockReturnValueOnce(attente.p);
			if (c.ouvrir) {
				await fireEvent.click(r.getByTestId(c.ouvrir));
			}
			const bouton = (await r.findByTestId(c.confirmer)) as HTMLButtonElement;
			await fireEvent.click(bouton);
			await waitFor(() => expect(bouton.disabled).toBe(true));
			expect(bouton.getAttribute('aria-busy')).toBe('true');
			const tous = gestes(r);
			expect(tous.length).toBeGreaterThanOrEqual(5);
			for (const b of tous) expect(b.disabled, b.getAttribute('data-testid') ?? '').toBe(true);
			// Deux clics de plus sur le bouton désactivé : Svelte appelle encore le
			// gestionnaire délégué ; le retour anticipé seul empêche un second POST.
			await fireEvent.click(bouton);
			await fireEvent.click(bouton);
			expect(c.appel()).toHaveBeenCalledTimes(1);
			attente.liberer({ ...ECRITE });
		});
	}
});

describe('McpPromptDetail — remédiation de la revue de code P1', () => {
	const PANNE = { status: 500, code: 'INTERNAL_ERROR', message: 'Panne du serveur.' };

	// M20 (B1 = E1 = A1) : une écriture réussie suivie d'une relecture en échec.
	it('relecture_en_echec_apres_ecriture — le succès est dit, Recharger offert, jamais un échec du geste', async () => {
		const { notifySuccess } = await import('$lib/shared/utils/notify');
		const r = await monter(detail([v(1), agent(2)], { version: 7 }));
		mockApi.approveMcpPromptVersion.mockResolvedValueOnce({ ...ECRITE, numero: 2 });
		mockApi.getMcpPrompt.mockRejectedValueOnce(PANNE);
		await fireEvent.click(r.getByTestId('mcp-prompt-approve-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-approve-confirm'));
		const erreur = await r.findByTestId('mcp-prompt-error');
		expect(erreur.textContent).toContain(
			'Le geste a été enregistré, mais la relecture du prompt a échoué : rechargez-le.',
		);
		expect(erreur.textContent).not.toContain('Panne du serveur.');
		expect(vi.mocked(notifySuccess)).toHaveBeenCalledWith('Version n° 2 approuvée.');
		expect(r.getByTestId('mcp-prompt-reload')).toBeTruthy();
		// Recharger relit, et l'écran montre l'état du serveur.
		mockApi.getMcpPrompt.mockResolvedValueOnce(
			detail([v(1), agent(2, { etat: 'approved' })], { version: 8, numeroEnVigueur: 2 }),
		);
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		await r.findByTestId('mcp-prompt-in-force-badge-2');
		expect(r.queryByTestId('mcp-prompt-error')).toBeNull();
		expect(mockApi.approveMcpPromptVersion).toHaveBeenCalledTimes(1);
	});

	// M21 (E2 = B2 = A2) : l'amendement de l'AC-12 hors du chemin nominal.
	it('saisie_conservee_si_le_rechargement_echoue — le détail reste, la saisie aussi, Recharger encore offert', async () => {
		const r = await monter(detail([v(1)], { version: 7 }));
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		t.value = 'ma saisie précieuse';
		await fireEvent.input(t);
		mockApi.addMcpPromptVersion.mockRejectedValueOnce({ status: 409, code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x' });
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		await r.findByTestId('mcp-prompt-reload');
		// Le rechargement échoue : rien ne disparaît.
		mockApi.getMcpPrompt.mockRejectedValueOnce(PANNE);
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		await waitFor(() => expect(r.getByTestId('mcp-prompt-error').textContent).toContain('Panne du serveur.'));
		expect(r.queryByTestId('mcp-prompt-load-error')).toBeNull();
		expect(r.getByTestId('mcp-prompt-title')).toBeTruthy();
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('ma saisie précieuse');
		// On réessaie : le rechargement réussit, la saisie est toujours gardée.
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1), agent(2)], { version: 8 }));
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		await r.findByTestId('mcp-prompt-editor-kept');
		expect((r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement).value).toBe('ma saisie précieuse');
		expect(mockApi.addMcpPromptVersion).toHaveBeenCalledTimes(1);
	});

	it('premier chargement en échec : un bouton pour réessayer', async () => {
		mockApi.getMcpPrompt.mockRejectedValueOnce(PANNE);
		const r = render(McpPromptDetail, { id: 4 });
		await r.findByTestId('mcp-prompt-load-error');
		mockApi.getMcpPrompt.mockResolvedValueOnce(detail([v(1)]));
		await fireEvent.click(r.getByTestId('mcp-prompt-load-retry'));
		await r.findByTestId('mcp-prompt-title');
	});

	// M22 (E3) : l'erreur d'un geste de l'éditeur, là où l'on agit.
	it('erreur_de_l_editeur_pres_de_l_editeur — 400 et 409, focalisée', async () => {
		const r = await monter(detail([v(1)]));
		mockApi.addMcpPromptVersion.mockRejectedValueOnce({
			status: 400,
			code: 'VALIDATION_ERROR',
			message: 'Le texte du prompt contient un caractère invisible.',
		});
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		const erreur = await r.findByTestId('mcp-prompt-error');
		const zone = r.getByTestId('mcp-prompt-editor-zone');
		expect(zone.contains(erreur)).toBe(true);
		expect(r.getAllByTestId('mcp-prompt-error')).toHaveLength(1);
		await waitFor(() => expect(document.activeElement).toBe(erreur));
		// Un geste hors de l'éditeur garde son erreur en tête de page.
		cleanup();
		const r2 = await monter(detail([v(1), agent(2)]));
		mockApi.refuseMcpPromptVersion.mockRejectedValueOnce({ status: 409, code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x' });
		await fireEvent.click(r2.getByTestId('mcp-prompt-refuse-2'));
		await fireEvent.click(await r2.findByTestId('mcp-prompt-refuse-confirm'));
		const e2 = await r2.findByTestId('mcp-prompt-error');
		expect(r2.getByTestId('mcp-prompt-editor-zone').contains(e2)).toBe(false);
		await waitFor(() => expect(document.activeElement).toBe(e2));
	});

	// M23 (B3 = E8) : pendant un rechargement, aucun geste.
	it('gestes_desactives_pendant_un_rechargement', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		mockApi.refuseMcpPromptVersion.mockRejectedValueOnce({ status: 409, code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x' });
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-refuse-confirm'));
		await r.findByTestId('mcp-prompt-reload');
		const attente = enAttente<Detail>();
		mockApi.getMcpPrompt.mockReturnValueOnce(attente.p);
		await fireEvent.click(r.getByTestId('mcp-prompt-reload'));
		for (const id of ['mcp-prompt-approve-2', 'mcp-prompt-refuse-2', 'mcp-prompt-editor-save', 'mcp-prompt-withdraw']) {
			await waitFor(() => expect((r.getByTestId(id) as HTMLButtonElement).disabled, id).toBe(true));
		}
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		expect(mockApi.addMcpPromptVersion).not.toHaveBeenCalled();
		attente.liberer(detail([v(1), agent(2)], { version: 8 }));
		await waitFor(() => expect((r.getByTestId('mcp-prompt-editor-save') as HTMLButtonElement).disabled).toBe(false));
	});

	// M24 (B5 = E6) : un dialogue ne se ferme pas pendant l'envoi.
	it('dialogue_ne_se_ferme_pas_pendant_l_envoi — Échap et croix', async () => {
		const r = await monter(detail([v(1), agent(2)]));
		// Témoin : hors envoi, Échap ferme le dialogue (sans quoi le test serait vide).
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-2'));
		const temoin = await r.findByTestId('mcp-prompt-refuse-dialog');
		expect(temoin.querySelector('[data-slot="dialog-close"]')).not.toBeNull();
		await fireEvent.keyDown(document.activeElement ?? temoin, { key: 'Escape' });
		await waitFor(() => expect(r.queryByTestId('mcp-prompt-refuse-dialog')).toBeNull());
		// Pendant l'envoi : Échap ne ferme rien, la croix n'est plus offerte.
		const attente = enAttente<unknown>();
		mockApi.refuseMcpPromptVersion.mockReturnValueOnce(attente.p as ReturnType<typeof api.refuseMcpPromptVersion>);
		await fireEvent.click(r.getByTestId('mcp-prompt-refuse-2'));
		await fireEvent.click(await r.findByTestId('mcp-prompt-refuse-confirm'));
		await waitFor(() =>
			expect((r.getByTestId('mcp-prompt-refuse-confirm') as HTMLButtonElement).disabled).toBe(true),
		);
		const dialogue = r.getByTestId('mcp-prompt-refuse-dialog');
		expect(dialogue.querySelector('[data-slot="dialog-close"]')).toBeNull();
		await fireEvent.keyDown(document.activeElement ?? dialogue, { key: 'Escape' });
		await new Promise((res) => setTimeout(res, 20));
		expect(r.getByTestId('mcp-prompt-refuse-dialog')).toBeTruthy();
		attente.liberer({ ...ECRITE });
	});
});
