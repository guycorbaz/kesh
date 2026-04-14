/**
 * Preview du numéro de facture côté frontend (Story 5.2 — FR35).
 *
 * Miroir du helper Rust `kesh-core::invoice_format`. Les règles de
 * validation doivent rester strictement alignées (cf. tests de parité).
 */

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
	// {SEQ:NN}
	out = out.replace(/\{SEQ:(\d+)\}/g, (_, nStr: string) => {
		const n = parseInt(nStr, 10);
		const s = String(seq);
		return s.length >= n ? s : '0'.repeat(n - s.length) + s;
	});
	out = out.replace(/\{SEQ\}/g, String(seq));
	return out;
}

/**
 * Valide un template de format (règles identiques au backend).
 */
export function validateFormatTemplate(template: string): FormatValidationResult {
	if (!template || template.trim().length === 0) {
		return { ok: false, error: 'Le format de numérotation est vide' };
	}
	if (template.length > MAX_TEMPLATE_LEN) {
		return {
			ok: false,
			error: `Le format dépasse ${MAX_TEMPLATE_LEN} caractères (actuel : ${template.length})`,
		};
	}
	if (!ALLOWED_CHAR_RE.test(template)) {
		return { ok: false, error: 'Le format contient des caractères non autorisés' };
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
			worstCaseLen += MAX_SEQ_PADDING;
		} else if (inner.startsWith('SEQ:')) {
			const n = parseInt(inner.slice(4), 10);
			if (!Number.isFinite(n) || n < 1 || n > MAX_SEQ_PADDING) {
				return {
					ok: false,
					error: `Padding {SEQ:${inner.slice(4)}} invalide — doit être entre 1 et ${MAX_SEQ_PADDING}`,
				};
			}
			hasPlaceholder = true;
			worstCaseLen += n;
		} else {
			return { ok: false, error: `Placeholder inconnu : {${inner}}` };
		}
		// Cas non reconnu
		if (!KNOWN_PLACEHOLDERS.has(inner) && !inner.startsWith('SEQ:')) {
			// (unreachable — logique ci-dessus couvre tout)
		}
	}

	if (!hasPlaceholder) {
		return {
			ok: false,
			error: 'Le format doit contenir au moins un placeholder reconnu ({YEAR}, {FY}, {SEQ}, {SEQ:NN})',
		};
	}
	if (worstCaseLen > MAX_RENDERED_LEN) {
		return {
			ok: false,
			error: `Le format générerait un numéro de ${worstCaseLen} caractères (max ${MAX_RENDERED_LEN})`,
		};
	}
	return { ok: true };
}
