// Story 32-10b (refs #723) — types de l'écran des prompts du serveur MCP.
//
// Ils suivent, champ pour champ, les réponses de `routes/mcp_prompts.rs`
// (Story 32-10a-iii : `PromptResumeResponse`, `PromptDetailResponse`,
// `VersionDetailResponse`, `TexteRendu`, `StandardDuCodeResponse`,
// `VersionEcriteResponse`, et les requêtes `CreerPromptRequest`,
// `NouvelleVersionRequest`, `VersionAttendueRequest`), en camelCase. Les dates
// sont des `NaiveDateTime` de MariaDB, à l'heure UTC, sans suffixe de fuseau.
//
// ⛔ **Trois formes d'un même texte, trois usages, jamais mêlés** (fiche
// 32-10b, AC-5) :
// - `visible` (rendu visible PAR LE SERVEUR, `rendre_visible`) : l'AFFICHAGE,
//   et lui seul ;
// - `exact` du texte SERVI (`texteServi.exact`) : l'égalité avec `prompts/get`
//   dans l'E2E, et elle seule — jamais affiché, jamais dans l'éditeur ;
// - `exact` du texte ENREGISTRÉ (`texte.exact`) : l'éditeur seul (pré-remplissage,
//   *Reprendre cette version*).

/** L'origine d'une version. */
export type Origine = 'standard' | 'admin' | 'agent';
/** L'état d'une version ; `approved` et `refused` sont terminaux. */
export type Etat = 'approved' | 'pending' | 'refused';
/** La nature d'une version : un texte, ou le retrait du prompt. */
export type Nature = 'text' | 'withdrawal';

/** Le résumé d'une version — sans texte. */
export interface ResumeVersion {
	numero: number;
	origine: Origine;
	etat: Etat;
	nature: Nature;
}

/** Une ligne de la liste : la synthèse, **sans** texte ni description. */
export interface McpPromptSummary {
	id: number;
	nom: string;
	/** Prompt standard de Kesh (`kesh-…`). */
	standard: boolean;
	/** Le verrou optimiste du prompt. */
	version: number;
	/** La version de plus grand numéro, quel que soit son état. */
	derniere: ResumeVersion;
	/** La version en vigueur, texte ou retrait ; nulle si aucune n'est approuvée. */
	enVigueur: ResumeVersion | null;
	/** Le nombre de propositions d'agent à valider. */
	aValider: number;
	/** Prompt standard qui ne sert pas le texte du code (AC-i5 de la 32-10a). */
	standardDiffere: boolean;
}

/** Un texte et sa forme rendue visible par le serveur. */
export interface TexteRendu {
	exact: string;
	visible: string;
}

/** Une version de l'historique, textes compris. */
export interface McpPromptVersion {
	numero: number;
	origine: Origine;
	etat: Etat;
	nature: Nature;
	/** La description ENREGISTRÉE ; nulle pour un retrait. */
	description: TexteRendu | null;
	/** Le texte ENREGISTRÉ ; nul pour un retrait. */
	texte: TexteRendu | null;
	/** Le verdict du serveur : la version serait-elle servie ? Faux pour un retrait. */
	servable: boolean;
	/** La description telle que `prompts/get` la servirait (marque comprise) ; nulle si non servable. */
	descriptionServie: TexteRendu | null;
	/** Le texte tel que `prompts/get` le servirait (marque comprise) ; nul si non servable. */
	texteServi: TexteRendu | null;
	auteurNom: string | null;
	decideurNom: string | null;
	cleAuteurId: number | null;
	cleNom: string | null;
	cleRevoquee: boolean | null;
	creeLe: string;
	decideLe: string | null;
}

/** Le texte du code d'un prompt standard, tel qu'un retour au standard le ferait servir. */
export interface StandardDuCode {
	description: TexteRendu;
	texte: TexteRendu;
}

/** Le détail d'un prompt et de tout son historique. */
export interface McpPromptDetail {
	id: number;
	nom: string;
	standard: boolean;
	/** Le verrou optimiste — celui qu'attendent les écritures. */
	version: number;
	/** Le numéro de la version en vigueur, rendu par le serveur ; nul si aucune. */
	numeroEnVigueur: number | null;
	standardDiffere: boolean;
	/** Nul hors standard, ou pour un standard que ce binaire ne porte plus. */
	standardDuCode: StandardDuCode | null;
	modifieLe: string;
	/** Toutes les versions, dans l'ordre du serveur. */
	versions: McpPromptVersion[];
}

/** Ce qu'une écriture a écrit — sans texte. */
export interface VersionEcrite {
	promptId: number;
	nom: string;
	numero: number;
	origine: Origine;
	nature: Nature;
	etat: Etat;
	/** La `version` du prompt après l'écriture. */
	version: number;
}

/** Le corps d'un prompt neuf. */
export interface CorpsCreation {
	nom: string;
	description: string;
	texte: string;
}

/** Le corps d'une version d'administrateur : un texte, ou un retrait sans contenu. */
export type CorpsVersion =
	| { version: number; nature: 'text'; description: string; texte: string }
	| { version: number; nature: 'withdrawal' };

/** La `version` attendue du prompt (décision, retour au standard). */
export interface CorpsVersionAttendue {
	version: number;
}
