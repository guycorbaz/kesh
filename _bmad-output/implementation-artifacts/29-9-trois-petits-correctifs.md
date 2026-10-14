# Story 29.9 : Trois petits correctifs — la suite de la liste de réconciliation, une bannière périmée, une suppression sans confirmation

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (`main`,
kickoff de l'Epic 29), branche `story/29-9`, worktree `/home/gcorbaz/devel/kesh-29-9`. Validation P1 à lancer
(prompt `29-9-validate-prompt-p1.md`). Choix consignés au registre `epic-29-choix-autonomes-29-9.md`, **C-29-9-1 à
C-29-9-12**.

10 AC, T0-T9. Trois issues, chacune **entièrement** réglée ou sortie du périmètre : [#649], [#655], [#646] — toutes
`bug` et `P3`.

⛔ **Consigne pour l'orchestrateur — la PR, et elle seule, porte `closes #649 #655 #646`** (corps de la PR ; le dépôt
merge en squash, et le squash recopie le corps des commits). Les commits de la branche portent **`refs #N`**, jamais
`closes`/`fixes`, même en prose ou au futur (B4 de la rétrospective 28). Avant la PR : `git log main..HEAD --format=%B
| grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\b"` doit être vide. Après merge, `gh issue view <N> --json state`
pour les trois.

## Story

**As a** Guy, qui veut que chaque release ait moins d'issues ouvertes que la précédente,
**I want** fermer trois défauts déjà diagnostiqués, chacun à son minimum,
**so that** le décompte baisse sans risque : la réconciliation dit qu'il reste des transactions et permet de les voir,
une page rechargée avec succès ne reste plus masquée par une erreur ancienne, et une personne de contact ne s'efface
plus d'un clic malheureux.

**Ajoutée à l'Epic 29 le 2026-10-14 sur décision de Guy**, pour tenir la règle de release de la v0.13.4 (moins
d'issues ouvertes qu'au tag v0.13.3 : 138).

