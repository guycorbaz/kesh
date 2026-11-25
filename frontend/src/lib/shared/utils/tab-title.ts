/**
 * Titre de l'onglet décoré du nom de la comptabilité — #1582 (D5, D6).
 *
 * Forme : `Kesh — <nom> — <page>` ; une page sans titre propre donne `Kesh — <nom>`.
 * Le nom est avant la page parce que les navigateurs coupent un onglet par la fin.
 * Les pages posent leur titre elles-mêmes (`<svelte:head><title>`) : le layout ne le
 * connaît pas, donc un `MutationObserver` sur `document.head` le décore à un seul endroit.
 */

const BRAND = 'Kesh';
const SEP = ' — ';

/** Retire la marque « Kesh » d'un titre de page : suffixe ou préfixe ancré, séparateur `-`, `–` ou `—`. */
function stripBrand(raw: string): string {
	let title = raw.trim();
	if (title === BRAND) return '';
	title = title.replace(/\s*[-–—]\s*Kesh$/, '');
	title = title.replace(/^Kesh\s*[-–—]\s*/, '');
	return title.trim();
}

/**
 * Titre décoré. Pure et idempotente : un titre qui commence déjà par `Kesh — <nom>` est
 * rendu tel quel. Le nom n'est jamais employé comme motif.
 */
export function titleWithCompany(raw: string, name: string): string {
	const head = BRAND + SEP + name;
	if (raw === head || raw.startsWith(head + SEP)) return raw;
	const page = stripBrand(raw);
	return page === '' ? head : head + SEP + page;
}

/**
 * Décore `document.title` tant qu'elle est active. L'appel de retour déconnecte l'observateur
 * et restaure le titre brut, **seulement si** `document.title` est encore celui qu'elle a
 * écrit (un titre posé entre-temps par une page n'est pas écrasé).
 */
export function watchTabTitle(name: string, doc: Document = document): () => void {
	let raw = doc.title;
	let written: string | null = null;

	function apply(): void {
		const current = doc.title;
		// Garde anti-boucle : jamais de traitement du titre que l'on vient d'écrire.
		if (current === written) return;
		raw = current;
		const decorated = titleWithCompany(current, name);
		written = decorated;
		if (decorated !== current) doc.title = decorated;
	}

	const observer = new MutationObserver(apply);
	observer.observe(doc.head, { childList: true, subtree: true, characterData: true });
	apply();

	return () => {
		observer.disconnect();
		if (doc.title === written) doc.title = raw;
	};
}
