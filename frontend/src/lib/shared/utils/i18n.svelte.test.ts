/**
 * Preuves du CHEMIN RÉEL de résolution i18n — passe 3 de revue de la Story 22-2 (#301).
 *
 * ⚠️ **Ce fichier existe parce qu'un correctif a été déclaré prouvé alors qu'il
 * était inerte en production.** Un sélecteur de pluriel Fluent avait été ajouté
 * pour corriger « et 1 autres » ; un test Rust appelant `bundle.format(clé,
 * Some(args))` passait, un test de composant passait, et l'application affichait
 * toujours « et 1 autres ».
 *
 * La raison tient au chemin : le backend sert un dictionnaire **pré-résolu SANS
 * arguments**, si bien qu'une expression `select` y est figée sur sa branche
 * `*[other]`. Aucun des deux tests n'empruntait ce chemin — l'un passait par
 * `format()` avec arguments, l'autre **mockait `i18nMsg` en réimplémentant sa
 * logique**. Trois niveaux de preuves, aucune sur le trajet réel.
 *
 * D'où ces tests-ci : ils partent du dictionnaire que le serveur envoie
 * VRAIMENT, et vont jusqu'à la chaîne affichée.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({ apiClient: { get: (u: string) => getMock(u) } }));

import { i18nMsg, loadI18nMessages } from './i18n.svelte';

/** Charge un dictionnaire comme le ferait `GET /api/v1/i18n/messages`. */
async function servir(messages: Record<string, string>) {
	getMock.mockResolvedValue({ locale: 'fr-CH', messages });
	await loadI18nMessages();
}

beforeEach(() => {
	getMock.mockReset();
});

describe('i18nMsg — le chemin que la production emprunte', () => {
	it('le dictionnaire du serveur PRIME sur le repli', async () => {
		// MUTATION : « inverser la précédence en `fallback || _messages[key]` ».
		//
		// ⚠️ C'est la clé de tout le défaut : un repli astucieux côté Svelte —
		// un ternaire qui choisit la bonne forme — ne sert **jamais** dès que la
		// clé existe côté serveur. On peut donc écrire un repli parfait et
		// n'avoir rien corrigé.
		await servir({ 'une-cle': 'du serveur' });
		expect(i18nMsg('une-cle', 'mon repli')).toBe('du serveur');
	});

	it('le repli ne sert QUE si la clé manque au dictionnaire', async () => {
		await servir({ autre: 'x' });
		expect(i18nMsg('une-cle', 'mon repli')).toBe('mon repli');
	});

	it('le placeholder est interpolé CÔTÉ CLIENT, marques d’isolation comprises', async () => {
		// Le serveur renvoie le placeholder non résolu, entouré des marques
		// d'isolation bidirectionnelle U+2068/U+2069 que Fluent y appose.
		await servir({ compteur: 'et ⁨{$count}⁩ autres' });
		expect(i18nMsg('compteur', 'et { $count } autres', { count: 3 })).toBe('et 3 autres');
	});

	it('DÉMONSTRATION du piège : un sélecteur Fluent arrive AMPUTÉ, et rend le pluriel à 1', async () => {
		// ⚠️ Ce test ne garde pas un comportement souhaitable — **il figes le
		// mécanisme du défaut**, pour que personne ne rétablisse un sélecteur en
		// croyant régler l'accord. Le dictionnaire ci-dessous est exactement ce
		// que `all_messages` produit à partir d'une expression `select` : la
		// branche `*[other]`, et elle seule.
		await servir({ compteur: 'et ⁨{$count}⁩ autres' });
		expect(i18nMsg('compteur', 'et 1 autre', { count: 1 })).toBe('et 1 autres');
	});

	it('la forme singulière vient d’une clé DISTINCTE, et elle est juste', async () => {
		// La réparation : deux clés plates, que le dictionnaire sait servir.
		await servir({
			'contact-duplicate-others-count-one': 'et 1 autre',
			'contact-duplicate-others-count': 'et ⁨{$count}⁩ autres'
		});
		expect(i18nMsg('contact-duplicate-others-count-one', 'et 1 autre')).toBe('et 1 autre');
		expect(
			i18nMsg('contact-duplicate-others-count', 'et { $count } autres', { count: 4 })
		).toBe('et 4 autres');
	});
});
