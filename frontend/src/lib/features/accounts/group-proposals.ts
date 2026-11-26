/**
 * Story 46-5b — le calcul pur des propositions de groupes du plan comptable.
 *
 * L'écran « Créer des groupes » (`routes/(app)/accounts/groups/`) coche des
 * comptes ; ce module en déduit les **propositions** (un compte de groupe à
 * créer, à rattacher ou à réutiliser, et les comptes à y ranger), juge chacune
 * (erreurs, avertissements, informations), construit l'arbre de l'aperçu et le
 * corps de `POST /api/v1/accounts/groups` (Story 46-5a). Aucune I/O, aucun
 * texte : les messages sont des **codes** que
 * `routes/(app)/accounts/groups/messages.ts` traduit. Aucune règle n'est recopiée dans le composant.
 *
 * La règle est celle de la fiche mère `46-5-aide-groupes-plan.md` (§ *Règle des
 * propositions*) :
 *
 * - mode `digit` (« numéro sans son dernier chiffre ») : chaque compte coché de
 *   numéro d'au moins deux caractères va au groupe `numéro[..len-1]` ; un
 *   préfixe qui ne réunit qu'un compte ne donne aucune proposition, sauf si le
 *   préfixe est un compte de groupe existant (rattachement) ;
 * - mode `one` (« un seul groupe pour la sélection ») : une proposition, numéro
 *   = plus long préfixe commun ; si ce préfixe est le numéro du parent commun,
 *   ce préfixe suivi de `0` s'il est libre, sinon un numéro vide à saisir.
 *
 * Le serveur rejuge tout sous verrou : l'aperçu peut dater, il ne décide rien.
 */

import type { AccountResponse, AccountType } from './accounts.types';

/** Les deux règles de proposition. */
export type GroupMode = 'digit' | 'one';

/** L'état d'une proposition : rien n'est accepté d'office. */
export type ProposalState = 'pending' | 'accepted' | 'rejected';

/**
 * Le parent choisi pour un groupe à créer : l'`id` d'un compte, `'root'`
 * (racine, choix exprès) ou `null` (pas encore choisi).
 */
export type ParentChoice = number | 'root' | null;

/** Ce que devient le compte de groupe : créé, rattaché (existant) ou réutilisé. */
export type ProposalKind = 'create' | 'attach' | 'reuse';

/** Le niveau d'un message ; seule une erreur bloque. */
export type MessageLevel = 'error' | 'warning' | 'info';

/** Les codes des messages d'une proposition (traduits par `routes/(app)/accounts/groups/messages.ts`). */
export type ProposalMessageCode =
	// erreurs
	| 'numberEmpty'
	| 'numberTooLong'
	| 'nameEmpty'
	| 'nameTooLong'
	| 'mixedTypes'
	| 'empty'
	| 'groupTypeMismatch'
	| 'groupHasEntries'
	| 'numberTakenEntries'
	| 'numberTakenArchived'
	| 'numberTakenReusable'
	| 'numberIsMember'
	| 'numberDuplicate'
	| 'groupIsMember'
	| 'parentMissing'
	| 'parentHasEntries'
	// avertissements
	| 'root'
	| 'mixedNames'
	| 'reuseNotPostable'
	// informations
	| 'attachExisting'
	| 'alreadyInGroup'
	| 'mixedCurrencies'
	| 'uncheckedSamePrefix';

/** Un message d'une proposition, avec ses paramètres d'interpolation. */
export interface ProposalMessage {
	level: MessageLevel;
	code: ProposalMessageCode;
	params: Record<string, string>;
}

/** Ce que l'utilisateur a choisi ou modifié pour une proposition. */
export interface ProposalEdit {
	number: string;
	name: string;
	parent: ParentChoice;
	state: ProposalState;
	/** Coche « réutiliser ce compte comme groupe » (décochée par défaut). */
	reuse: boolean;
	/** Identifiants des membres décochés. */
	excluded: number[];
}

/**
 * Ce que l'utilisateur a **réellement** touché sur une proposition : seuls ces
 * champs sont gardés, appliqués sur les valeurs par défaut recalculées à chaque
 * changement de sélection ou de mode (revue P1 de la 46-5b, M1). Un champ que
 * l'utilisateur n'a pas modifié suit donc la sélection.
 */
