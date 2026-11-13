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
  Le champ est alors vide **et** la valeur nulle : le message existant de chaque dialogue s'applique — *Écriture manuelle* :
  « Compte de contrepartie obligatoire » (`reconciliation-manual-error-counterparty-required`, `ManualMatchModal.svelte:106-111`),
  bouton désactivé ; *Éclater* : « Ligne { $line } : compte requis » (`reconciliation-split-error-account-required`,
  `TransactionSplitModal.svelte:147-173`, première ligne en défaut). Aucun texte neuf. **Limite écrite** (validation P1,
  V1-4) : le champ se vide de lui-même quand l'identifiant du compte de la banque arrive (résolution au montage,
  `ReconciliationProposals.svelte:217-229`), fenêtre courte ; seul ce message dit pourquoi — C-e350-36 tient.
- **D2 — Chemin 2 : sur un champ strict, le `blur` jette le brouillon** (C-e350-37). `handleBlur` : en mode dégradé
  (`loadError`) rien ne change (le brouillon y est l'identifiant tapé, déjà envoyé par `handleInput`) ; sans `allowClear`,
  `draft = null` — le champ **réaffiche le compte retenu** (ou reste vide si la valeur est nulle), sans jamais appeler
  `onSelect` (la dette #271 reste tenue : un champ obligatoire n'est pas nullifié) ; avec `allowClear`, inchangé.
  **Les textes qui disent « les 4 consommateurs … inchangés » deviennent faux** (validation P1, V1-1 ; le compte lui-même est
  périmé : **sept** sites, cinq stricts et deux `allowClear`). Inventaire relevé sur `main`
  (`/usr/bin/grep -n "4 consommateurs\|strictement inchangé\|stricts inchangés"` sur le composant et son test) :
  - **réécrits** : `AccountAutocomplete.svelte:20-23` (doc d'`allowClear` : « Défaut `false` : les 4 consommateurs existants
    sont stricts inchangés ») ; `:285-287` (« le brouillon est conservé tel quel — comportement des 4 consommateurs existants
    strictement inchangé ») — l'invariant « le champ ne contredit jamais `value` » vaut pour tous ;
    `AccountAutocomplete.test.ts:8` (en-tête : tests « orientés non-régression des 4 consommateurs existants ») ; `:81`
    (`describe` « non-régression des 4 consommateurs ») et `:99` (titre : « vider le champ au clavier ne nullifie RIEN ») —
    le test `:99` garde son assertion (`onSelect` jamais appelé) et gagne celle d'AC1 (le champ réaffiche le compte) ;
  - **gardés, avec leur raison** (le compte « 4 » corrigé en « consommateurs » sans changer le sens) : `:183` (l'effet qui
    annule le brouillon sur une valeur venue de l'extérieur : inchangé) ; `:199` (`requiredAccountType` à `undefined` par
    défaut : inchangé) ; test `:279` (même objet).
  L'étape 0 relance le grep et le Change Log en cite la sortie.
- **D3 — Ce qui ne change pas** : `allowClear`, `markInvalid`, le mode dégradé, le filtre (`active`, `postable`,
  `requiredAccountType`), le bouton d'effacement, les deux consommateurs `allowClear`. Le serveur reste juge
  (`counterparty_equals_bank_ledger`, `ACCOUNT_NOT_POSTABLE`).

## Critères d'acceptation

1. **AC1 — Chemin 2, le composant** (`AccountAutocomplete.test.ts`) : sans `allowClear`, valeur `1` résolue : vider le champ
   puis `blur` → le champ affiche `1000 — Caisse`, `onSelect` jamais appelé ; taper `zzz` puis `blur` → idem ; valeur
   `null` : taper `zzz` puis `blur` → champ vide. Mode dégradé (`loadError`) : taper `7` puis `blur` → le champ garde `7`
   (`onSelect` simulé par un `vi.fn()` nu, `value` **non** mise à jour — sinon l'effet `:185-190` réaffiche `7` avec ou sans
   M3, qui survivrait).
   Avec `allowClear` : les tests `:121-264` restent verts, inchangés.
2. **AC2 — Chemin 1, la fonction** (`reconciliation-counterparties.test.ts`, existant) : `null` → `null` ;
   compte absent de la liste → l'identifiant ; compte de la banque → `null` ; compte archivé ou non imputable → `null` ;
   compte proposé → l'identifiant ; `bankLedgerAccountId` nul → l'identifiant d'un compte actif et imputable.
3. **AC3 — Chemin 1, les dialogues** (Vitest, `ManualMatchModal.test.ts` et `TransactionSplitModal.test.ts`) : monté avec
   `bankLedgerAccountId: null`, le compte de la banque choisi, puis le prop passé à son identifiant (`rerender`) → le champ
   est vide, la valeur nulle — *Écriture manuelle* : bouton de validation désactivé, message « Compte de contrepartie
   obligatoire » ; *Éclater* : la ligne concernée seule vidée, l'autre intacte ; la ligne 1 vidée → « Ligne 1 : compte
   requis » ; la ligne 2 vidée seule, la ligne 1 remplie (compte **et** montant > 0 : `clientError` teste, ligne par ligne,
   le compte puis le montant, `TransactionSplitModal.svelte:166-182`) → « Ligne 2 : compte requis » (aucune ligne antérieure ne masque le
   message). Un compte resté proposé n'est pas touché. Aucun appel serveur.
4. **AC4 — Consommateurs stricts, non-régression** : les suites de `JournalEntryForm`, `ManualMatchModal`,
   `TransactionSplitModal` restent vertes ; `VatPurchaseAssistant` n'a **pas** de suite de composant (`vat-purchase.test.ts`
   ne teste que ses fonctions pures) : la couverture de ses deux champs stricts (`:159`, `:204`) est celle du composant seul
   (AC1), écrit au Change Log ; une ligne d'écriture dont on vide le compte puis qu'on quitte
   réaffiche son compte (un test ajouté dans `JournalEntryForm.edit.test.ts`, ou constat que le composant seul suffit, écrit au
   Change Log).
5. **AC5 — Textes et docs** : aucune clé i18n ajoutée ni changée (`sitesTotal` de `i18n-keys.test.ts` inchangé, recompté) ;
   le manuel ne décrit pas le comportement du champ au clavier. Trois sites décrivent le champ de contrepartie
   (`user-manual.tex`, sur `main`) : `:3121` (*Écriture manuelle* : ce que le sélecteur propose) ; `:3180` (*Éclater* : « choisi
   dans le même sélecteur que l'écriture manuelle ») ; `:3186` (« La contrepartie n'est jamais le compte de la banque » :
   « Ce filtrage n'est toutefois pas complet »). Contrôle sur `:3105-3190`
   (`sed -n 3105,3190p docs/manual/fr/user-manual.tex | /usr/bin/grep -nE "vide|effac|quitt|tap|filtrage"`). **Tranché**
   (validation P2, placement précisé en P3) : `:3186` gagne une demi-phrase **après** « c'est le refus qui protège le grand
   livre », la proposition d'origine gardée entière — « … pas complet~: c'est le refus qui protège le grand livre~; un compte
   de la banque choisi avant que Kesh l'ait identifié est ensuite retiré du champ, qui se vide. » —, `:3121` et `:3180`
   restent tels quels (ils ne disent rien du
   champ quand on le quitte, que D2 change). PDF régénéré, contrôlé aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`) :
   présence de « retiré du champ » (sans ligature). CHANGELOG `## [0.17.7] — Non
   publié`, *Corrigé*, un point `refs #1000` (C-e350-8).
6. **AC6 — Captures** (thème clair, base jetable, `kesh-gate-logs/e357/captures/`) : *Écriture manuelle*, champ vidé puis
   quitté (le compte réaffiché) ; *Éclater*, même geste.
7. **AC7 — E2E du chemin 2** (`reconciliation-manual-entry.spec.ts`, montage de `:112-140`) : le compte `2000` choisi
   (`option.dispatchEvent('mousedown')`, `:137`), puis `champ.fill('zzz')` et `champ.blur()` → le champ affiche de nouveau
   `2000 — …` ; *Passer l'écriture* → le corps du `POST /api/v1/reconciliation/manual` (lu par `page.waitForRequest`) porte
   l'identifiant du compte **affiché**. Rouge au départ : le champ reste à `zzz`. Le **chemin 1** (course sur
   `bankLedgerAccountId` au montage) n'est pas déterministe en E2E : **limite assumée**, couvert par Vitest seul (AC3).

## Tests à écrire, et la preuve du rouge

Écrits d'abord, journal `kesh-gate-logs/e357/rouges.log`. Les tests montent les composants avec un mock local
d'`i18nMsg` (`vi.mock('$lib/shared/utils/i18n.svelte', …)`, patron de `AccountAutocomplete.test.ts:21-23`), sans état
i18n global. Rouges au départ : AC1 (le champ reste vide, ou garde `zzz`) ; AC3 (le champ vide, la valeur gardée, le
bouton actif) ; AC7 (le champ reste à `zzz`) ; AC2 : la fonction est d'abord écrite comme `(id) => id` (la lecture d'aujourd'hui), pour que ses rouges portent sur
le comportement et non sur un export absent ; puis remplacée. Verts au départ, et dits : `:99-109`, les tests `allowClear`, le mode dégradé.

**Mutations** : M1 `draft = null` retiré du `blur` strict (AC1) ; M2 le `blur` strict nullifie (`onSelect(null)`) — AC1
(`onSelect` jamais appelé) ; M3 le mode dégradé jette aussi son brouillon (AC1, `7`) ; M4 `keepOfferedCounterparty` rend
toujours l'identifiant (AC2, AC3) ; M5 la fonction rend `null` pour un compte absent de la liste (AC2 — un compte non
chargé ne doit pas être effacé) ; M6 l'`$effect` d'*Éclater* vide toutes les lignes (AC3, l'autre ligne intacte) ; M7
l'`$effect` d'*Écriture manuelle* retiré (AC3) ; M1 est aussi tuée par AC7, à la frontière HTTP.

## Tâches

- [ ] Étape 0 : relire les lignes citées sur `main` ; `git grep -n "<AccountAutocomplete" -- frontend/src` (sept sites) ;
      le grep des « 4 consommateurs » de D2, sortie citée au Change Log.
- [ ] Tests, rouges consignés ; `AccountAutocomplete.svelte` (D2, avec ses commentaires) et son test ;
      `reconciliation-counterparties.ts` (D1), les deux dialogues ; l'E2E d'AC7 ; manuel (`user-manual.tex:3186`) et PDF
      régénéré, contrôlé aplati (AC5) ; CHANGELOG ; captures.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au dernier
      commit de code (`reconciliation-manual.spec.ts`, `reconciliation-manual-entry.spec.ts`, `reconciliation-split.spec.ts`, `journal-entries.spec.ts` en particulier).
      Backend : rien ne change côté Rust ; le gate backend complet n'est dû qu'au push de la branche d'intégration.

## Dev Notes

### Hors périmètre

- Afficher un compte hors liste avec un marqueur (seconde piste de #1000) : écartée, C-e350-36.
- Le `select` natif des réglages (#271, fermée) : autre objet.

### Fichiers communs — règle d'union C-e350-8

`CHANGELOG.md` (un point sous `### Corrigé`) et `docs/manual/fr/user-manual.tex` (`:3186`, avec la **35-4**, qui touche
`:3049-3050`, `:3097` et `:3107` — paragraphes distincts, voisins ; le PDF se régénère après l'union, règle C-e350-8 point 6).
Pour le code, aucun autre : `features/reconciliation/` est aussi touché par la **35-4**
(`ReconciliationProposals.svelte`, `AlreadyBookedConfirmDialog.svelte`, `already-booked.ts`) — **fichiers distincts**, pas
de hunk commun ; la 35-7 touche `ManualMatchModal.svelte`, `TransactionSplitModal.svelte`, `reconciliation-counterparties.ts`
et leurs tests. Si l'une des deux voit un fichier de l'autre à toucher, elle le dit avant de l'écrire. ⚠️ La fiche 35-4
**cite** des lignes de ces deux dialogues (`ManualMatchModal.svelte:139-151`, `TransactionSplitModal.svelte:234-239`, pour le
refus d'une ligne déjà rapprochée) : les `$effect` de la 35-7 les décalent ; celle des deux qui part la seconde relit ces
citations après la fusion de la première. La spec `reconciliation-manual-entry.spec.ts` est éditée ici (AC7) ; la 35-4 ne
fait que la rejouer. Écrit aussi au fichier d'epic (validation P1, V1-6).

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
  **Écart à la consigne Edit** : six corrections de rédaction (références de ligne de `reconciliation-counterparties.ts`,
  noms de fichiers de tests existants, liste des specs E2E, rouge d'AC2), faites dans la fiche neuve par un court script
  Python qui contrôlait l'unicité de chaque chaîne avant de la remplacer, au lieu de l'outil Edit. Rien d'autre.
- 2026-11-13 — **Validation P1** (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW (`kesh-gate-logs/e35-7-validation-p1.md`).
  Remédiée par l'agent de rédaction des fiches (Opus 5.5). Prémisses relues sur `main` : `AccountAutocomplete.svelte:20`,
  `:183`, `:199`, `:286-287` et son test `:8`, `:81`, `:279` (grep « 4 consommateurs ») ; `TransactionSplitModal.svelte:147-173`
  (`reconciliation-split-error-account-required`) ; `reconciliation-manual-entry.spec.ts:112-140` ; `user-manual.tex:3121` ;
  `ReconciliationProposals.svelte:215-229`. Toutes confirmées.
  - V1-1 (MEDIUM) : inventaire des textes rendus faux par D2 écrit dans D2 (réécrits : `:20-23`, `:285-287`, test `:8`, `:81`,
    `:99` ; gardés avec leur raison : `:183`, `:199`, test `:279`) ; grep à l'étape 0.
  - V1-2 (MEDIUM) : AC7, E2E déterministe du chemin 2 dans `reconciliation-manual-entry.spec.ts` (le corps du
    `POST /reconciliation/manual` porte le compte affiché) ; chemin 1 : limite assumée, Vitest seul.
  - LOW : V1-3 (message d'*Éclater* « Ligne { $line } : compte requis », AC3) ; V1-4 (limite du champ qui se vide au
    montage, écrite en D1) ; V1-5 (contrôle du manuel restreint à `:3105-3135`, `:3121` nommée) ; V1-6 (ligne 35-7 du
    fichier d'epic et point 6 de ses dépendances : 35-7 et 35-4).
  - Symptômes greppés (fiche, fichier d'epic, registre) : « 4 consommateurs », « Compte de contrepartie obligatoire » (dit
    désormais pour *Écriture manuelle* seule), « manuel inchangé, grep cité » — remplacés ; registre : C-e350-36 dit « le
    message existant « Compte de contrepartie obligatoire » s'applique », annoté.
- 2026-11-13 — **Validation P2** (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW (`kesh-gate-logs/e35-7-validation-p2.md`) ;
  série 2 → 1 MEDIUM, né de la remédiation P1. Remédiée au plus court. Prémisses relues sur `main` : `user-manual.tex:3180`,
  `:3186` ; `TransactionSplitModal.svelte:166-182` ; `git grep -ln VatPurchaseAssistant -- 'frontend/src/**/*.test.ts'`
  (aucune suite de composant). Confirmées.
  - V2-1 (MEDIUM) : AC5 nomme les trois sites (`:3121`, `:3180`, `:3186`), contrôle sur `:3105-3190`, PDF aplati ; tranché :
    `:3186` gagne une demi-phrase ; union avec la 35-4 écrite (Fichiers communs, C-e350-8 annoté).
  - LOW : V2-2 (AC3 : ligne 1 avec compte et montant) ; V2-3 (AC4 : `VatPurchaseAssistant` sans suite de composant) ;
    V2-4 (AC1 : mode dégradé avec `onSelect` simulé, `value` non mise à jour).
  - Symptôme greppé : `3105-3135` — ne reste que dans l'entrée passée de P1.
- 2026-11-13 — **Validation P3, passe ciblée** (Sonnet) sur la remédiation P2 : 0 finding au-dessus de LOW, 2 LOW
  (`kesh-gate-logs/e35-7-validation-p3-ciblee.md`). Appliqués : V3-1 (manuel `:3186` et PDF aux *Tâches* ;
  `user-manual.tex` à la ligne 35-7 du fichier d'epic) ; V3-2 (la demi-phrase placée après « c'est le refus qui protège le
  grand livre », la proposition d'origine gardée entière). Rédaction seule.
- 2026-11-13 — **Validation terminée.** Série des findings au-dessus de LOW : P1 2 MEDIUM → P2 1 → P3 0 (CRITICAL et HIGH :
  0). Modèles : Sonnet (P1), Opus (P2), Sonnet en passe ciblée (P3). La boucle se clôt sur une passe sans finding au-dessus
  de LOW ; la remédiation de P3 ne touche que la rédaction. Choix retenus : **C-e350-36** (un compte que le dialogue ne
  propose plus est retiré de la sélection) et **C-e350-37** (sur un champ obligatoire, quitter le champ jette le brouillon).
  Statut : `ready-for-dev`.
