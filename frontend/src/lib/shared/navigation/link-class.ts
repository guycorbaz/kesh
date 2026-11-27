/**
 * Story 55-2 (D17, AC-16) — le style des liens de Kesh : un lien se voit **sans la couleur** (WCAG 1.4.1),
 * par un soulignement léger, au repos comme au survol.
 *
 * - `LINK_CLASS` : la couleur des liens de compte (`text-indigo-700`) et le soulignement léger, à 40 % de
 *   la couleur, plein au survol. Employée par `AccountLink` (Rapports) ; l'Epic 56 l'importe, sans copie.
 * - `LINK_UNDERLINE_CLASS` : le même soulignement **sans couleur**, pour un lien qui garde la sienne
 *   (`text-primary`, la couleur du texte) : `decoration-current/40` suit la couleur du lien.
 *
 * ⚠️ Chaînes littérales entières : Tailwind ne voit que les classes écrites telles quelles dans le code ;
 * ne pas les assembler.
 */

/** Lien de compte : couleur des liens et soulignement léger (décision D17, à l'octet près). */
export const LINK_CLASS =
	'text-indigo-700 underline decoration-indigo-700/40 underline-offset-2 hover:decoration-current';

/** Le soulignement léger seul, pour un lien qui garde sa couleur. */
export const LINK_UNDERLINE_CLASS =
	'underline decoration-current/40 underline-offset-2 hover:decoration-current';