export interface ProposalPatch extends Partial<ProposalEdit> {
	/**
	 * Les membres (`membersSignature`) au moment de l'acceptation : une
	 * proposition acceptée dont les membres ont changé depuis repasse en attente.
	 */
	acceptedFor?: string;
}

/** Les choix de l'utilisateur, par clé de proposition. */
export type ProposalEdits = Record<string, ProposalPatch>;

/** Une proposition avant jugement : sa clé, ses membres et ses valeurs par défaut. */
export interface ProposalSkeleton {
	/** Clé stable (`digit:103`, `one`) : elle garde les choix d'un recalcul à l'autre. */
	key: string;
	members: AccountResponse[];
	defaults: ProposalEdit;
}

/** Les propositions d'une sélection, et les comptes seuls de leur préfixe. */
export interface ProposalSet {
	skeletons: ProposalSkeleton[];
	singles: AccountResponse[];
}

/** Une proposition jugée. */
export interface CheckedProposal {
	key: string;
	members: AccountResponse[];
	edit: ProposalEdit;
	kind: ProposalKind;
	/** Le compte existant qui porte le numéro (groupe rattaché, compte réutilisé ou refusé). */
	existing: AccountResponse | null;
	/** Les membres retenus (non décochés). */
	included: AccountResponse[];
	/** Les membres retenus qui changent de parent. */
	moves: AccountResponse[];
	messages: ProposalMessage[];
	hasError: boolean;
	/** Le numéro est un compte actif sans écriture ni sous-compte : la coche « réutiliser » s'offre. */
	canReuse: boolean;
}

/** Longueurs maximales du serveur (`POST /accounts`), en octets UTF-8. */
export const NUMBER_MAX_BYTES = 10;
export const NAME_MAX_BYTES = 255;
/** Bornes de la demande (`tooMany`, 46-5a). */
export const MAX_PROPOSALS = 100;
export const MAX_MEMBERS = 1000;

const encoder = new TextEncoder();
const bytes = (s: string): number => encoder.encode(s).length;
/** Clé de comparaison d'un numéro : le serveur le compare sans casse (46-5a, L1). */
const numberKey = (s: string): string => s.trim().toLowerCase();
/** Ordre des numéros, celui de `String::cmp` côté serveur (unités de code). */
const byNumber = (a: { number: string }, b: { number: string }): number =>
	a.number < b.number ? -1 : a.number > b.number ? 1 : 0;

/**
 * Un compte de groupe : actif, avec au moins un sous-compte actif (D1 de la
 * 46-1). Il ne se coche pas ; il sert de parent.
 */
export function isGroupAccount(account: AccountResponse, accounts: readonly AccountResponse[]): boolean {
	return account.active && accounts.some((c) => c.active && c.parentId === account.id);
}

/** Les comptes qui se cochent : actifs et non groupes. */
export function selectableAccounts(accounts: readonly AccountResponse[]): AccountResponse[] {
	return accounts.filter((a) => a.active && !isGroupAccount(a, accounts));
}

/** Le compte, archivés compris, qui porte ce numéro (comparé sans casse). */
function accountByNumber(accounts: readonly AccountResponse[], number: string): AccountResponse | null {
	const k = numberKey(number);
	if (k === '') return null;
	return accounts.find((a) => numberKey(a.number) === k) ?? null;
}

/** Le plus long préfixe commun. */
export function longestCommonPrefix(values: readonly string[]): string {
	if (values.length === 0) return '';
	let p = values[0];
	for (const v of values) while (!v.startsWith(p)) p = p.slice(0, -1);
	return p;
}

/** Les mots de tête communs à tous les noms (« Banque A CHF », « Banque A EUR » → « Banque A »). */
export function commonLeadingWords(names: readonly string[]): string {
	if (names.length === 0) return '';
	const words = names.map((n) => n.trim().split(/\s+/));
	const out: string[] = [];
	for (let i = 0; i < words[0].length; i++) {
		if (words.every((w) => w[i] === words[0][i])) out.push(words[0][i]);
		else break;
	}
	return out.join(' ');
}

