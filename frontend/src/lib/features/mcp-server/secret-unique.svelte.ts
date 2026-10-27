// Story 32-7 (refs #723) — le secret d'une clé MCP, tenu une seule fois (AC-11).
//
// ⛔ Une instance PAR COMPOSANT, créée dans le bloc `<script>` d'instance — jamais au
// niveau d'un module ni dans un `<script module>` : une instance de module survit à
// la fermeture de l'encart et à la navigation (garde G-b,
// `mcp-server-gardes.test.ts`).
//
// Les champs sont privés (`#`) : `JSON.stringify` et `Object.getOwnPropertyNames` ne
// les voient pas — `lire()` est donc l'unique preuve que `oublier()` efface.

/** Le nom de la clé créée et son secret, tant qu'ils sont montrés. */
export class SecretUnique {
	#valeur = $state<string | null>(null);
	#nom = $state<string | null>(null);

	/** Montre le secret d'une clé ; remplace tout secret précédent. */
	montrer(nom: string, valeur: string): void {
		this.#nom = nom;
		this.#valeur = valeur;
	}

	/** Le nom et le secret montrés, ou `null` s'il n'y en a pas. */
	lire(): { nom: string; valeur: string } | null {
		return this.#valeur === null ? null : { nom: this.#nom ?? '', valeur: this.#valeur };
	}

	/** Oublie le nom et le secret. */
	oublier(): void {
		this.#valeur = null;
		this.#nom = null;
	}
}