**Effet sur le décompte** (fichier d'epic, § *Le décompte*) : les trois portent `bug` → **−3 issues, −3 bugs**.

## Le constat, au sol (`fcd2ae75`)

| Issue | Constat | Preuve relevée |
|---|---|---|
| [#649] | `GET /reconciliation/proposals` rend au plus `limit` transactions (défaut 100, plafond 500) et `hasMore` ; le frontend envoie toujours 100 et **ne lit pas** `hasMore` | `crates/kesh-api/src/routes/reconciliation.rs:63` (`DEFAULT_PROPOSALS_LIMIT = 100`), `:67` (`MAX_PROPOSALS_LIMIT = 500`), `:598-602` (filtre `> 0`, `min(MAX)`), `:618-631` (`limit + 1`, `has_more`, `truncate`) ; `:84-90` doc-comment de `has_more` : « v0.1 : structure présente mais pas de UI dédiée » ; `frontend/src/lib/features/reconciliation/reconciliation.api.ts:16-26` (`limit = 100`) ; `ReconciliationProposals.svelte:106` (`getProposals(bankAccountId)`) ; `git grep -n hasMore -- frontend/src ':!*.test.ts'` → seul `reconciliation.types.ts:55` |
| [#649] (tri) | Les plus récentes d'abord ; seules les `pending` non rejetées | `crates/kesh-db/src/repositories/reconciliation.rs:288-311` (`status = 'pending' AND auto_match_rejected_at IS NULL ORDER BY booking_date DESC, id DESC LIMIT ?`) |
| [#649] (textes) | La limite est déjà **écrite** à quatre endroits, qui resteront vrais ou se réécrivent | `docs/manual/fr/user-manual.tex:2045` (« les 100 plus récentes au plus »), `:2057` (« qui ne montre que les 100 plus récentes en attente : une transaction plus ancienne n'y reparaît qu'une fois les plus récentes traitées ») ; `docs/api-external.md:594` ; `reconciliation-unreject-success` ×4 (`fr-CH/messages.ftl:867`, `de/en/it-CH:819`) ; `CHANGELOG.md` § `[0.13.3]` (publié : **ne se touche pas**) |
| [#649] (test) | Le défaut de 100 et `hasMore = true` sont testés ; le **plafond** de 500 ne l'est pas | `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116` (`get_proposals_paginates_at_100_default`, 150 transactions) ; `grep -n "limit=" crates/kesh-api/tests/reconciliation_e2e.rs` → aucun appel avec `limit` |
| [#655] cas 1 | `load()` du détail d'import n'a **aucune** garde de séquence : la réponse la plus lente l'emporte | `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:47-70` ; relectures lancées par `UnrejectTransactionAction` (`:160`) et `CancelReconciliationDialog` (`:176`) ; le patron de garde existe à côté : `ReconciliationProposals.svelte:99-117` (`loadGen`) |
| [#655] cas 2 | `bank-import/profiles/[id]/+page.svelte` ne remet jamais `errorMessage` à `null` ni `isLoading` à `true` ; aucune garde de séquence | même fichier, l. 11 (déclaration), 16 et 28 (écritures), 44 (lecture) ; l'`$effect` `:13-31` |
| [#646] | *Supprimer* une personne de contact appelle `deleteContactPerson` sans confirmation | `frontend/src/lib/features/contacts/ContactPersonsManager.svelte:76-83` (`removePerson`), bouton `:111-119` (`contact-person-delete-{id}`) ; le composant est rendu dans le dialogue d'édition (`routes/(app)/contacts/+page.svelte:1097-1100`, dans `Dialog.Root` `:811`) |
| [#646] (sémantique) | « Supprimer » **archive** : `active = FALSE`, trace `contact_person.archived` ; aucune route ne désarchive | `crates/kesh-db/src/repositories/contact_persons.rs:158` ; `crates/kesh-api/src/routes/contact_persons.rs:295` ; routes `crates/kesh-api/src/lib.rs:426-431`, `:794` (création, modification, suppression, liste) |
| [#646] (tests) | Le composant n'a **aucun** test Vitest ni E2E ; l'inventaire des boutons rouges écrit « sans confirmation (#646, #648) » | `git grep -ln "ContactPersonsManager\|contact-person-delete" -- frontend` → le composant, la page, `i18n-harvest.test.ts:35`, `boutons-rouges-inventory.test.ts:231-234` |
| [#646] (manuel) | Le manuel décrit l'ajout de personnes, pas leur suppression | `docs/manual/fr/user-manual.tex:869-874` (`keshnote`) |

### Les prémisses des issues

- **#649 — exacte.** Précision : l'API sait déjà rendre jusqu'à 500 transactions ; seul l'écran est en défaut. Le
  geste minimal est donc **frontend** (plus un test Rust qui fige le contrat, C-29-9-4).
- **#655, cas 1 — exacte.**
- **#655, cas 2 — exacte sur le code, rectifiée sur l'effet** (C-29-9-8). L'issue dit « même symptôme » ; or l'`$effect`
  de la page des profils ne se rejoue que si `page.params.id` change **sans quitter la route**, et aucun lien de
  l'application ne mène d'un profil à un autre (`git grep -n "bank-import/profiles/" -- frontend/src ':!*.test.ts'`
  → `profiles/+page.svelte:12` vers `new`, `profiles/new/+page.svelte:8` → `goto` vers `[id]` — deux routes
  distinctes, donc un montage neuf ; `BankProfileList.svelte:83`, lien de la liste). Le défaut est **latent** ; il
  se corrige quand même (mécanique). **L'orchestrateur commente #655** pour le dire.
- **#646 — exacte.** Précision : la suppression est un archivage sans retour possible à l'écran — c'est ce qui rend la
  confirmation utile, et le texte de la confirmation le dit.

## Critères d'acceptation

### #649 — la liste de la réconciliation dit qu'il en reste, et montre la suite

**AC1 — S'il en reste, l'écran le dit et offre la suite (C-29-9-2).**

Dans `ReconciliationProposals.svelte`, branche « liste non vide », **après `</table>`** (`:401`) et avant
`<ManualMatchModal` (`:403`) :

- si `hasMore` est vrai **et** `limit < PROPOSALS_MAX_LIMIT` : un paragraphe `data-testid="reconciliation-more"`
  (`reconciliation-labels-more-hidden`, `{ $count }` = `proposals.length`) et un bouton
  `data-testid="reconciliation-show-older-btn"` (`reconciliation-actions-show-older`, `{ $max }` =
  `PROPOSALS_MAX_LIMIT`), désactivé pendant `busy` ;
- si `hasMore` est faux : rien (ni avis, ni bouton).

Style : celui des boutons secondaires voisins (`rounded border border-text px-4 py-2`, comme *Rejeter* `:291-299`),
jetons du thème seulement (B2 de l'Epic 27 : pas de couleur codée en dur), lisible en clair et en sombre.

**AC2 — Le bouton passe au plafond, et le plafond tient pour les rechargements suivants (C-29-9-5, C-29-9-6).**

- `let limit = $state(PROPOSALS_DEFAULT_LIMIT)` et `let hasMore = $state(false)` ; `load()` appelle
  `getProposals(bankAccountId, limit)` et, sur une réponse non périmée, pose `hasMore = r.hasMore` avec `proposals`.
- Le bouton ne fait **que** `limit = PROPOSALS_MAX_LIMIT` : l'`$effect` de `:118-120`, qui suit `limit` parce que
  `load()` le lit avant son premier `await`, recharge. ⚠️ **Ne pas appeler `load()` en plus** : deux requêtes.
  Commentaire au site, qui dit aussi que le `{#key selectedId}` de `routes/(app)/reconciliation/+page.svelte:73` remet
  `limit` à 100 à chaque changement de compte (« ⚠️ retirer ce `{#key}` exigerait de remettre `limit` à zéro »).
- Après un lot, une affectation manuelle ou un éclatement, `load()` relit avec le `limit` **courant** (la liste ne
  retombe pas à 100).
- La sélection est vidée par le rechargement (`:109`, inchangé ; C-29-9-5).

**AC3 — Au plafond, un avis sans bouton, qui ne conseille pas le rejet (C-29-9-3).**

Si `hasMore` est vrai **et** `limit >= PROPOSALS_MAX_LIMIT` : un paragraphe `data-testid="reconciliation-more-capped"`
(`reconciliation-labels-more-capped`, `{ $max }`), **aucun** bouton. Le texte nomme *accepter, affecter
manuellement, éclater* — **pas** *rejeter* (une transaction rejetée reste `pending` et bloque la clôture du compte :
fiche 29-6, #670).

**AC4 — Le contrat se lit d'un seul endroit de chaque côté, et un test Rust le fige (C-29-9-4).**

- `reconciliation.api.ts` exporte `PROPOSALS_DEFAULT_LIMIT = 100` et `PROPOSALS_MAX_LIMIT = 500`, avec un
  doc-comment qui renvoie aux constantes Rust ; `getProposals(bankAccountId, limit = PROPOSALS_DEFAULT_LIMIT)`.
- `routes/reconciliation.rs` : les doc-comments de `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT` (`:62-67`)
  nomment leur reflet frontend ; celui de `has_more` (`:86-89`) ne dit plus « v0.1 … pas de UI dédiée » mais ce que
  fait l'écran depuis la 29-9. **Aucun code de production Rust ne change.**
- Test neuf dans `crates/kesh-api/tests/reconciliation_e2e.rs`, à côté de `get_proposals_paginates_at_100_default`,
  même décor (`setup_company`, `make_new_tx`, `seed_bank_transactions`) :
  `get_proposals_limit_is_capped_at_500` — 501 transactions, `?limit=1000` → **500** rendues, `hasMore` **vrai** ; et
  `get_proposals_limit_500_returns_all_when_fewer` — 150 transactions, `?limit=500` → **150**, `hasMore` **faux**
  (le second état de l'écran).

### #655 — une relecture ancienne ne masque plus une page à jour

**AC5 — Détail d'un import : numéro de séquence (C-29-9-7).**

`routes/(app)/bank-import/[id]/+page.svelte`, `load()` (`:47-70`) : `let loadSeq = 0` ; `const mySeq = ++loadSeq` en
tête ; après l'`await`, `detail`, `errorMessage` et `loading` ne s'écrivent que si `mySeq === loadSeq` (dans le
`try`, le `catch` et le `finally`). La remise à `null` de la 28-5b (`:51`) et son commentaire restent ; un commentaire
dit le patron (`ReconciliationProposals.svelte`, `loadGen`) et la raison (#655).

**AC6 — Page d'un profil bancaire : remise à zéro et séquence (C-29-9-8).**

`routes/(app)/bank-import/profiles/[id]/+page.svelte`, `$effect` (`:13-31`) : en tête, `errorMessage = null` et
`isLoading = true` ; un numéro de séquence (même patron), qui ignore une réponse — succès comme échec — plus ancienne
que la dernière demandée. ⚠️ Les écritures d'état dans l'effet ne doivent **pas** le faire boucler : l'effet ne lit
que `page.params.id` (et le compteur, variable non réactive). Commentaire : défaut latent au sol (aucun lien d'un
profil à l'autre), corrigé pour le jour où il y en aura un.

### #646 — une personne de contact ne s'efface pas sans confirmation

**AC7 — Confirmation avant la suppression (C-29-9-10).**

`ContactPersonsManager.svelte`, `removePerson` : la fonction reçoit la personne (ou son nom) ; avant tout appel,
`if (!window.confirm(i18nMsg('contact-persons-confirm-delete', '…', { name: \`${p.firstName} ${p.lastName}\` })))
return;` — refus → **aucun** appel à `deleteContactPerson`, la liste et le message d'erreur inchangés. Acceptation →
comportement actuel. Le bouton, sa classe et son `data-testid` ne changent pas (l'inventaire des boutons rouges le
suit, `boutons-rouges-inventory.test.ts:231`).

### Textes, tests, preuves

**AC8 — Textes : i18n quatre locales, manuel FR et son PDF, API, CHANGELOG, inventaire.**

- **Quatre clés neuves**, dans les quatre `crates/kesh-i18n/locales/*/messages.ftl`, à côté de leurs voisines
  (`reconciliation-labels-*` `fr-CH:1208-1216`, `reconciliation-actions-*` `:1252`, `contact-persons-*`
  `:1806-1815`) ; le fallback du `.svelte` = la valeur `fr-CH`, au caractère près :

  | Clé | fr-CH |
  |---|---|
  | `reconciliation-labels-more-hidden` | `Seules les { $count } transactions en attente les plus récentes sont affichées : il en reste de plus anciennes.` |
  | `reconciliation-actions-show-older` | `Afficher aussi les plus anciennes (jusqu'à { $max } en tout)` |
  | `reconciliation-labels-more-capped` | `Les { $max } transactions en attente les plus récentes sont affichées, le maximum que cette page montre à la fois. Il en reste de plus anciennes : elles apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement ou éclatées.` |
  | `contact-persons-confirm-delete` | `Supprimer la personne de contact { $name } ? Aucun écran ne permet de la rétablir.` |

  de-CH, en-CH, it-CH : traductions fidèles, même vouvoiement / registre que les clés voisines du même fichier.
- **Une clé réécrite**, `reconciliation-unreject-success` ×4 : « …dont la liste montre **d'abord** les 100 plus
  récentes en attente. » (de : « zuerst », en : « first », it : « prima ») — le fallback de
  `UnrejectTransactionAction.svelte:49` suit ; `UnrejectTransactionAction.test.ts:147` lit la valeur `fr-CH` et suit
  seul.
- `frontend/src/lib/shared/i18n-keys.test.ts` : `sitesTotal` (`:634`, 2107 au sol) se **recompte à la source**
  après le dernier rebase (la 29-3, la 29-6 et la 29-8 en ajoutent), avec une ligne de commentaire datée qui donne les
  deux bornes et la commande, sur le modèle des lignes `:620-633`. `npm run lint-i18n-ownership` vert.
- **Manuel** `docs/manual/fr/user-manual.tex` :
  - `:2045` — « les 100 plus récentes au plus » devient : les 100 plus récentes d'abord ; s'il en reste, un avis le dit
    sous la liste et le bouton *Afficher aussi les plus anciennes* l'étend à 500 ; au-delà, les plus anciennes
    apparaissent à mesure que les premières sont acceptées, affectées ou éclatées ; le bouton vide la sélection ;
  - `:2057` — la parenthèse « qui ne montre que les 100 plus récentes… » renvoie à ce qui précède (plus de chiffre
    contradictoire) ;
  - `:869-874` — la `keshnote` des personnes de contact ajoute : *Supprimer* demande confirmation ; la personne quitte
    la fiche, et aucun écran ne la rétablit.
  - PDF régénéré (`make fr` dans `docs/manual/`, **sous le verrou**), contrôlé aplati (§ *Propagation*).
- **`docs/api-external.md:594`** : « (100 par défaut) » → « (100 par défaut, 500 au plus ; `hasMore` vaut `true` s'il
  en reste) » ; le lien #649 est retiré (l'issue sera close) ; le reste de la phrase, vrai pour l'API, ne change pas.
- **CHANGELOG** : dans la section de la v0.13.4 que porte `main` au moment du rebase (la 29-2 la crée — sinon
  `## [Non publié]`), sous `### Corrigé`, trois puces : la suite de la liste de réconciliation (#649), la bannière
  périmée du détail d'un import (#655), la confirmation de suppression d'une personne de contact (#646). La section
  `[0.13.3]` ne se touche pas.
- **`boutons-rouges-inventory.test.ts:233`** : motif → `'D — suppression d\'une personne de contact, confirmée (#646) ;
  texte rouge sans bordure (#648)'` (le motif dirait faux sinon ; l'entrée et ses jetons restent).

**AC9 — Tests Vitest rouges d'abord, E2E, captures.**

Vitest (écrits **avant** le correctif et vus rouges, sortie copiée au Dev Agent Record) :

- `ReconciliationProposals.test.ts` (patrons du fichier : `mockApi.getProposals`, `satisfies GetProposalsResponse`) :
  - V1 `hasMore: false` → ni `reconciliation-more`, ni bouton, ni `reconciliation-more-capped` ;
  - V2 `hasMore: true` (100 lignes) → avis avec « 100 », bouton présent ; `getProposals` appelé avec `(id, 100)` ;
  - V3 clic → `getProposals` appelé **exactement deux fois** en tout, la seconde avec `(id, 500)` ; liste de la
    seconde réponse rendue ;
  - V4 seconde réponse `hasMore: true` → `reconciliation-more-capped` présent, **aucun** bouton ; le texte ne
    contient pas « rejet » ;
  - V5 après le clic, un *Accepter* réussi recharge avec `(id, 500)` (pas 100) ;
  - aucun test existant ne vérifie les arguments de `getProposals` (`grep -n "toHaveBeenCalledWith"
    ReconciliationProposals.test.ts` au sol : aucun sur `getProposals`) ; `reconciliation.api.test.ts` vérifie l'URL
    (`limit=100` par défaut) et reste vert.
- `bank-import-detail-page.test.ts` :
  - V6 deux relectures (deux *Annuler le rejet*, patron du test `:171-217`) ; la **plus récente** réussit d'abord,
    la plus ancienne **échoue ensuite** → aucune bannière, lignes de la plus récente ;
  - V7 même décor, la plus ancienne **réussit** ensuite avec des données périmées → les lignes restent celles de la
    plus récente ;
  - le test existant `:171` reste vert (ordre direct).
- `profiles/[id]/profile-page.test.ts` : un `page` **réactif** (fixture `profile-page-state.svelte.ts` à côté du test,
  `export const fakePage = $state({ params: { id: '5' } })`, servie par le `vi.mock('$app/state', …)`) :
  - V8 id 5 en échec → message ; id 6 en succès → message disparu, formulaire rendu ;
  - V9 id 5 lent puis id 6 rapide ; 6 réussit, 5 échoue ensuite → aucun message, profil 6 ;
  - les deux tests existants restent verts. Si la fixture réactive se révèle impossible sous Vitest, le dire au Dev
    Agent Record avec l'erreur, et **ne pas** déclarer AC6 prouvé.
- `ContactPersonsManager.test.ts` (neuf ; `vi.stubGlobal('confirm', …)`, patron `BankProfileList.test.ts:57-58`) :
  - V10 refus → `deleteContactPerson` **non** appelé, la personne reste listée ; `confirm` reçu avec le nom ;
  - V11 acceptation → `deleteContactPerson(id)` appelé, liste relue.

E2E (Playwright, montage `docs/testing.md` § *Prérequis Playwright local*, port **3049**, base `kesh_e2e_299`) :

- `frontend/tests/e2e/reconciliation-show-older.spec.ts` (neuf, patron `reconciliation-unreject.spec.ts:79-121`) :
  import par l'API d'un CAMT **synthétique** de **501** transactions (le bloc `<Ntry>` de
  `camt053_v04_minimal.xml` répété, références, montants et contreparties uniques par run, dates **antérieures** à
  celles de toutes les autres fixtures E2E — établir au `grep` des fixtures), `confirmBalanceMismatch` ; page
  *Réconciliation*, compte sélectionné → 100 lignes, avis et bouton ; clic → 500 lignes, avis plafonné, aucun bouton ;
  la 150e de nos transactions (par ordre décroissant) absente avant, présente après (hypothèse écrite : moins de 350
  autres transactions en attente sur ce compte) ; import supprimé en `finally`. Captures clair et sombre des deux
  états (`capturePath` de `reconciliation-unreject.spec.ts:125`).
- `frontend/tests/e2e/contact-persons-delete.spec.ts` (neuf) : un contact *Entreprise* et une personne créés par
  l'API (noms uniques), dialogue d'édition ouvert ; `page.once('dialog', d => d.dismiss())` → la personne reste ;
  `page.once('dialog', d => d.accept())` → elle part, et `GET /api/v1/contacts/{id}/persons` (route `lib.rs:792-795`) ne la rend plus. Le message du dialogue contient le nom.
- **Aucune capture** de la confirmation (dialogue natif, invisible des captures Playwright ; C-29-9-10) — à déclarer.
  Pages #655 : rendu inchangé, aucune capture.
- **Suite E2E complète** au dernier commit de code, jugée fichier par fichier contre `docs/testing.md` § *Les échecs
  attendus*.

**AC10 — Mutations et gates.**

Mutations (motif vérifié unique avant écriture ; restauration `git checkout -- f && touch f`) :

| # | Mutation | Doit tuer |
|---|---|---|
| M1 | `hasMore = r.hasMore` retiré (reste `false`) | V2 |
| M2 | le bouton pose `limit = PROPOSALS_DEFAULT_LIMIT` | V3 |
| M3 | le bouton pose `limit` **et** appelle `load()` | V3 (deux appels → trois) |
| M4 | condition du bouton sans `limit < PROPOSALS_MAX_LIMIT` | V4 |
| M5 | `load()` appelle `getProposals(bankAccountId)` (sans `limit`) | V3, V5 |
| M6 | `MAX_PROPOSALS_LIMIT = 499` (Rust) | `get_proposals_limit_is_capped_at_500` |
| M7 | garde `mySeq !== loadSeq` retirée du `catch` du détail | V6 |
| M8 | garde retirée du `try` du détail | V7 |
| M9 | `errorMessage = null` retiré de l'effet du profil | V8 |
| M10 | garde de séquence retirée du `.catch` du profil | V9 |
| M11 | `if (!confirm(…)) return;` → `confirm(…);` | V10 |
| M12 | `return` placé **avant** l'appel quelle que soit la réponse | V11 |

Gates : gate ciblé entre les passes ; **au dernier commit de code** : `scripts/test-fast.sh` complet sur base
`kesh_299` remise à zéro (migrations du worktree, seed), frontend complet (`check`, `lint-i18n-ownership`,
`test:unit`, `build`), E2E complet — sous le verrou `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Ne déclarer
que ce qui a tourné, avec ses chiffres.

## Tasks / Subtasks

- [ ] **T0** — Montage : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-9/target` ; `node_modules` par `cp -al` depuis le
  dépôt principal ; bases `kesh_299` et `kesh_e2e_299` (migrations du worktree, `scripts/seed-dev-db.sql`) ; port
  3049. **Rebaser d'abord** sur `main` si la 29-3, la 29-6 ou la 29-8 y sont mergées, puis relire chaque numéro de
  ligne de cette fiche.
- [ ] **T1** — #649 contrat (AC4) : constantes frontend, doc-comments Rust, deux tests Rust ; M6 ; commit
  `test(29-9): … (refs #649)`.
- [ ] **T2** — #649 écran (AC1-AC3) : V1-V5 rouges, puis le correctif ; M1-M5 ; commit `fix(reconciliation): … (refs
  #649)`.
- [ ] **T3** — #655 (AC5, AC6) : V6-V9 rouges, puis les deux pages ; M7-M10 ; commit `fix(bank-import): … (refs #655)`.
- [ ] **T4** — #646 (AC7) : V10-V11 rouges, puis le correctif ; M11-M12 ; commit `fix(contacts): … (refs #646)`.
- [ ] **T5** — i18n (AC8) : quatre clés, une réécrite, ×4 ; `sitesTotal` recompté ; inventaire des boutons rouges.
- [ ] **T6** — Manuel FR, `make fr` sous le verrou, contrôle aplati ; `api-external.md` ; CHANGELOG ; commit
  `docs(29-9): … (refs #649, refs #646)`.
- [ ] **T7** — E2E (AC9) : deux specs, captures clair et sombre (hors dépôt, chemins au Dev Agent Record) ; commit
  `test(e2e): … (refs #649, refs #646)`.
- [ ] **T8** — Propagation (§ ci-dessous), puis gates complets au dernier commit de code (AC10).
- [ ] **T9** — Dev Agent Record, File List, Change Log ; statut `review` ; registre de sprint. Rapport : demander le
  push, la PR (`closes #649 #655 #646` au corps seulement), le commentaire de #655 (C-29-9-8), et la décision sur une
  issue de pagination au-delà de 500 (C-29-9-3).

## Dev Notes

### Dérogation à la règle de découpage

Modules touchés au sens de la règle : (1) le frontend — trois zones, `features/reconciliation`, `routes/(app)/
bank-import`, `features/contacts` ; (2) `kesh-api` — deux doc-comments et deux tests, **aucun code de production** ;
(3) les catalogues `kesh-i18n` ; (4) la documentation (manuel, PDF, `api-external.md`, CHANGELOG). **Quatre**, sous
la limite de cinq : le critère « scope cross-cutting » n'est pas atteint. Si l'on compte chaque zone du frontend
comme un module, on en compte six : la section tient alors lieu de la `Dérogation règle de splitting`, **déclarée au
Project Lead par le rapport de l'orchestrateur**. Dans les deux lectures, pas de découpage : les trois correctifs sont
indépendants, chacun tient en un ou deux fichiers de production, aucun ne change une règle métier, et la revue se
fait correctif par correctif (précédent : 28-8, C-28-8-1). Un correctif fautif **sort du périmètre** plutôt que de
retenir les deux autres. C-29-9-1.

### Zones partagées avec les fiches voisines (non mergées)

| Fiche | Ce qu'elle touche de commun | Ce que fait la 29-9 |
|---|---|---|
| 29-3 (#527) | `ReconciliationProposals.svelte` : en-tête `:8`, paragraphe de priorité **avant** la table ; `i18n-keys.test.ts` `sitesTotal` +1 ; manuel `:2045-2050` | ajout **après** `</table>` ; `sitesTotal` recompté ; phrase `:2045` réécrite **sur la version 29-3** |
| 29-6 (#665) | même composant : case sur toutes les lignes, *Accepter* grisé, indication dans la rangée des boutons, garde d'`onAccept` ; s'appuie sur « `load()` vide `selected` » (état (a)) ; manuel `:2050` | ne touche ni la case, ni la rangée, ni `onAccept` ; **garde** la remise à vide de `selected` (C-29-9-5) |
| 29-8 (#670) | même composant : troisième bouton de ligne, modale ; réécrit des phrases du manuel de la zone `:2043-2060` | ne touche pas la ligne de table ; rebase du manuel à la main |

**Ordre** : la 29-9 se rebase et se merge **après** les trois (C-29-9-11). Au rebase : jamais `--ours`/`--theirs` sur
les PDF, régénérer ; recompter `sitesTotal` à la source.

### Ce qui doit être préservé

- Le plafond serveur de 500 (M9 de la 8-4) et le défaut de 100 : aucun code de production Rust ne change.
- `loadGen` de `ReconciliationProposals.svelte` et la remise à vide de `selected` dans `load()`.
- La remise à `null` de la 28-5b dans le détail d'un import et le test `bank-import-detail-page.test.ts:171`.
- `{#key selectedId}` de la page de réconciliation.
- Le bouton de suppression d'une personne : classe, texte, `data-testid` (l'inventaire des boutons rouges et #648 le
  suivent).
- La section `[0.13.3]` du CHANGELOG (publiée).

### Exclusions écrites (périmètre minimal)

- **Pas de pagination serveur** au-delà de 500 (C-29-9-2, C-29-9-3) ; l'orchestrateur décide s'il ouvre une issue.
- **Pas de filtre par période** (proposé par #649 parmi d'autres moyens ; le bouton suffit).
- **#655** : seuls les deux sites de l'issue ; la classe n'est pas revendiquée fermée (C-29-9-9). Le détail d'import
  charge à l'`onMount` et ne relit pas sur changement de `page.params.id` — non atteignable aujourd'hui (aucun lien d'un
  import à un autre), **non traité**.
- **#646** : pas de désarchivage d'une personne, pas de dialogue maison ; l'aspect du bouton relève de #648.
- **Manuels DE/IT/EN** : vides (`docs/manual/{de,en,it}/`) ; rien à traduire.

### Propagation — le symptôme à greper après chaque patch

```sh
git grep -nE "\b100\b" -- docs/manual/fr/user-manual.tex | sed -n '/20[0-9][0-9]:/p'      # #649, zone réconciliation
git grep -nE "100 plus récentes|100 most recent|100 neuesten|100 più recenti" -- crates docs frontend/src
git grep -n "pas de UI dédiée\|Charger plus" -- crates/kesh-api/src                      # #649, doc-comments
git grep -n "hasMore" -- frontend/src ':!*.test.ts'                                     # #649 : types + composant
git grep -n "issues/649" -- docs README.md website                                      # lien retiré d'api-external
git grep -n "errorMessage" -- "frontend/src/routes/(app)/bank-import"                    # #655
git grep -n "sans confirmation" -- frontend/src docs                                    # #646
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "plus récentes.\{0,200\}"
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "personnes de contact.\{0,300\}"
```

Les occurrences légitimes (le CHANGELOG `[0.13.3]`, la valeur 100 des constantes) se trient à la main et se citent au
Dev Agent Record.

### Pièges connus du dépôt, applicables ici

- **Effet qui suit `limit`** (AC2) : poser `limit` puis appeler `load()` double la requête ; V3 l'attrape.
- **Fallbacks Svelte = valeur `fr-CH`**, au caractère près (apostrophe typographique ou droite : copier celle du
  `.ftl`).
- **Marques d'isolation Fluent** : `i18nMsg` les consomme avec la variable (`i18n.svelte.ts:22-27`) ; ne pas les
  re-coder dans un test.
- **jsdom n'a pas `window.confirm`** : `vi.stubGlobal` (patron `BankProfileList.test.ts:57`).
- **Base remise à zéro avant chaque gate complet** (KF-039), sur `kesh_299` seulement ; jamais `docker restart` du
  MariaDB partagé.
- **Gates lourds et `make` des manuels sous le verrou** ; tuer le backend E2E **par PID**.
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **Édition par l'outil Edit uniquement** ; aucun `sed -i`/`perl -pi` sur du code.
- **Données réelles** : fixtures CAMT synthétiques seulement ; aucune donnée du relevé de Guy.

### References

- [#649], [#655], [#646] (`gh issue view N`, lues le 2026-10-14).
- `crates/kesh-api/src/routes/reconciliation.rs:56-90`, `:578-631` ; `crates/kesh-db/src/repositories/reconciliation.rs:276-311` ;
  `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116`.
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `reconciliation.api.ts:16-26`,
  `reconciliation.types.ts:55`, `ReconciliationProposals.test.ts`.
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, `bank-import-detail-page.test.ts:171-217` ;
  `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte`, `profile-page.test.ts`.
- `frontend/src/lib/features/contacts/ContactPersonsManager.svelte` ; `BankProfileList.svelte:30`,
  `BankProfileList.test.ts:57-58` ; `crates/kesh-db/src/repositories/contact_persons.rs:158`.
- `docs/manual/fr/user-manual.tex:869-874`, `:2043-2060` ; `docs/api-external.md:594`.
- Fiches voisines : `/home/gcorbaz/devel/kesh-29-3/…/29-3-meilleure-proposition.md`,
  `/home/gcorbaz/devel/kesh-29-6/…/29-6-trop-percu-centimes.md`, `/home/gcorbaz/devel/kesh-29-8/…/29-8-transaction-reglee-ailleurs.md`.
- Précédent de forme : `28-8-petits-correctifs.md`. Consignes : `docs/agents-orchestration.md` ; rétrospective
  `epic-28-retro-2026-10-14.md` § 4, § 7.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie), sur `fcd2ae75`. Prémisse du second cas de
  #655 rectifiée (défaut latent, C-29-9-8). Choix C-29-9-1 à 12.

[#646]: https://github.com/guycorbaz/kesh/issues/646
[#649]: https://github.com/guycorbaz/kesh/issues/649
[#655]: https://github.com/guycorbaz/kesh/issues/655