/** Le « nom de tête » d'un compte (deux premiers mots), qui sépare deux banques. */
const stemOf = (a: AccountResponse): string => a.name.trim().split(/\s+/).slice(0, 2).join(' ');

/** Le parent commun des comptes : son `id`, `'root'` s'ils sont tous à la racine, `null` sinon. */
function commonParent(members: readonly AccountResponse[]): ParentChoice {
	const parents = new Set(members.map((m) => m.parentId));
	if (parents.size !== 1) return null;
	const [p] = parents;
	return p === null ? 'root' : p;
}

function defaultEdit(number: string, members: readonly AccountResponse[]): ProposalEdit {
	return {
		number,
		name: commonLeadingWords(members.map((m) => m.name)),
		parent: commonParent(members),
		state: 'pending',
		reuse: false,
		excluded: [],
	};
}

/**
 * Les propositions de la sélection, selon le mode. `accounts` est la liste
 * **archivés compris** (`GET /accounts?includeArchived=true`) : les archivés
 * comptent pour les numéros pris, jamais comme membres.
 */
export function proposalSkeletons(
	accounts: readonly AccountResponse[],
	selection: ReadonlySet<number>,
	mode: GroupMode,
): ProposalSet {
	const members = selectableAccounts(accounts)
		.filter((a) => selection.has(a.id))
		.sort(byNumber);
	if (mode === 'one') {
		if (members.length === 0) return { skeletons: [], singles: [] };
		let number = longestCommonPrefix(members.map((m) => m.number));
		const parent = commonParent(members);
		if (typeof parent === 'number') {
			const parentAccount = accounts.find((a) => a.id === parent);
			if (parentAccount && number === parentAccount.number) {
				const candidate = `${number}0`;
				number = accountByNumber(accounts, candidate) ? '' : candidate;
			}
		}
		return { skeletons: [{ key: 'one', members, defaults: defaultEdit(number, members) }], singles: [] };
	}

	const buckets = new Map<string, AccountResponse[]>();
	const singles: AccountResponse[] = [];
	for (const m of members) {
		if (m.number.length < 2) {
			singles.push(m);
			continue;
		}
		const prefix = m.number.slice(0, -1);
		const list = buckets.get(prefix);
		if (list) list.push(m);
		else buckets.set(prefix, [m]);
	}
	const skeletons: ProposalSkeleton[] = [];
	for (const [prefix, list] of buckets) {
		const existing = accountByNumber(accounts, prefix);
		const existingGroup = existing !== null && isGroupAccount(existing, accounts);
		if (list.length < 2 && !existingGroup) {
			singles.push(...list);
			continue;
		}
		// Rattachement à un groupe existant dont tous les comptes sont déjà les
		// sous-comptes : rien à faire, aucune proposition.
		if (existingGroup && list.every((m) => m.parentId === existing.id)) continue;
		skeletons.push({ key: `digit:${prefix}`, members: list, defaults: defaultEdit(prefix, list) });
	}
	skeletons.sort((a, b) => byNumber({ number: a.defaults.number }, { number: b.defaults.number }));
	singles.sort(byNumber);
	return { skeletons, singles };
}

/** L'ensemble des membres d'une proposition, en chaîne comparable (identifiants triés). */
export function membersSignature(members: readonly Pick<AccountResponse, 'id'>[]): string {
	return members
		.map((m) => m.id)
		.sort((a, b) => a - b)
		.join(',');
}

/**
 * Le choix courant d'une proposition : les valeurs par défaut **recalculées**,
 * sur lesquelles s'appliquent les seuls champs modifiés par l'utilisateur.
 * Deux corrections (revue P1 de la 46-5b) :
 *
 * - une proposition acceptée dont les membres ont changé depuis l'acceptation
 *   repasse **en attente** (M1) : rien ne s'élargit en silence ;
 * - un parent choisi devenu membre de la proposition est **remis à vide** (M3) :
 *   il ne figure plus parmi les parents offerts, l'écran le montre « à
 *   choisir », et l'erreur `parentMissing` le dit au lieu d'envoyer un parent
 *   que le serveur refuserait (`ACCOUNT_PARENT_CYCLE`).
 */
