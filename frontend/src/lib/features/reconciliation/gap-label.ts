/**
 * Story 43-3 (#823, D2 bis, C-43-26) — l'écart d'un règlement groupé dit avec son **sens** :
 * « Moins-payé de 0.40 », « Débit supérieur aux factures de 0.40 »… Partagé par la modale du
 * règlement et le dialogue d'annulation (revue P1 : l'annulation affichait l'écart signé, sans
 * son sens), pour qu'un même écart se dise partout de la même façon.
 */
import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatUnits, gapSense, type GroupFamily } from './group-settlement';

/**
 * L'écart `gap` (en unités de 1/10 000, signé) en mots, ou « Aucun écart ».
 *
 * `inSentence` : le libellé s'insère après un deux-points (« Écart trop grand : moins-payé
 * de 6.00 ») ; sa première lettre passe alors en minuscule, sauf en allemand, où les noms
 * gardent leur majuscule (revue P2, L-5).
 */
export function gapSenseLabel(family: GroupFamily, gap: bigint, inSentence = false): string {
	const label = senseLabel(family, gap);
	if (!inSentence || i18nLocale().toLowerCase().startsWith('de')) return label;
	return label.charAt(0).toLocaleLowerCase() + label.slice(1);
}

function senseLabel(family: GroupFamily, gap: bigint): string {
	const amount = formatUnits(gap < 0n ? -gap : gap);
	switch (gapSense(family, gap)) {
		case 'client-under':
			return i18nMsg('reconciliation-group-gap-client-under', 'Moins-payé de { $amount }', { amount });
		case 'client-over':
			return i18nMsg('reconciliation-group-gap-client-over', 'Trop-perçu de { $amount }', { amount });
		case 'supplier-over':
			return i18nMsg(
				'reconciliation-group-gap-supplier-over',
				'Débit supérieur aux factures de { $amount }',
				{ amount },
			);
		case 'supplier-under':
			return i18nMsg(
				'reconciliation-group-gap-supplier-under',
				'Débit inférieur aux factures de { $amount }',
				{ amount },
			);
		default:
			return i18nMsg('reconciliation-group-gap-none', 'Aucun écart');
	}
}
