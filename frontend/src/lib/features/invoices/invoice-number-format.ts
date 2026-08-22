/**
 * Preview du numéro de facture côté frontend (Story 5.2 — FR35).
 *
 * Miroir du helper Rust `kesh-core::invoice_format`. Les règles de
 * validation doivent rester strictement alignées (cf. tests de parité).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

export const MAX_TEMPLATE_LEN = 64;
export const MAX_RENDERED_LEN = 64;
export const MAX_SEQ_PADDING = 10;

const ALLOWED_CHAR_RE = /^[A-Za-z0-9\-_/.#\s{}:]+$/;
const KNOWN_PLACEHOLDERS = new Set(['YEAR', 'FY', 'SEQ']);

export interface FormatValidationResult {
	ok: boolean;
	error?: string;
}

/**
 * Rend un template avec des valeurs concrètes (pour preview UI).
 * Ne fait pas de validation — le caller peut valider en amont.
 */
export function previewInvoiceNumber(
	template: string,
	year: number,
	fyName: string,
	seq: number,
): string {
	let out = template;
	out = out.replace(/\{YEAR\}/g, String(year));
	out = out.replace(/\{FY\}/g, fyName);
	// {SEQ:NN} — review P9 : si NN hors borne, laisser le placeholder brut
	// (cohérent avec la validation backend qui rejettera le save).
	out = out.replace(/\{SEQ:(\d+)\}/g, (match, nStr: string) => {
		const n = parseInt(nStr, 10);
		if (!Number.isFinite(n) || n < 1 || n > MAX_SEQ_PADDING) {
			return match;
		}
		const s = String(seq);
		return s.length >= n ? s : '0'.repeat(n - s.length) + s;
	});
	out = out.replace(/\{SEQ\}/g, String(seq));
	return out;
}

/**
 * Valide un template de libellé d'écriture comptable (mirror Rust
 * `validate_description_template`). Placeholders reconnus :
 * `{YEAR}`, `{INVOICE_NUMBER}`, `{CONTACT_NAME}`.
 */
const DESCRIPTION_KNOWN = new Set(['YEAR', 'INVOICE_NUMBER', 'CONTACT_NAME']);
const MAX_DESCRIPTION_LEN = 128;

export function validateDescriptionTemplate(template: string): FormatValidationResult {
	if (!template || template.trim().length === 0) {
		return { ok: false, error: i18nMsg('invoices-description-error-empty', 'Le libellé est vide') };
	}
	// Longueur en caractères (parité P17 UTF-8).
	const chars = [...template];
	if (chars.length > MAX_DESCRIPTION_LEN) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-description-error-too-long',
				'Le libellé dépasse { $max } caractères (actuel : { $actual })',
				{ max: MAX_DESCRIPTION_LEN, actual: chars.length },
			),
		};
	}
	// Rejeter caractères de contrôle (< 0x20, hors tab) — review P11.
	for (const ch of chars) {
		const cp = ch.codePointAt(0) ?? 0;
		if (cp < 0x20 && cp !== 0x09) {
			return {
				ok: false,
				error: i18nMsg(
					'invoices-description-error-control-char',
					'Caractère de contrôle non autorisé',
				),
			};
		}
	}
	// Vérifier les placeholders.
	let hasPlaceholder = false;
	const re = /\{([^}]*)\}/g;
	let match: RegExpExecArray | null;
	while ((match = re.exec(template)) !== null) {
		if (DESCRIPTION_KNOWN.has(match[1])) {
			hasPlaceholder = true;
		} else {
			return {
				ok: false,
				error: i18nMsg(
					'invoices-description-error-unknown-placeholder',
					'Placeholder inconnu : {{ $name }}',
					{ name: match[1] },
				),
			};
		}
	}
	if (!hasPlaceholder) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-description-error-no-placeholder',
				'Le libellé doit contenir au moins un placeholder reconnu ({YEAR}, {INVOICE_NUMBER}, {CONTACT_NAME})',
			),
		};
	}
	return { ok: true };
}

/**
 * Valide un template de format (règles identiques au backend).
 */
export function validateFormatTemplate(template: string): FormatValidationResult {
	if (!template || template.trim().length === 0) {
		return {
			ok: false,
			error: i18nMsg('invoices-format-error-empty', 'Le format de numérotation est vide'),
		};
	}
	if (template.length > MAX_TEMPLATE_LEN) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-format-error-too-long',
				'Le format dépasse { $max } caractères (actuel : { $actual })',
				{ max: MAX_TEMPLATE_LEN, actual: template.length },
			),
		};
	}
	if (!ALLOWED_CHAR_RE.test(template)) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-format-error-bad-chars',
				'Le format contient des caractères non autorisés',
			),
		};
	}

	// Parse les placeholders.
	let hasPlaceholder = false;
	let worstCaseLen = 0;
	const re = /\{([^}]*)\}|([^{}]+)/g;
	let match: RegExpExecArray | null;
	while ((match = re.exec(template)) !== null) {
		if (match[2] !== undefined) {
			worstCaseLen += match[2].length;
			continue;
		}
		const inner = match[1];
		if (inner === 'YEAR') {
			hasPlaceholder = true;
			worstCaseLen += 4;
		} else if (inner === 'FY') {
			hasPlaceholder = true;
			worstCaseLen += 16;
		} else if (inner === 'SEQ') {
			hasPlaceholder = true;
			// i64::MAX = 19 chiffres (review P2 parité Rust).
			worstCaseLen += 19;
		} else if (inner.startsWith('SEQ:')) {
			const n = parseInt(inner.slice(4), 10);
			if (!Number.isFinite(n) || n < 1 || n > MAX_SEQ_PADDING) {
				return {
					ok: false,
					error: i18nMsg(
						'invoices-format-error-bad-padding',
						'Padding {SEQ:{ $n }} invalide — doit être entre 1 et { $max }',
						{ n: inner.slice(4), max: MAX_SEQ_PADDING },
					),
				};
			}
			hasPlaceholder = true;
			worstCaseLen += Math.max(n, 19);
		} else {
			return {
				ok: false,
				error: i18nMsg(
					'invoices-format-error-unknown-placeholder',
					'Placeholder inconnu : {{ $name }}',
					{ name: inner },
				),
			};
		}
	}

	if (!hasPlaceholder) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-format-error-no-placeholder',
				'Le format doit contenir au moins un placeholder reconnu ({YEAR}, {FY}, {SEQ}, {SEQ:NN})',
			),
		};
	}
	if (worstCaseLen > MAX_RENDERED_LEN) {
		return {
			ok: false,
			error: i18nMsg(
				'invoices-format-error-rendered-too-long',
				'Le format générerait un numéro de { $len } caractères (max { $max })',
				{ len: worstCaseLen, max: MAX_RENDERED_LEN },
			),
		};
	}
	return { ok: true };
}