export function editOf(skeleton: ProposalSkeleton, edits: Readonly<ProposalEdits>): ProposalEdit {
	const patch = edits[skeleton.key];
	if (!patch) return skeleton.defaults;
	const { acceptedFor, ...fields } = patch;
	const edit: ProposalEdit = { ...skeleton.defaults, ...fields };
	if (edit.state === 'accepted' && acceptedFor !== membersSignature(skeleton.members)) edit.state = 'pending';
	const parent = edit.parent;
	if (typeof parent === 'number' && skeleton.members.some((m) => m.id === parent)) edit.parent = null;
	return edit;
}

/**
 * Range un geste de l'utilisateur : `patch` s'ajoute aux champs déjà modifiés
 * de la proposition, sans recopier les valeurs par défaut du moment. Une
 * acceptation note les membres acceptés (`acceptedFor`).
 */
export function applyEdit(
	edits: Readonly<ProposalEdits>,
	proposal: Pick<CheckedProposal, 'key' | 'members'>,
	patch: Partial<ProposalEdit>,
): ProposalEdits {
	const next: ProposalPatch = { ...edits[proposal.key], ...patch };
	if (patch.state === 'accepted') next.acceptedFor = membersSignature(proposal.members);
	return { ...edits, [proposal.key]: next };
}

/**
 * Juge chaque proposition. Les contrôles entre propositions (numéro demandé
 * deux fois, compte d'un rattachement membre d'une autre) ne portent que sur
 * les propositions **acceptées**.
 */
export function checkProposals(
	set: ProposalSet,
	edits: Readonly<ProposalEdits>,
	accounts: readonly AccountResponse[],
	selection: ReadonlySet<number>,
	mode: GroupMode,
): CheckedProposal[] {
	const base = set.skeletons.map((s) => judgeAlone(s, editOf(s, edits), accounts, selection, mode, set.singles));
	const accepted = base.filter((p) => p.edit.state === 'accepted');
	for (const p of base) {
		if (p.edit.state !== 'accepted') continue;
		const others = accepted.filter((o) => o !== p);
		const k = numberKey(p.edit.number);
		if (k !== '' && others.some((o) => numberKey(o.edit.number) === k)) {
			push(p, 'error', 'numberDuplicate', { number: p.edit.number.trim() });
		}
		if (p.existing && p.kind !== 'create') {
			const id = p.existing.id;
			if (others.some((o) => o.included.some((m) => m.id === id))) {
				push(p, 'error', 'groupIsMember', { number: p.existing.number });
			}
		}
	}
	return base;
}

function push(p: CheckedProposal, level: MessageLevel, code: ProposalMessageCode, params: Record<string, string> = {}) {
	p.messages.push({ level, code, params });
	if (level === 'error') p.hasError = true;
}

