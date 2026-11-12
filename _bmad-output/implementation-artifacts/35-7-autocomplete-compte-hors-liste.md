# Story 35-7 : le champ de compte ne paraît plus vide quand un compte part au serveur (#1000)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Release **0.17.7**. **Aucune migration**, aucun changement serveur ni d'API, aucune clé
i18n. Code lu sur **`main`** (`4d39f377`, la 0.17.6 publiée). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -18,
-36, -37) ; `e35-choix-autonomes-e357.md` pour la story, à créer au premier choix. Validation non lancée.*

## Story

En tant que titulaire qui passe une écriture manuelle ou éclate une transaction bancaire, je veux que le champ du compte
de contrepartie montre toujours le compte qui partira au serveur — ou soit vide parce qu'aucun ne partira —, afin de ne
jamais envoyer un compte que je ne vois pas, ni recevoir un refus incompréhensible pour un champ qui paraît vide.

## Ce qui se lit au code (`main`)

**Le composant** — `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` (397 lignes) :
- `selected` (`:153`) résout `value` dans la liste **reçue** `accounts` ; `displayLabel` (`:167-171`) vaut `''` quand
  `selected` est `undefined` (hors `loadError`) ; le champ montre `query = draft ?? displayLabel` (`:179`).
- `handleBlur` (`:282-303`) : sans `allowClear`, **retour immédiat** (`:288`) — le brouillon reste affiché tel quel, la
  valeur ne change pas (« dette #271 » : un champ obligatoire n'est jamais nullifié au `blur`). Avec `allowClear`, un
  brouillon vide nullifie (`:291-297`), un texte libre est jeté (`:302`).
- L'invariant que le fichier se donne (`:120-150`, `:300-301`) — « le champ ne peut jamais contredire `value` » — n'est
  donc tenu **que** pour les consommateurs `allowClear`.

**Les consommateurs** (`git grep -n "<AccountAutocomplete" -- frontend/src`, sept sites) :
- **stricts** (sans `allowClear`) : `ManualMatchModal.svelte:179`, `TransactionSplitModal.svelte:288`,
  `VatPurchaseAssistant.svelte:159`, `:204`, `JournalEntryForm.svelte:393` ;
- **`allowClear`** : `routes/(app)/products/+page.svelte:697` (`:703`), `components/invoices/InvoiceForm.svelte:877` (`:881`).

**Chemin 1 de #1000 — le compte sort de la liste.** Les deux dialogues de réconciliation passent `accounts =
counterpartyAccounts(accounts, bankLedgerAccountId)` (`ManualMatchModal.svelte:88`, `TransactionSplitModal.svelte:98` ;
`reconciliation-counterparties.ts:25-30`). Tant que le parent n'a pas résolu `bankLedgerAccountId` (`null` : aucun filtrage
par identifiant, `:20-23`), le compte de la banque est proposé ; s'il est choisi puis que l'identifiant arrive, il sort de
la liste : `selected` devient `undefined`, le champ s'affiche **vide**, et `counterpartyId` (`ManualMatchModal.svelte:70`,
`:182`) ou `splits[i].counterpartyAccountId` (`TransactionSplitModal.svelte:73`, `:291`) **garde** l'identifiant. Le
bouton reste actif (`clientError`, `ManualMatchModal.svelte:106-121`, ne voit pas de `null`) ; le serveur refuse en 400
`counterparty_equals_bank_ledger`.

**Chemin 2 de #1000 — le texte effacé au clavier.** Sur un consommateur strict, vider le champ puis quitter le champ laisse
le champ vide et la valeur intacte (`:288`) : un compte invisible part au serveur. Même chose pour un texte libre tapé et
jamais choisi (le champ montre « 40 » ; la valeur est l'ancien compte, ou `null`).

**Tests existants** — `AccountAutocomplete.test.ts` (409 lignes) : `:99-109` « sans `allowClear` : vider le champ au
clavier ne nullifie RIEN (dette #271) » (n'asserte que `onSelect` non appelé ; reste vert) ; `:246-264` le texte libre
restauré **avec** `allowClear`. `ManualMatchModal.test.ts:183` (bouton désactivé si `counterpartyId` nul), `:276-289`
(filtre de la banque) ; `TransactionSplitModal.test.ts:315` (`monter(bankLedgerAccountId)`).

## Décisions

- **D1 — Chemin 1 : un compte que le dialogue ne propose plus est retiré de la sélection** (C-e350-36). Une fonction pure,
  dans `reconciliation-counterparties.ts`, écrite une fois pour les deux dialogues :
  `keepOfferedCounterparty(id, accounts, bankLedgerAccountId): number | null` — `null` si `id` est `null` ; `id` si le compte
  est **absent** de `accounts` (liste pas encore chargée : rien à juger, le serveur tranche, comme aujourd'hui) ; sinon `id`
  s'il est dans `counterpartyAccounts([compte], bankLedgerAccountId)`, `null` dans le cas contraire. Chaque dialogue l'applique
  par un `$effect` qui **n'écrit que si la valeur change** (`counterpartyId` ; chaque `splits[i].counterpartyAccountId`).
  Le champ est alors vide **et** la valeur nulle : le message existant « Compte de contrepartie obligatoire » s'applique, le
  bouton se désactive. Aucun texte neuf.
- **D2 — Chemin 2 : sur un champ strict, le `blur` jette le brouillon** (C-e350-37). `handleBlur` : en mode dégradé
  (`loadError`) rien ne change (le brouillon y est l'identifiant tapé, déjà envoyé par `handleInput`) ; sans `allowClear`,
  `draft = null` — le champ **réaffiche le compte retenu** (ou reste vide si la valeur est nulle), sans jamais appeler
  `onSelect` (la dette #271 reste tenue : un champ obligatoire n'est pas nullifié) ; avec `allowClear`, inchangé. Le
  commentaire `:285-287` est réécrit : l'invariant « le champ ne contredit jamais `value` » vaut pour tous.
- **D3 — Ce qui ne change pas** : `allowClear`, `markInvalid`, le mode dégradé, le filtre (`active`, `postable`,
  `requiredAccountType`), le bouton d'effacement, les deux consommateurs `allowClear`. Le serveur reste juge
  (`counterparty_equals_bank_ledger`, `ACCOUNT_NOT_POSTABLE`).

## Critères d'acceptation

1. **AC1 — Chemin 2, le composant** (`AccountAutocomplete.test.ts`) : sans `allowClear`, valeur `1` résolue : vider le champ
   puis `blur` → le champ affiche `1000 — Caisse`, `onSelect` jamais appelé ; taper `zzz` puis `blur` → idem ; valeur
   `null` : taper `zzz` puis `blur` → champ vide. Mode dégradé (`loadError`) : taper `7` puis `blur` → le champ garde `7`.
   Avec `allowClear` : les tests `:121-264` restent verts, inchangés.
2. **AC2 — Chemin 1, la fonction** (`reconciliation-counterparties.test.ts`, existant) : `null` → `null` ;
   compte absent de la liste → l'identifiant ; compte de la banque → `null` ; compte archivé ou non imputable → `null` ;
   compte proposé → l'identifiant ; `bankLedgerAccountId` nul → l'identifiant d'un compte actif et imputable.
3. **AC3 — Chemin 1, les dialogues** (Vitest, `ManualMatchModal.test.ts` et `TransactionSplitModal.test.ts`) : monté avec
   `bankLedgerAccountId: null`, le compte de la banque choisi, puis le prop passé à son identifiant (`rerender`) → le champ
   est vide, la valeur nulle (bouton de validation désactivé, message « Compte de contrepartie obligatoire » pour
   *Écriture manuelle* ; pour *Éclater*, la ligne concernée seule vidée, l'autre intacte) ; un compte resté proposé n'est
   pas touché. Aucun appel serveur.
4. **AC4 — Consommateurs stricts, non-régression** : les suites de `JournalEntryForm`, `VatPurchaseAssistant`,
   `ManualMatchModal`, `TransactionSplitModal` restent vertes ; une ligne d'écriture dont on vide le compte puis qu'on quitte
   réaffiche son compte (un test ajouté dans `JournalEntryForm.edit.test.ts`, ou constat que le composant seul suffit, écrit au
   Change Log).
5. **AC5 — Textes et docs** : aucune clé i18n ajoutée ni changée (`sitesTotal` de `i18n-keys.test.ts` inchangé, recompté) ;
   le manuel ne décrit pas le comportement du champ au clavier — contrôle :
   `git grep -nE "vide[rz]? le champ|champ vide|effac" -- docs/manual/fr/user-manual.tex` ; toute phrase qui le
   contredirait est corrigée, sinon le Change Log dit « manuel inchangé, grep cité ». CHANGELOG `## [0.17.7] — Non
   publié`, *Corrigé*, un point `refs #1000` (C-e350-8).
6. **AC6 — Captures** (thème clair, base jetable, `kesh-gate-logs/e357/captures/`) : *Écriture manuelle*, champ vidé puis
   quitté (le compte réaffiché) ; *Éclater*, même geste.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, journal `kesh-gate-logs/e357/rouges.log`. Les tests montent les composants avec un mock local
d'`i18nMsg` (`vi.mock('$lib/shared/utils/i18n.svelte', …)`, patron de `AccountAutocomplete.test.ts:21-23`), sans état
i18n global. Rouges au départ : AC1 (le champ reste vide, ou garde `zzz`) ; AC3 (le champ vide, la valeur gardée, le
bouton actif) ; AC2 : la fonction est d'abord écrite comme `(id) => id` (la lecture d'aujourd'hui), pour que ses rouges portent sur
le comportement et non sur un export absent ; puis remplacée. Verts au départ, et dits : `:99-109`, les tests `allowClear`, le mode dégradé.

**Mutations** : M1 `draft = null` retiré du `blur` strict (AC1) ; M2 le `blur` strict nullifie (`onSelect(null)`) — AC1
(`onSelect` jamais appelé) ; M3 le mode dégradé jette aussi son brouillon (AC1, `7`) ; M4 `keepOfferedCounterparty` rend
toujours l'identifiant (AC2, AC3) ; M5 la fonction rend `null` pour un compte absent de la liste (AC2 — un compte non
chargé ne doit pas être effacé) ; M6 l'`$effect` d'*Éclater* vide toutes les lignes (AC3, l'autre ligne intacte) ; M7
l'`$effect` d'*Écriture manuelle* retiré (AC3).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur `main` ; `git grep -n "<AccountAutocomplete" -- frontend/src` (sept sites).
- [ ] Tests, rouges consignés ; `AccountAutocomplete.svelte` (D2), `reconciliation-counterparties.ts` (D1), les deux
      dialogues ; CHANGELOG ; captures.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au dernier
      commit de code (`reconciliation-manual.spec.ts`, `reconciliation-manual-entry.spec.ts`, `reconciliation-split.spec.ts`, `journal-entries.spec.ts` en particulier).
      Backend : rien ne change côté Rust ; le gate backend complet n'est dû qu'au push de la branche d'intégration.

## Dev Notes

### Hors périmètre

- Afficher un compte hors liste avec un marqueur (seconde piste de #1000) : écartée, C-e350-36.
- Le `select` natif des réglages (#271, fermée) : autre objet.

### Fichiers communs — règle d'union C-e350-8

`CHANGELOG.md` (un point sous `### Corrigé`). Aucun autre : `features/reconciliation/` est aussi touché par la **35-4**
(`ReconciliationProposals.svelte`, `AlreadyBookedConfirmDialog.svelte`, `already-booked.ts`) — **fichiers distincts**, pas
de hunk commun ; la 35-7 touche `ManualMatchModal.svelte`, `TransactionSplitModal.svelte`, `reconciliation-counterparties.ts`
et leurs tests. Si l'une des deux voit un fichier de l'autre à toucher, elle le dit avant de l'écrire.

### Ce qui doit rester vrai

- Un champ obligatoire n'est jamais nullifié par un simple `blur` (#271). Le champ ne contredit jamais la valeur envoyée.
  Rien n'est fait d'office au serveur. Fixtures synthétiques seulement.

### Références

- Issue #1000 ; Story 34-3 (relevé E1-5 de sa revue P1) ; Story 16-1b (`allowClear`, `markInvalid`) ; #271.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `main`, `4d39f377`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
