/**
 * Story 32-10b (refs #723) — les fonctions pures de l'écran des prompts (AC-3,
 * AC-7, AC-10, AC-12).
 *
 * L'écran **reflète** les règles du serveur (AC-i6, AC-i7 de la 32-10a) sans les
 * refaire : la version en vigueur est celle que nomme `numeroEnVigueur`, rendu
 * par le serveur ; le verdict `servable` est le sien.
 */
import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
	i18nLocale: () => 'fr-CH',
}));

import {
	derniereVersion,
	erreurDeGeste,
	etiquetteStandard,
	peutApprouver,
	peutRefuser,
	peutRevenirAuStandard,
	versionEnVigueur,
} from './mcp-prompts-format';
import type { McpPromptDetail, McpPromptSummary, McpPromptVersion } from './mcp-prompts.types';

function version(over: Partial<McpPromptVersion> = {}): McpPromptVersion {
	return {
		numero: 1,
		origine: 'admin',
		etat: 'approved',
		nature: 'text',
		description: { exact: 'd', visible: 'd' },
		texte: { exact: 't', visible: 't' },
		servable: true,
		descriptionServie: { exact: 'd', visible: 'd' },
		texteServi: { exact: 't', visible: 't' },
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

function detail(versions: McpPromptVersion[], over: Partial<McpPromptDetail> = {}): McpPromptDetail {
	return {
		id: 4,
		nom: 'mon-prompt',
		standard: false,
		version: 7,
		numeroEnVigueur: 1,
		standardDiffere: false,
		standardDuCode: null,
		modifieLe: '2026-10-27T10:00:00',
		versions,
		...over,
	};
}

function resume(over: Partial<McpPromptSummary> = {}): McpPromptSummary {
	return {
		id: 1,
		nom: 'kesh-assistant',
		standard: true,
		version: 1,
		derniere: { numero: 1, origine: 'standard', etat: 'approved', nature: 'text' },
		enVigueur: { numero: 1, origine: 'standard', etat: 'approved', nature: 'text' },
		aValider: 0,
		standardDiffere: false,
		...over,
	};
}

const agentEnAttente = (numero: number, over: Partial<McpPromptVersion> = {}) =>
	version({ numero, origine: 'agent', etat: 'pending', cleAuteurId: 5, cleNom: 'agent', ...over });

describe('mcp-prompts-format', () => {
	it('versionEnVigueur : celle que nomme le serveur, nulle sinon', () => {
		const d = detail([version({ numero: 1 }), agentEnAttente(2)], { numeroEnVigueur: 1 });
		expect(versionEnVigueur(d)?.numero).toBe(1);
		expect(versionEnVigueur({ ...d, numeroEnVigueur: null })).toBeNull();
	});

	it('derniereVersion : le plus grand numéro', () => {
		expect(derniereVersion(detail([version({ numero: 3 }), version({ numero: 1 })]))?.numero).toBe(3);
		expect(derniereVersion(detail([]))).toBeNull();
	});

	it('peutApprouver : la dernière, d’agent, à valider, servable — et elle seule', () => {
		const d = detail([version({ numero: 1 }), agentEnAttente(2), agentEnAttente(3)]);
		expect(peutApprouver(d, d.versions[2])).toBe(true);
		expect(peutApprouver(d, d.versions[1]), 'pas la dernière').toBe(false);
		const nonServable = detail([agentEnAttente(1, { servable: false, texteServi: null })]);
		expect(peutApprouver(nonServable, nonServable.versions[0])).toBe(false);
		const decidee = detail([agentEnAttente(1, { etat: 'refused' })]);
		expect(peutApprouver(decidee, decidee.versions[0])).toBe(false);
		const admin = detail([version({ numero: 1, etat: 'pending' })]);
		expect(peutApprouver(admin, admin.versions[0])).toBe(false);
	});

	it('peutRefuser : toute version d’agent à valider, dernière ou non', () => {
		expect(peutRefuser(agentEnAttente(2))).toBe(true);
		expect(peutRefuser(agentEnAttente(2, { servable: false }))).toBe(true);
		expect(peutRefuser(agentEnAttente(2, { etat: 'approved' }))).toBe(false);
		expect(peutRefuser(version({ etat: 'pending' }))).toBe(false);
	});

	it('peutRevenirAuStandard : drapeau levé ET texte du code connu', () => {
		const code = { description: { exact: 'd', visible: 'd' }, texte: { exact: 't', visible: 't' } };
		expect(peutRevenirAuStandard(detail([], { standardDiffere: true, standardDuCode: code }))).toBe(true);
		expect(peutRevenirAuStandard(detail([], { standardDiffere: true, standardDuCode: null }))).toBe(false);
		expect(peutRevenirAuStandard(detail([], { standardDiffere: false, standardDuCode: code }))).toBe(false);
	});

	it('etiquetteStandard : Personnalisé, Diffère du standard, ou rien', () => {
		expect(etiquetteStandard(resume())).toBeNull();
		expect(etiquetteStandard(resume({ standardDiffere: true }))).toBe('differe');
		const perso = resume({
			standardDiffere: true,
			enVigueur: { numero: 2, origine: 'admin', etat: 'approved', nature: 'text' },
		});
		expect(etiquetteStandard(perso)).toBe('personnalise');
		expect(etiquetteStandard(resume({ standard: false, standardDiffere: false }))).toBeNull();
		const nonServi = resume({ enVigueur: null, standardDiffere: true });
		expect(etiquetteStandard(nonServi)).toBeNull();
	});

	describe('erreurDeGeste (AC-12)', () => {
		it('409 OPTIMISTIC_LOCK_CONFLICT : le message de l’écran, Recharger', () => {
			expect(erreurDeGeste({ status: 409, code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x' })).toEqual({
				message: 'Ce prompt a changé depuis son affichage : rechargez-le avant d’agir.',
				recharger: true,
			});
		});
		it('409 ILLEGAL_STATE_TRANSITION : le message du serveur, Recharger', () => {
			expect(
				erreurDeGeste({ status: 409, code: 'ILLEGAL_STATE_TRANSITION', message: 'Déjà décidée.' }),
			).toEqual({ message: 'Déjà décidée.', recharger: true });
		});
		it('404 : Prompt introuvable', () => {
			expect(erreurDeGeste({ status: 404, code: 'NOT_FOUND', message: 'x' })).toEqual({
				message: 'Prompt introuvable.',
				recharger: false,
			});
		});
		it('400 : le message du serveur', () => {
			expect(
				erreurDeGeste({ status: 400, code: 'VALIDATION_ERROR', message: 'Le texte du prompt est vide.' }),
			).toEqual({ message: 'Le texte du prompt est vide.', recharger: false });
		});
		it('403 : Réservé à l’Administrateur', () => {
			expect(erreurDeGeste({ status: 403, code: 'FORBIDDEN', message: 'x' })).toEqual({
				message: 'Réservé à l’Administrateur.',
				recharger: false,
			});
		});
		it('le reste : le message lisible', () => {
			expect(erreurDeGeste({ status: 500, code: 'INTERNAL_ERROR', message: 'Panne.' })).toEqual({
				message: 'Panne.',
				recharger: false,
			});
		});
	});
});