/** Les contrôles propres à une proposition. */
function judgeAlone(
	skeleton: ProposalSkeleton,
	edit: ProposalEdit,
	accounts: readonly AccountResponse[],
	selection: ReadonlySet<number>,
	mode: GroupMode,
	singles: readonly AccountResponse[],
): CheckedProposal {
	const excluded = new Set(edit.excluded);
	const included = skeleton.members.filter((m) => !excluded.has(m.id));
	const number = edit.number.trim();
	const p: CheckedProposal = {
		key: skeleton.key,
		members: skeleton.members,
		edit,
		kind: 'create',
		existing: null,
		included,
		moves: [],
		messages: [],
		hasError: false,
		canReuse: false,
	};

	if (number === '') push(p, 'error', 'numberEmpty');
	else if (bytes(number) > NUMBER_MAX_BYTES) push(p, 'error', 'numberTooLong', { max: String(NUMBER_MAX_BYTES) });

	const types = [...new Set(included.map((m) => m.accountType))];
	if (types.length > 1) push(p, 'error', 'mixedTypes', { types: types.join(', ') });
	if (included.length === 0) push(p, 'error', 'empty');

	const existing = accountByNumber(accounts, number);
	p.existing = existing;
	const memberType: AccountType | undefined = types.length === 1 ? types[0] : undefined;
	if (existing) {
		const params = { number: existing.number, name: existing.name };
		if (skeleton.members.some((m) => m.id === existing.id)) {
			push(p, 'error', 'numberIsMember', { number: existing.number });
		} else if (!existing.active) {
			push(p, 'error', 'numberTakenArchived', params);
		} else if (isGroupAccount(existing, accounts)) {
			p.kind = 'attach';
			push(p, 'info', 'attachExisting', params);
			if (existing.currencyLocked === true) push(p, 'error', 'groupHasEntries', params);
			if (memberType && existing.accountType !== memberType) {
				push(p, 'error', 'groupTypeMismatch', { ...params, type: existing.accountType });
			}
		} else if (existing.currencyLocked === true) {
			push(p, 'error', 'numberTakenEntries', params);
		} else {
			p.canReuse = true;
			if (edit.reuse) {
				p.kind = 'reuse';
				push(p, 'warning', 'reuseNotPostable', params);
				if (memberType && existing.accountType !== memberType) {
					push(p, 'error', 'groupTypeMismatch', { ...params, type: existing.accountType });
				}
			} else {
				push(p, 'error', 'numberTakenReusable', params);
			}
		}
	}

	if (p.kind === 'create') {
		const name = edit.name.trim();
		if (name === '') push(p, 'error', 'nameEmpty');
		else if (bytes(name) > NAME_MAX_BYTES) push(p, 'error', 'nameTooLong', { max: String(NAME_MAX_BYTES) });
		if (edit.parent === null) push(p, 'error', 'parentMissing');
		else if (edit.parent === 'root') push(p, 'warning', 'root');
		else {
			const parentId = edit.parent;
			const parent = accounts.find((a) => a.id === parentId);
			if (parent?.currencyLocked === true) {
				push(p, 'error', 'parentHasEntries', { number: parent.number, name: parent.name });
			}
		}
	}

	const targetId = p.kind === 'create' ? null : (existing?.id ?? null);
	const already = targetId === null ? [] : included.filter((m) => m.parentId === targetId);
	p.moves = targetId === null ? included : included.filter((m) => m.parentId !== targetId);
	if (already.length > 0) {
		push(p, 'info', 'alreadyInGroup', { numbers: already.map((m) => m.number).join(', ') });
	}
	if (included.length > 0 && p.moves.length === 0 && !p.messages.some((m) => m.code === 'empty')) {
		push(p, 'error', 'empty');
	}

	if (mode === 'digit') {
		// Deux noms de tête réunis sous un préfixe : un bloc de dix qui chevauche une dizaine.
		// Le compte d'une banque tombé seul dans la dizaine suivante compte aussi.
		const stems = [...new Set(included.map(stemOf))];
		const strays = singles.filter((s) => stems.includes(stemOf(s)));
		const repeated = stems.filter((st) => included.filter((m) => stemOf(m) === st).length >= 2);
		if (stems.length > 1 && (repeated.length >= 2 || (repeated.length >= 1 && strays.length > 0))) {
			push(p, 'warning', 'mixedNames', { names: stems.join(', ') });
		}
	}

	const currencies = [...new Set(included.map((m) => m.currency ?? 'CHF'))];
	if (currencies.length > 1) push(p, 'info', 'mixedCurrencies', { currencies: currencies.join(', ') });

	if (mode === 'digit') {
		const prefix = skeleton.key.slice('digit:'.length);
		const unchecked = selectableAccounts(accounts).filter(
			(a) =>
				a.number.length === prefix.length + 1 &&
				a.number.startsWith(prefix) &&
				!selection.has(a.id) &&
				(targetId === null || a.parentId !== targetId),
		);
		if (unchecked.length > 0) {
			push(p, 'info', 'uncheckedSamePrefix', { numbers: unchecked.sort(byNumber).map((a) => a.number).join(', ') });
		}
	}

	return p;
}

/** Les propositions qui comptent : acceptées et sans erreur. */
export function appliedProposals(checked: readonly CheckedProposal[]): CheckedProposal[] {
	return checked.filter((p) => p.edit.state === 'accepted' && !p.hasError);
}

/** Un compte à ranger, avec la `version` lue à l'aperçu. */
export interface GroupMemberRequest {
	id: number;
	version: number;
}

/** Une proposition du corps de `POST /api/v1/accounts/groups` (46-5a). */
export type GroupRequest =
	| { kind: 'create'; number: string; name: string; parentId: number | null; accounts: GroupMemberRequest[] }
	| { kind: 'attach'; accountId: number; accounts: GroupMemberRequest[] }
	| { kind: 'reuse'; accountId: number; accounts: GroupMemberRequest[] };

/** Corps de `POST /api/v1/accounts/groups`. */
export interface ApplyGroupsRequest {
	groups: GroupRequest[];
}

/** Réponse `200` de `POST /api/v1/accounts/groups`. */
export interface ApplyGroupsResponse {
	created: AccountResponse[];
	reused: AccountResponse[];
	moved: AccountResponse[];
}

/**
 * Le corps de la demande, depuis les seules propositions acceptées et sans
 * erreur, et la clé de proposition de chaque rang (`groupIndex` d'un refus).
 */
export function buildGroupsRequest(checked: readonly CheckedProposal[]): {
	request: ApplyGroupsRequest;
	keys: string[];
} {
	const applied = appliedProposals(checked);
	const groups = applied.map((p): GroupRequest => {
		const accounts = p.moves.map((m) => ({ id: m.id, version: m.version }));
		if (p.kind === 'create') {
			return {
				kind: 'create',
				number: p.edit.number.trim(),
				name: p.edit.name.trim(),
				parentId: typeof p.edit.parent === 'number' ? p.edit.parent : null,
				accounts,
			};
		}
		return { kind: p.kind, accountId: (p.existing as AccountResponse).id, accounts };
	});
	return { request: { groups }, keys: applied.map((p) => p.key) };
}

/** Le décompte du pied de page. */
export interface ProposalSummary {
	created: number;
	reused: number;
	moved: number;
	pending: number;
	acceptedWithErrors: number;
	/** Plus de 100 propositions retenues ou de 1000 comptes à ranger (`tooMany`). */
	tooMany: boolean;
	/** *Confirmer* est actif. */
	canConfirm: boolean;
}

export function proposalSummary(checked: readonly CheckedProposal[]): ProposalSummary {
	const applied = appliedProposals(checked);
	const moved = applied.reduce((s, p) => s + p.moves.length, 0);
	const acceptedWithErrors = checked.filter((p) => p.edit.state === 'accepted' && p.hasError).length;
	const tooMany = applied.length > MAX_PROPOSALS || moved > MAX_MEMBERS;
	return {
		created: applied.filter((p) => p.kind === 'create').length,
		reused: applied.filter((p) => p.kind === 'reuse').length,
		moved,
		pending: checked.filter((p) => p.edit.state === 'pending').length,
		acceptedWithErrors,
		tooMany,
		canConfirm: applied.length > 0 && acceptedWithErrors === 0 && !tooMany,
	};
}

/** Une ligne de l'aperçu de l'arbre. */
export interface TreeRow {
	key: string;
	number: string;
	name: string;
	accountType: AccountType;
	depth: number;
	isGroup: boolean;
	mark: 'new' | 'moved' | 'becomesGroup' | null;
}

interface TreeNode {
	key: string;
	number: string;
	name: string;
	accountType: AccountType;
	parentKey: string | null;
	mark: TreeRow['mark'];
}

/**
 * L'arbre des comptes actifs tel qu'il serait après confirmation des
 * propositions acceptées et sans erreur. `onlyTouched` ne garde que les
 * branches modifiées (les groupes visés, leurs ancêtres et leurs sous-comptes).
 */
export function previewTree(
	accounts: readonly AccountResponse[],
	checked: readonly CheckedProposal[],
	onlyTouched = false,
): TreeRow[] {
	const nodes = new Map<string, TreeNode>();
	const active = accounts.filter((a) => a.active);
	const activeIds = new Set(active.map((a) => a.id));
	for (const a of active) {
		nodes.set(`a${a.id}`, {
			key: `a${a.id}`,
			number: a.number,
			name: a.name,
			accountType: a.accountType,
			parentKey: a.parentId !== null && activeIds.has(a.parentId) ? `a${a.parentId}` : null,
			mark: null,
		});
	}
	const touched = new Set<string>();
	for (const p of appliedProposals(checked)) {
		let target: string;
		if (p.kind === 'create') {
			target = `n${p.key}`;
			nodes.set(target, {
				key: target,
				number: p.edit.number.trim(),
				name: p.edit.name.trim(),
				accountType: p.included[0]?.accountType ?? 'Asset',
				parentKey: typeof p.edit.parent === 'number' ? `a${p.edit.parent}` : null,
				mark: 'new',
			});
		} else {
			target = `a${(p.existing as AccountResponse).id}`;
			const node = nodes.get(target);
			if (node && p.kind === 'reuse') node.mark = 'becomesGroup';
		}
		for (const m of p.moves) {
			const node = nodes.get(`a${m.id}`);
			if (node) {
				node.parentKey = target;
				node.mark = 'moved';
			}
		}
		touched.add(target);
	}

	const children = new Map<string | null, TreeNode[]>();
	for (const n of nodes.values()) {
		const parent = n.parentKey !== null && nodes.has(n.parentKey) ? n.parentKey : null;
		const list = children.get(parent);
		if (list) list.push(n);
		else children.set(parent, [n]);
	}
	for (const list of children.values()) list.sort(byNumber);

	let keep: Set<string> | null = null;
	if (onlyTouched && touched.size > 0) {
		const kept = new Set<string>();
		for (const t of touched) {
			let cur = nodes.get(t);
			let guard = 0;
			while (cur && guard++ < nodes.size) {
				kept.add(cur.key);
				cur = cur.parentKey !== null ? nodes.get(cur.parentKey) : undefined;
			}
			for (const c of children.get(t) ?? []) kept.add(c.key);
		}
		keep = kept;
	}

	const rows: TreeRow[] = [];
	const seen = new Set<string>();
	const walk = (parent: string | null, depth: number) => {
		for (const n of children.get(parent) ?? []) {
			if (seen.has(n.key)) continue;
			seen.add(n.key);
			if (keep === null || keep.has(n.key)) {
				rows.push({
					key: n.key,
					number: n.number,
					name: n.name,
					accountType: n.accountType,
					depth,
					isGroup: (children.get(n.key)?.length ?? 0) > 0,
					mark: n.mark,
				});
			}
			walk(n.key, depth + 1);
		}
	};
	walk(null, 0);
	return rows;
}

/**
 * Les comptes que peut prendre pour parent un groupe à créer : actifs, sans
 * écriture (règle du `PUT`, #1351), hors des membres de la proposition — plus le
 * parent déjà choisi, pour qu'il reste lisible.
 */
export function parentOptionsFor(
	accounts: readonly AccountResponse[],
	proposal: Pick<CheckedProposal, 'members' | 'edit'>,
): AccountResponse[] {
	const memberIds = new Set(proposal.members.map((m) => m.id));
	const current = proposal.edit.parent;
	return accounts
		.filter(
			(a) =>
				a.active &&
				!memberIds.has(a.id) &&
				(a.currencyLocked !== true || a.id === current),
		)
		.sort(byNumber);
}

/** Où porte un refus de `POST /accounts/groups` : le rang de la proposition et, s'il est donné, le compte. */
export interface GroupRefusalTarget {
	groupIndex: number;
	accountId: number | null;
}

/**
 * Lit la cible d'un refus sans faire confiance à sa forme : `details.groupStep`
 * (codes existants de la création et du déplacement) ou `details.groupIndex` /
 * `details.accountId` (`ACCOUNT_GROUP_INVALID`). `null` si rien n'est lisible.
 */
export function readGroupRefusal(details: Record<string, unknown> | undefined): GroupRefusalTarget | null {
	if (!details) return null;
	const step = details.groupStep;
	const source =
		step !== null && typeof step === 'object' ? (step as Record<string, unknown>) : details;
	const { groupIndex, accountId } = source;
	if (typeof groupIndex !== 'number' || !Number.isInteger(groupIndex) || groupIndex < 0) return null;
	return { groupIndex, accountId: typeof accountId === 'number' ? accountId : null };
}
