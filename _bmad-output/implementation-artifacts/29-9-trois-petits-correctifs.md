# Story 29.9 : Quatre petits correctifs — la suite de la liste de réconciliation, une bannière périmée, une suppression sans confirmation, des cases sans nom

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (`main`,
kickoff de l'Epic 29), branche `story/29-9`, worktree `/home/gcorbaz/devel/kesh-29-9`. Validation P1 (Sonnet,
lentilles R et F : 0/0/4/5 et 0/0/5/6) remédiée le 2026-10-14 ; validation P2 (Opus, lentilles R et F : 0/0/1/8
et 0/0/2/7) remédiée le 2026-10-14 ; passe ciblée P3 à lancer (prompt `29-9-validate-prompt-p3-ciblee.md`). Choix
consignés au registre `epic-29-choix-autonomes-29-9.md`, **C-29-9-1 à C-29-9-18**.

*Annotation du 2026-10-14 (amendement #678, décision de l'orchestrateur)* : la passe ciblée P3 a eu lieu et a clos la
validation des trois premiers correctifs (0/0/0/1, Change Log). Un **quatrième** entre ensuite dans la story, [#678]
(`bug`, `P4`) — accessibilité de la table de réconciliation —, sous l'**AC11** et la tâche **T10** ; les AC1-AC10 ne
changent pas, sauf des renvois datés vers l'AC11 (AC8 : CHANGELOG, `sitesTotal` ; AC10 : gates). Choix **C-29-9-19 à
C-29-9-25** (24 : forme du site, P4-3 ; 25 : repli tenu par V19/V20, P5-1). Passe ciblée P4 (0/0/2/6) puis P5 (0/0/1/5)
de validation, sur le seul diff de la remédiation précédente, toutes deux remédiées : prompts `29-9-validate-prompt-p4-ciblee.md`
et `29-9-validate-prompt-p5-ciblee.md`.
⚠️ **La clé reste `29-9-trois-petits-correctifs`** (fichier, `sprint-status.yaml`, registre, prompts) : seul le titre
dit « quatre » (C-29-9-19).

**review** — *annotation du 2026-10-15 (développement, Opus 5.5, autonomie)* : branche rebasée sur `4b358373` (tête
de `story/29-8`, non mergée) ; T0-T10 faits, dernier commit de code `284c1d3b` ; gates complets et E2E complet à ce
commit (Dev Agent Record). Choix nés du développement : **C-29-9-26 à C-29-9-30**. Revue de code P1 à lancer : prompt
`29-9-review-prompt-p1.md`.
*Annotation du 2026-10-15 (report, puis revue P1)* : la branche a été **reportée** sur `1261c4bb` (tête finale de la
29-8, revue close, PR #681) ; le dernier commit de code est **`3e5dfed8`**, les gates complets et l'E2E complet ont
été rejoués à `d2c64df9` (journaux `299-rebase2-*`, Change Log). La revue P1 est faite et remédiée (Change Log).

11 AC, T0-T10. Quatre issues, chacune **entièrement** réglée ou sortie du périmètre : [#649], [#655], [#646] — `bug`
et `P3` — et [#678] — `bug` et `P4`.

⛔ **Consigne pour l'orchestrateur — la PR, et elle seule, porte `closes #649 #655 #646 #678`** (corps de la PR ; le dépôt
merge en squash, et le squash recopie le corps des commits). Les commits de la branche portent **`refs #N`**, jamais
`closes`/`fixes`, même en prose ou au futur (B4 de la rétrospective 28). Avant la PR : `git log main..HEAD --format=%B
| grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+(([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)?#[0-9]+|https://github\.com/\S+/issues/[0-9]+)"` doit être vide (la forme sans `#N` rend le substantif
« validation close » du commit P3 : un faux positif). Le message de squash se rédige à la main : GitHub y recopie les
messages des commits, il faut donc y vérifier l'absence de mot-clé hors du seul `closes` voulu. Après merge, `gh issue view <N> --json state`
pour les quatre.

## Story

**As a** Guy, qui veut que chaque release ait moins d'issues ouvertes que la précédente,
**I want** fermer quatre défauts déjà diagnostiqués (le quatrième, #678, ajouté le 2026-10-14), chacun à son minimum,
**so that** le décompte baisse sans risque : la réconciliation dit qu'il reste des transactions et permet de les voir,
une page rechargée avec succès ne reste plus masquée par une erreur ancienne, et une personne de contact ne s'efface
plus d'un clic malheureux.

**Ajoutée à l'Epic 29 le 2026-10-14 sur décision de Guy**, pour tenir la règle de release de la v0.13.4 (moins
d'issues ouvertes qu'au tag v0.13.3 : 138).

**Effet sur le décompte** (fichier d'epic, § *Le décompte*) : les trois portent `bug` → **−3 issues, −3 bugs**.

*Annotation du 2026-10-14 (amendement #678)* : un quatrième défaut entre dans la story ; #678 porte `bug` aussi →
**−4 issues, −4 bugs**. Et un lecteur d'écran sait quelle transaction il coche, et entend qu'une ligne sans
proposition ne peut pas être acceptée.

## Le constat, au sol (`fcd2ae75`)

| Issue | Constat | Preuve relevée |
|---|---|---|
| [#649] | `GET /reconciliation/proposals` rend au plus `limit` transactions (défaut 100, plafond 500) et `hasMore` ; le frontend envoie toujours 100 et **ne lit pas** `hasMore` | `crates/kesh-api/src/routes/reconciliation.rs:63` (`DEFAULT_PROPOSALS_LIMIT = 100`), `:67` (`MAX_PROPOSALS_LIMIT = 500`), `:598-602` (filtre `> 0`, `min(MAX)`), `:618-631` (`limit + 1`, `has_more`, `truncate`) ; `:84-90` doc-comment de `has_more` : « v0.1 : structure présente mais pas de UI dédiée » ; `frontend/src/lib/features/reconciliation/reconciliation.api.ts:16-26` (`limit = 100`) ; `ReconciliationProposals.svelte:106` (`getProposals(bankAccountId)`) ; `git grep -n hasMore -- frontend/src ':!*.test.ts'` → seul `reconciliation.types.ts:55` |
| [#649] (tri) | Les plus récentes d'abord ; seules les `pending` non rejetées | `crates/kesh-db/src/repositories/reconciliation.rs:288-311` (`status = 'pending' AND auto_match_rejected_at IS NULL ORDER BY booking_date DESC, id DESC LIMIT ?`) |
| [#649] (textes) | La limite est déjà **écrite** à quatre endroits, qui resteront vrais ou se réécrivent | `docs/manual/fr/user-manual.tex:2045` (« les 100 plus récentes au plus »), `:2057` (« qui ne montre que les 100 plus récentes en attente : une transaction plus ancienne n'y reparaît qu'une fois les plus récentes traitées ») ; `docs/api-external.md:594` ; `reconciliation-unreject-success` ×4 (`fr-CH/messages.ftl:867`, `de/en/it-CH:819`) ; `CHANGELOG.md` § `[0.13.3]` (publié : **ne se touche pas**) |
| [#649] (test) | Le défaut de 100 et `hasMore = true` sont testés ; le **plafond** de 500 ne l'est pas | `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116` (`get_proposals_paginates_at_100_default`, 150 transactions) ; `grep -n "limit=" crates/kesh-api/tests/reconciliation_e2e.rs` → aucun appel avec `limit` |
| [#655] cas 1 | `load()` du détail d'import n'a **aucune** garde de séquence : la réponse la plus lente l'emporte | `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:47-70` ; relectures lancées par `UnrejectTransactionAction` (`:160`) et `CancelReconciliationDialog` (`:176`) ; le patron de garde existe à côté : `ReconciliationProposals.svelte:99-117` (`loadGen`) |
| [#655] cas 2 | `bank-import/profiles/[id]/+page.svelte` ne remet jamais `errorMessage` à `null` ni `isLoading` à `true` ; aucune garde de séquence | même fichier, l. 11 (déclaration), 16 et 28 (écritures), 44 (lecture) ; l'`$effect` `:13-31` |
| [#655] cas 2 (dépendance de l'effet) | L'effet lit `page.params.id` ; or SvelteKit **réassigne** `page.params` (objet neuf) à chaque navigation où `page_changed` est vrai — URL, erreur, formulaire ou données changés —, donc même quand l'`id` ne change pas (une *query string*, des données relues) ; sinon `props.page = clone_page(page)` garde la référence (validation P2, F5) | `node_modules/@sveltejs/kit/src/runtime/client/state.svelte.js:37` (`params = $state.raw({})`), `client.js:763-768` (`page_changed`), `:770-772` (`props.page` neuf, `params` compris), `:707` (`clone_page(page)` sinon), `:1888` (`Object.assign(page, navigation_result.props.page)`), `@sveltejs/kit` 2.55.0 ; `$derived` compare par `===` (`svelte/src/internal/client/reactivity/equality.js:4-6`, Svelte 5.55.1) ; `BankProfileForm.svelte:46-60` initialise son état **une fois** depuis `existing` (validation P1, F4) |
| [#646] | *Supprimer* une personne de contact appelle `deleteContactPerson` sans confirmation | `frontend/src/lib/features/contacts/ContactPersonsManager.svelte:76-83` (`removePerson`), bouton `:112-119` (`contact-person-delete-{id}`) ; le composant est rendu dans le dialogue d'édition (`routes/(app)/contacts/+page.svelte:1097-1100`, dans `Dialog.Root` `:811`) |
| [#646] (sémantique) | « Supprimer » **archive** : `active = FALSE`, trace `contact_person.archived` ; aucune route ne désarchive | `crates/kesh-db/src/repositories/contact_persons.rs:158` ; `crates/kesh-api/src/routes/contact_persons.rs:295` ; routes `crates/kesh-api/src/lib.rs:426-431`, `:794` (création, modification, suppression, liste) |
| [#646] (tests) | Le composant n'a **aucun** test Vitest ni E2E ; l'inventaire des boutons rouges écrit « sans confirmation (#646, #648) » | `git grep -ln "ContactPersonsManager\|contact-person-delete" -- frontend` → le composant, la page, `i18n-harvest.test.ts:35`, `boutons-rouges-inventory.test.ts:231-234` |
| [#646] (manuel) | Le manuel décrit l'ajout de personnes, pas leur suppression | `docs/manual/fr/user-manual.tex:869-874` (`keshnote`) |
| [#678] cas 1 *(amendement)* | La case de chaque ligne n'a **aucun** libellé accessible (ni `aria-label`, ni `<label>`, ni `aria-labelledby`) ; l'en-tête de sa colonne est vide | `fcd2ae75` : `ReconciliationProposals.svelte:319-327` (case dans `{#if p.candidates.length > 0}`), `<th class="text-left"></th>` de l'en-tête ; fiche 29-6 non mergée, worktree `kesh-29-6` à `d71c506b` : `:356-363` (case sur **toute** ligne) ; `grep -n "aria-label" ReconciliationProposals.svelte` → seul l'en-tête *Actions* (`:310` au sol, `:347` en 29-6) |
| [#678] cas 2 *(amendement)* | L'indication de la 29-6 est un `<p>` inséré par un `{#if}` : aucune région d'annonce n'existe avant qu'il apparaisse | `kesh-29-6` à `d71c506b` : `:316-323` (`{#if selectionHasUnmatched}` → `<p data-testid="reconciliation-unmatched-not-acceptable-hint">`), dernier enfant de la rangée des boutons `:294-324` ; `grep -n "aria-live\|role=\"status\"" ReconciliationProposals.svelte` → **rien**, au sol comme en 29-6 ; `selectionHasUnmatched` `:193-195` |
| [#678] (contrats éprouvés) *(amendement)* | Le `data-testid` de l'indication est éprouvé par présence **et** absence, et sa **position** par un test | Vitest (`kesh-29-6`, `ReconciliationProposals.test.ts:864-987`, `describe` de la 29-6) : `getByTestId(HINT_TESTID)` non nul / `queryByTestId(HINT_TESTID)` nul (`:898`, `:939`, `:943`, `:960`, `:962`) ; position `:965-986` (assertions `:977-978`) : `hint.parentElement` = parent de *Rejeter*, `lastElementChild` = l'indication (C-29-6-18) ; E2E `reconciliation-reject-unmatched.spec.ts:160-169` (`toBeVisible`, `toHaveCount(0)`) ; les cases sont éprouvées par `data-testid="tx-checkbox"` et `data-tx-id` (aucune par rôle et nom : `grep -rn "getByRole('checkbox'" frontend/src frontend/tests` → rien) |

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
  *Annotation du 2026-10-14 (validation P1, F4)* : la phrase « ne se rejoue que si `page.params.id` change » est
  **fausse au code** : l'effet dépend du signal `page.params`, que SvelteKit réassigne (objet neuf) à **toute**
  navigation dans la route, même à `id` égal (tableau ci-dessus). Aucun appelant ne le fait au sol (le défaut reste
  latent), mais un `isLoading = true` en tête d'un tel effet démonterait le formulaire et **perdrait une saisie en
  cours** à chaque navigation de ce genre. D'où AC6 réécrit : l'effet ne dépend plus que de l'identifiant, par un
  `$derived` (C-29-9-13).
  *Précision du 2026-10-14 (validation P2, F5)* : « à **toute** navigation » est trop large — la réassignation n'a lieu
  que si `page_changed` est vrai (`client.js:763-772`) ; une navigation à `id` égal qui change l'URL ou les données
  réassigne bien, ce qui suffit à la conclusion, qui ne change pas.
- **#646 — exacte.** Précision : la suppression est un archivage sans retour possible à l'écran — c'est ce qui rend la
  confirmation utile, et le texte de la confirmation le dit.
- **#678 — exacte** *(amendement du 2026-10-14)*, sur les deux cas. Précision : le second cas **n'existe qu'avec la
  29-6** (l'indication est son ajout, #665), non mergée à la date de l'amendement ; le premier existe au sol mais ne
  touche que les lignes avec proposition, la 29-6 l'étendant à toutes. L'AC11 s'écrit donc sur le composant **tel que
  la 29-6 le laisse**, et la 29-9 se rebase après elle (C-29-9-11, déjà).

## Critères d'acceptation

### #649 — la liste de la réconciliation dit qu'il en reste, et montre la suite

**AC1 — S'il en reste, l'écran le dit et offre la suite (C-29-9-2).**

Dans `ReconciliationProposals.svelte`, branche « liste non vide », **immédiatement après `</table>`** (`:401`),
**avant toute instance de modale** : celles du sol (`<ManualMatchModal` `:403`, `<TransactionSplitModal`) et la
`LinkSettlementModal` que la 29-8 ajoute à ce composant sans en prescrire l'emplacement (fiche 29-8 `:370`, T4) — son
emplacement naturel est le même, à côté des deux autres. Au rebase (T0), l'avis et le bouton se placent donc entre
`</table>` et la première modale, quelle qu'elle soit (validation P2, F8 ; C-29-9-11 annoté) :

- si `hasMore` est vrai **et** `limit < PROPOSALS_MAX_LIMIT` : un paragraphe `data-testid="reconciliation-more"`
  (`reconciliation-labels-more-hidden`, `{ $count }` = `proposals.length`) et un bouton
  `data-testid="reconciliation-show-older-btn"` (`reconciliation-actions-show-older`, `{ $max }` =
  `PROPOSALS_MAX_LIMIT`), `disabled={busy}` — pendant un lot (*Accepter*, *Rejeter*), la liste reste affichée
  (`busy` passe à `true` `:194`/`:239`, `loading` seulement au `load()` final), et un clic rechargerait la liste au
  milieu du lot ; V18 et M21 le prouvent (validation P2, R8) ;
- si `hasMore` est faux : rien (ni avis, ni bouton).

Style : celui des boutons secondaires voisins (`rounded border border-text px-4 py-2`, comme *Rejeter* `:290-298`),
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
- **Saut de défilement, assumé et déclaré (C-29-9-15).** Pendant le rechargement, `{#if loading}` (`:266-269`)
  remplace la table, l'avis et le bouton par le message de chargement — comme il le fait déjà après *Accepter* ou
  *Rejeter* ; le document se raccourcit, la page remonte, puis la liste revient à 500 lignes. `load()` **ne change
  pas** sur ce point : garder la liste affichée pendant un rechargement la laisserait cliquable pendant un lot en vol,
  pour tous les déclencheurs et pas seulement ce bouton. Le Dev Agent Record le déclare avec la capture.

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
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts:52-55` : le commentaire de `hasMore` (« v0.1 :
  structure présente mais pas de UI dédiée (bouton « Charger plus » v0.2) ») est réécrit comme celui du Rust — ce
  que fait l'écran depuis la 29-9 (avis, bouton jusqu'au plafond, avis plafonné) ; il ne parle plus de « Charger
  plus » (validation P1, R3/F3).
- **Commentaire de module `kesh-db` réécrit en T1** (décision de l'orchestrateur, validation P2, R1 = F1 ; C-29-9-16
  annoté) : `crates/kesh-db/src/repositories/reconciliation.rs:18-19` (« paginées (limit hardcodé 100 au caller, cf.
  L24) ») est **faux depuis la Story 8-4** (`22fd6d4a`, le commit même qui l'a écrit) — l'appelant ne code pas 100 en
  dur : la route prend `?limit=`, 100 par défaut, 500 au plus (`routes/reconciliation.rs:598-602`), et passe
  `limit + 1` au repository (`:622-626`) ; après la story, l'écran demande lui-même 500. Il devient : « paginées
  (`limit` fourni par la route : défaut 100, plafond 500, sentinelle `limit + 1` pour `hasMore` — cf.
  `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT`, `kesh-api/src/routes/reconciliation.rs`) ». Le renvoi `L24`
  (limitation « pas de UI » de la 8-4, que la 29-9 lève) disparaît. C'est un **commentaire** : aucun code de production
  Rust ne change, l'affirmation du point précédent tient.
  ⚠️ **Conséquence : `kesh-db` entre dans le diff**, et avec lui l'exception *kesh-db* de CLAUDE.md (« dès qu'un
  **patch** touche … un repository, le ciblage est interdit »). Elle vaut pour **le commit qui touche ce fichier** —
  celui de T1 — et pour lui seul : ce commit-là passe le gate complet ; les commits suivants, qui ne touchent pas
  `kesh-db`, reviennent au gate ciblé. Si une passe de revue retouche ce fichier, son patch reprend le gate complet
  (AC10). Un module de plus au sens de la règle de découpage (§ *Dérogation*).
- Tests neufs dans `crates/kesh-api/tests/reconciliation_e2e.rs`, à côté de `get_proposals_paginates_at_100_default`,
  même décor (`setup_company`, `make_new_tx`, `seed_bank_transactions`) :
  `get_proposals_limit_is_capped_at_500` — 501 transactions, `?limit=1000` → **500** rendues, `hasMore` **vrai** ; et
  `get_proposals_limit_500_exact_has_no_more` — **exactement 500** transactions, `?limit=500` → **500**, `hasMore`
  **faux** (le second état de l'écran, pris **à la frontière** : `>=` pour `>` y rougit, M18 ; validation P1, F8).
  ⚠️ La mutation `limit` pour `limit + 1` (M20) **ne** le rougit **pas** — 500 lignes lues sur 500, `500 > 500` est faux,
  `hasMore` faux comme attendu — : elle est tuée par le test **à 501** (`get_proposals_limit_is_capped_at_500` : 500
  lues, `hasMore` faux au lieu de vrai) et par `get_proposals_paginates_at_100_default` (validation P2, R2 = F3 ; la
  phrase de la P1 qui la lui attribuait était fausse).

### #655 — une relecture ancienne ne masque plus une page à jour

**AC5 — Détail d'un import : numéro de séquence (C-29-9-7).**

`routes/(app)/bank-import/[id]/+page.svelte`, `load()` (`:47-70`) : `let loadSeq = 0` ; `const mySeq = ++loadSeq` en
**première** instruction de `load()`, avant la remise à `null` et avant la branche « Identifiant invalide » ; après
l'`await`, `detail` et `errorMessage` ne s'écrivent que si `mySeq === loadSeq` (dans le `try` et le `catch`). La
remise à `null` de la 28-5b (`:51`) et son commentaire restent ; un commentaire dit le patron
(`ReconciliationProposals.svelte`, `loadGen`) et la raison (#655).

**`loading` n'est pas gardé (C-29-9-14).** `loading` ne passe à `true` qu'à la déclaration (`:43`) ; `load()` ne le
remet jamais à `true`, et une relecture ne peut partir qu'une fois la page rendue, donc après que la première lecture
l'a posé à `false` (les deux déclencheurs, `UnrejectTransactionAction` `:160` et le bouton qui ouvre
`CancelReconciliationDialog` `:150`/`:169-176`, ne sont rendus que dans la branche `{:else if detail}` `:96`, derrière
`{#if loading}` `:86`). Un `finally` périmé qui écrit `false` sur un `false` n'a aucun effet observable : une garde y serait
intuable (validation P1, F7 et R8). Le `finally` reste `loading = false`, avec une ligne de commentaire qui le dit —
et qui dit qu'il faudra le garder le jour où `load()` remettra `loading` à `true`.

**AC6 — Page d'un profil bancaire : remise à zéro et séquence (C-29-9-8).**

`routes/(app)/bank-import/profiles/[id]/+page.svelte`, `$effect` (`:13-31`). ⛔ **Contrainte première : la
correction ne démonte jamais le formulaire ni ne perd une saisie en cours au même identifiant** (C-29-9-13).

- **L'effet dépend de l'identifiant, et de lui seul** : `const rawId = $derived(page.params.id)` au niveau du script ;
  l'effet lit `rawId` (puis `Number(rawId)`), **jamais** `page.params`. Un `$derived` d'une **chaîne** compare par
  `===` (`equality.js:4-6`) : la réassignation de `page.params` à `id` égal le recalcule à la même valeur, et
  l'effet ne se rejoue pas. ⚠️ Dériver la chaîne, **pas** `Number(page.params.id)` : `NaN !== NaN`, si bien qu'un
  identifiant invalide relancerait l'effet à chaque réassignation de `page.params` (navigation où `page_changed` est
  vrai, `client.js:763-772`).
- **En tête de l'effet** — donc au début d'un chargement **réel**, puisqu'il ne se rejoue qu'à `id` changé :
  `const mySeq = ++loadSeq` en **première** instruction, **avant** la branche `!Number.isFinite(id)` (une réponse
  ancienne encore en vol ne doit pas écraser « ID invalide » ; validation P1, R7), puis `errorMessage = null` et
  `isLoading = true`.
- **`isLoading = true` est retenu, et c'est établi au code qu'il ne touche pas le formulaire au même id** : il n'est
  exécuté que dans l'effet, qui ne se rejoue qu'à `rawId` changé. À id changé, il est **nécessaire** :
  `BankProfileForm` initialise ses champs **une fois** depuis `existing` (`BankProfileForm.svelte:46-60`, `$state(existing?.…)`),
  si bien que sans démontage le profil 6 s'afficherait avec les champs du profil 5.
- **Garde de séquence** : dans le `.then` et le `.catch`, `profile`, `errorMessage` **et** `isLoading` ne s'écrivent
  que si `mySeq === loadSeq` — une réponse plus ancienne, succès comme échec, est ignorée en entier.
- ⚠️ Pas de boucle : l'effet lit `rawId` et le compteur (variable non réactive) ; il écrit `errorMessage`,
  `isLoading`, `profile`, qu'il ne lit pas.
- **Ce que l'effet exécute aussi : la partie synchrone de `getBankProfile(id)`** (validation P2, F6 ; C-29-9-18).
  « L'effet ne dépend que de l'identifiant » suppose qu'elle ne lit aucun signal. C'est vrai au sol : `request` →
  `buildHeaders` → `fetchWithRetry` → `fetchWithTimeout` ne lisent aucun `$state` avant le premier `await`
  (`bank-profile.api.ts:19-21`, `api-client.ts:181-199`, `:267-280`, `:329-336`, `:392-399`). Mais V13 mocke l'API et
  ne verrait pas une lecture synchrone d'`authState` qui y entrerait un jour (l'effet se relancerait au renouvellement
  du jeton, et `isLoading = true` perdrait la saisie). **Retenu : un commentaire au site** qui écrit cette dépendance
  et dit le remède (`untrack(() => getBankProfile(id))`) ; **pas** d'`untrack` dans le code — garde qu'aucun test ne
  peut rougir, même règle que C-29-9-14.
- **`onUpdated` reçoit la même garde** (décision de l'orchestrateur, validation P2, F2 ; C-29-9-17). Au sol
  (`:33-35`), `onUpdated(updated)` écrit `profile` sans condition. Scénario, possible dès qu'un lien mènera d'un profil
  à un autre : sur 5, *Mettre à jour* part (`BankProfileForm.svelte:113-114`, `existing.id` = 5 lu **avant** l'`await`) ;
  navigation vers 6, le formulaire de 5 est démonté mais `handleSubmit` continue ; 6 se charge, un formulaire neuf
  s'initialise sur p6 ; la sauvegarde de 5 se termine, `onSuccess(p5)` → `profile = p5` ; le formulaire monté reçoit
  `existing` = p5 mais garde les champs de 6 (initialisés une fois, `:46-60`) ; la sauvegarde suivante écrit les valeurs
  de 6 **dans le profil 5**. Correctif : une sauvegarde qui se termine après une navigation vers un autre profil
  n'écrit plus `profile`.
  ⚠️ **Forme de la garde, établie au code** : un numéro de séquence capturé dans une fermeture passée au formulaire
  (`onSuccess={onUpdatedFor(seq)}`) **ne tient pas**, parce que le formulaire lit ses props **en direct** : Svelte
  5.55.1 compile `let { existing, onSuccess } = $props()` suivi d'un appel après `await` en
  `$$props.onSuccess?.($$props.existing)` (compilation d'un composant d'essai, 2026-10-14, hors dépôt) — le formulaire
  démonté de 5 appellerait le gestionnaire **courant**, celui de 6, et la garde passerait. La garde se pose donc
  **dans** `onUpdated`, évaluée à l'appel, sur l'identité du chargement courant :
  `if (updated.id !== Number(rawId)) return;` puis `profile = updated`. C'est l'équivalent, pour une écriture que la
  page ne lance pas elle-même, du `mySeq === loadSeq` du chargement : est ignorée toute écriture qui ne relève plus du
  profil affiché. Retour de 6 à 5 avant la fin de la sauvegarde de 5 : la réponse est acceptée, et c'est juste (même
  profil, valeurs enregistrées). Si V17 montrait une autre lecture des props, le dire au Dev Agent Record avant de
  changer de forme.
- Commentaire au site : défaut latent au sol (aucun lien d'un profil à l'autre), corrigé pour le jour où il y en aura
  un ; pourquoi `$derived` de la chaîne (réassignation de `page.params`, `NaN`) ; pourquoi `isLoading = true` est
  sans effet à id égal et nécessaire à id changé ; la dépendance synchrone de `getBankProfile` (C-29-9-18) ; pourquoi
  la garde d'`onUpdated` porte sur l'identifiant et non sur une fermeture (C-29-9-17).

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
  `:1806-1815`) ; le fallback du `.svelte` = la valeur `fr-CH`, au caractère près (l'apostrophe droite de
  `jusqu'à`, celle du `.ftl`, impose un repli entre guillemets doubles ou échappé ; jamais un gabarit JS) :

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
  deux bornes et la commande, sur le modèle des lignes `:620-633`. *Annotation du 2026-10-14 (amendement #678)* : le
  recompte se fait **après** T10, qui ajoute les sites des deux clés de l'AC11.
- **`frontend/scripts/lint-i18n-ownership.js`** : la clé `contact-persons-confirm-delete` entre dans
  `KNOWN_VIOLATIONS` (ensemble `:22-140` ; bloc `ContactPersonsManager.svelte` `:117-139`, commentaire `:117-124`,
  entrées `:125-139`), **après la ligne `:139`**, à la suite des autres clés de ce composant (validation P2, R5 = F9) :
  `'src/lib/features/contacts/ContactPersonsManager.svelte:contact-persons-confirm-delete',` — le lint compare le
  préfixe `contact` au dossier `contacts` (dette #30), et le rougirait sinon. La dette #30 s'accroît d'**une** entrée,
  écrite comme telle au Dev Agent Record (validation P1, R2/F2). Les trois clés `reconciliation-*` sont dans le
  dossier `reconciliation` : aucune entrée. `npm run lint-i18n-ownership` vert.
- **Gardes i18n à relancer et qui portent sur ces clés** (validation P1, R6) :
  `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` (variables `{ $count }`, `{ $max }`, `{ $name }`
  identiques entre site, repli et **quatre** locales ; repli littéral) ; `i18n-repli-divergent-actif.test.ts` (relève
  les sites, mais ne **vérifie** « repli = valeur `fr-CH` » que pour une clé à deux replis divergents ou inscrite à
  `REPLIS_A_SITE_UNIQUE` : les clés neuves de cette story, à site unique et non inscrites, **ne sont pas couvertes** ;
  pour `reconciliation-labels-more-hidden` et `-more-capped`, l'égalité repli/catalogue n'est gardée par aucun test —
  angle mort assumé ; pour les deux clés de l'AC11, V19/V20 lisent leur attendu au catalogue `fr-CH`) ; `i18n-keys.test.ts` ; `i18n-harvest.test.ts`. Toutes au
  `test:unit` ; leur vert se cite au Dev Agent Record. Hors de portée, relevé au sol : `i18n-un-repli-par-cle.test.ts`
  ne vise que le domaine `supplier-invoices` (en-tête du fichier) ; `crates/kesh-api/tests/textes_coherents.rs`,
  parcouru par la lentille R, n'a aucune garde sur ces phrases.
- **Manuel** `docs/manual/fr/user-manual.tex` :
  - `:2045` — « les 100 plus récentes au plus » devient : les 100 plus récentes d'abord ; s'il en reste, un avis le dit
    sous la liste et le bouton *Afficher aussi les plus anciennes* l'étend à 500 ; au-delà, les plus anciennes
    apparaissent à mesure que les premières sont acceptées, affectées ou éclatées ; le bouton vide la sélection ;
  - `:2057` — **en delta sur la phrase rebasée** : le paragraphe *Rejeter* est réécrit par la 29-6 (et relu par la
    29-8) ; la 29-9 ne le remplace pas. Dans la phrase telle que `main` la porte au rebase, elle ne touche **que** la
    parenthèse « qui ne montre que les 100 plus récentes en attente~: une transaction plus ancienne n'y reparaît
    qu'une fois les plus récentes traitées » : le chiffre en sort, et la parenthèse renvoie à l'avis et au bouton
    décrits plus haut (`:2045`). Tout le reste de la phrase — ce que la 29-6 et la 29-8 y ont écrit — reste à
    l'octet ; le diff de ce paragraphe ne porte que sur la parenthèse (validation P1, R4) ;
  - `:869-874` — la `keshnote` des personnes de contact ajoute : *Supprimer* demande confirmation ; la personne quitte
    la fiche, et aucun écran ne la rétablit.
  - PDF régénéré (`make fr` dans `docs/manual/`, **sous le verrou**), contrôlé aplati (§ *Propagation*).
- **`docs/api-external.md:594`** : « (100 par défaut) » → « (100 par défaut, 500 au plus ; `hasMore` vaut `true` s'il
  en reste) » ; le lien #649 est retiré (l'issue sera close) ; le reste de la phrase, vrai pour l'API, ne change pas.
  **En delta** sur le texte rebasé : la 29-8 ajoute ses deux routes en paragraphes à côté de `…/unreject` (`:594`),
  qui restent à l'octet (zones partagées ; validation P2, R6).
- **CHANGELOG** : dans la section de la v0.13.4 que porte `main` au moment du rebase (la 29-2 la crée — sinon
  `## [Non publié]`), sous `### Corrigé`, trois puces : la suite de la liste de réconciliation (#649), la bannière
  périmée du détail d'un import (#655), la confirmation de suppression d'une personne de contact (#646). La section
  `[0.13.3]` ne se touche pas. *Annotation du 2026-10-14 (amendement #678)* : **quatre** puces, la quatrième pour #678
  (AC11).
- **`boutons-rouges-inventory.test.ts:233`** : motif → `'D — suppression d\'une personne de contact, confirmée (#646) ;
  texte rouge sans bordure (#648)'` (le motif dirait faux sinon ; l'entrée et ses jetons restent).

**AC9 — Tests Vitest rouges d'abord, E2E, captures.**

Vitest (écrits **avant** le correctif et vus rouges, sortie copiée au Dev Agent Record) :

- `ReconciliationProposals.test.ts` (patrons du fichier : `mockApi.getProposals`, `satisfies GetProposalsResponse`) :
  - ⛔ **Étape 0, avant les tests neufs (V1-V5, V12, V18) — le mock du module API** (validation P1, R1/F1). La fabrique
    `vi.mock('./reconciliation.api', () => ({ getProposals: vi.fn(), … }))` (`:22-27`) n'exporte pas les constantes
    neuves : dès que le composant importe `PROPOSALS_DEFAULT_LIMIT`/`PROPOSALS_MAX_LIMIT`, Vitest lève à l'accès
    (« No "…" export is defined on the mock ») et **tout** le fichier rougit, sans rapport avec son objet. **Geste en
    delta, sur la fabrique telle que `main` la porte au rebase** (validation P2, R6) : elle passe à
    `vi.mock('./reconciliation.api', async (importOriginal) => ({ ...(await importOriginal<typeof
    import('./reconciliation.api')>()), /* les vi.fn() de la fabrique rebasée, tous gardés */ }))` — chaque entrée que
    la fabrique porte au rebase reste un `vi.fn()` (au sol, quatre : `getProposals`, `acceptProposals`,
    `rejectProposals`, `manualMatchTransaction` ; la 29-8 en ajoute, `getLinkCandidates`/`linkTransaction`, fiche 29-8
    `:387`, `:550`), les constantes sont les vraies. Ne **jamais** recopier une fabrique complète de cette fiche : ce
    serait effacer les entrées d'une fiche voisine.
    Ce changement se fait **en premier**, avec les constantes de l'AC4 (T1), et le fichier existant se relance **vert**
    avant d'écrire les tests neufs : ainsi V2-V5 rougissent pour la **bonne** raison (le comportement absent), et la
    sortie copiée au Dev Agent Record le montre ; V1 et V12, qui exigent une **absence**, sont verts au sol (T2 ;
    validation P2, F7). Seul ce fichier mocke `./reconciliation.api` pour un composant qui importera les constantes
    (`git grep -ln "reconciliation.api'" -- frontend/src`, relu par les deux lentilles).
    **Dépendance neuve, à savoir d'avance** (validation P2, R4) : au sol, ce fichier de test ne charge **pas**
    `api-client` — les deux modales sont des doublures (`:44-49`), `error-message.ts` n'importe que `./api-error`, les
    API des comptes sont mockées. `importOriginal` de `./reconciliation.api` fait entrer `api-client.ts` et ses imports :
    `$lib/app/stores/auth.svelte` (qui crée un `BroadcastChannel` dès son chargement, `auth.svelte.ts:34-36`, toléré
    absent sous jsdom), `api-health.svelte`, et `i18n.svelte` (mocké dans ce fichier, `:53`). Risque faible —
    `bank-import-page.test.ts:19-20` charge déjà le vrai `api-client` par `importOriginal` —, mais un échec au
    chargement du fichier se lit d'abord là.
    Si `importOriginal` se révèle impossible (import réel qui casse le montage), repli : loger les deux constantes dans
    un module sans I/O `reconciliation.limits.ts`, non mocké, et **`ReconciliationProposals.svelte` importe les
    constantes directement depuis `./reconciliation.limits`** — une réexportation par `reconciliation.api.ts` reste
    possible pour les autres lecteurs, mais le composant qui passerait par elle traverserait le module mocké, et l'échec
    « No "…" export » reviendrait (validation P2, R4). Repli à déclarer au Dev Agent Record.
  - V1 `hasMore: false` → ni `reconciliation-more`, ni bouton, ni `reconciliation-more-capped` ;
  - V2 `hasMore: true` (100 lignes) → avis avec « 100 », bouton présent ; `getProposals` appelé avec `(id, 100)` ;
  - V3 clic → liste de la seconde réponse rendue, **puis** `await tick()`, **puis seulement** l'assertion :
    `getProposals` appelé **exactement deux fois** en tout, la seconde avec `(id, 500)` (une troisième requête — M3 —
    part au tick suivant : compter avant elle la laisserait survivre ; validation P1, F10) ;
  - V4 seconde réponse `hasMore: true` → `reconciliation-more-capped` présent, **aucun** bouton ; le texte ne
    contient pas « rejet » ;
  - V5 après le clic, un *Accepter* réussi recharge avec `(id, 500)` (pas 100) ;
  - V12 `hasMore` **retombe** : premier chargement `hasMore: true`, *Accepter* réussi, relecture `hasMore: false` → ni
    `reconciliation-more`, ni bouton, ni `reconciliation-more-capped` (validation P1, F6) ;
  - V18 **bouton désactivé pendant un lot** : premier chargement `hasMore: true`, une ligne cochée, `acceptProposals`
    **différé** (promesse tenue par le test), clic *Accepter* → `reconciliation-show-older-btn` porte `disabled` et
    `getProposals` n'a été appelé qu'une fois ; puis la promesse se résout (validation P2, R8) ;
  - aucun test existant ne vérifie les arguments de `getProposals` (`grep -n "toHaveBeenCalledWith"
    ReconciliationProposals.test.ts` au sol : aucun sur `getProposals`) ; `reconciliation.api.test.ts` vérifie l'URL
    (`limit=100` par défaut) et reste vert.
- `bank-import-detail-page.test.ts` :
  - V6 deux relectures (deux *Annuler le rejet*, patron du test `:171-217`) ; la **plus récente** réussit d'abord,
    la plus ancienne **échoue ensuite** → aucune bannière, lignes de la plus récente ;
  - V7 même décor, la plus ancienne **réussit** ensuite avec des données périmées → les lignes restent celles de la
    plus récente ;
  - le test existant `:171` reste vert (ordre direct) ;
  - pas de test sur `loading` : non gardé, et sans effet observable (C-29-9-14).
- `profiles/[id]/profile-page.test.ts` : un `page` **réactif qui imite SvelteKit** (validation P1, F4) — fixture
  `profile-page-state.svelte.ts` à côté du test : `class FakePage { params = $state.raw<Record<string, string>>({ id:
  '5' }); } export const fakePage = new FakePage();`, servie par `vi.mock('$app/state', async () => ({ page: (await
  import('./profile-page-state.svelte')).fakePage }))`. Une navigation **réassigne** l'objet entier
  (`fakePage.params = { id: '6' }`), comme `client.js:1888` ; jamais `fakePage.params.id = …`. `beforeEach` remet
  `fakePage.params = { id: '5' }` (module partagé : sans cela l'ordre des tests décide ; F10). `await tick()` après
  chaque réassignation.
  - V8 id 5 en échec → message ; id 6 en succès → message disparu, formulaire rendu ;
  - V9 id 5 lent puis id 6 rapide ; 6 réussit, 5 échoue ensuite → aucun message, profil 6 ;
  - V13 **navigation au même id** : id 5 en succès, une saisie dans le champ `bank-name-input` (`BankProfileForm.svelte:135-137`),
    puis `fakePage.params = { id: '5' }` (objet neuf) et `await tick()` → `getBankProfile` appelé **une seule fois**,
    jamais « Chargement… », la saisie **intacte** ;
  - V14 **montage neuf à id changé** : id 5 en succès (`bankName` « Banque 5 »), puis id 6 (« Banque 6 ») → pendant
    la lecture de 6, « Chargement… » et aucun formulaire ; ensuite le champ montre « Banque 6 » (validation P1, F5) ;
  - V15 **5 se termine pendant que 6 est en vol** : id 5 lent, id 6 plus lent ; 5 **réussit** pendant que 6 court →
    « Chargement… » toujours affiché, aucun champ « Banque 5 » ; puis 6 réussit → « Banque 6 » (validation P1, R8) ;
  - V16 **identifiant invalide après une lecture lente** : id 5 lent, puis `{ id: 'abc' }` → « ID invalide » ; 5
    échoue ensuite → « ID invalide » reste (validation P1, R7) ;
  - V17 **sauvegarde tardive après navigation** (validation P2, F2) : la fabrique de
    `$lib/features/bank-import/bank-profile.api` gagne `updateBankProfile: (id, req) => updateMock(id, req)` (et
    `createBankProfile: vi.fn()`, jamais appelé, pour que le module mocké reste complet — le formulaire importe les deux,
    `BankProfileForm.svelte:11-14`) ; id 5 en succès (« Banque 5 », fabrique de profil **complète** : le formulaire lit
    `existing.columnMapping`, `:34-62`), `updateMock` **différé**, envoi du formulaire (`fireEvent.submit` sur
    `bank-import-profile-form`) ; `fakePage.params = { id: '6' }`, 6 en succès (« Banque 6 ») ; **puis** la sauvegarde
    de 5 se résout avec p5 → le champ montre « Banque 6 », et un second envoi appelle `updateMock` avec **6** en premier
    argument (sous M19 : 5) ;
  - les deux tests existants restent verts. Si la fixture réactive se révèle impossible sous Vitest, le dire au Dev
    Agent Record avec l'erreur, et **ne pas** déclarer AC6 prouvé.
- `ContactPersonsManager.test.ts` (neuf ; `vi.stubGlobal('confirm', …)`, patron `BankProfileList.test.ts:57-58`) :
  - V10 refus → `deleteContactPerson` **non** appelé, la personne reste listée ; `confirm` reçu avec le nom ;
  - V11 acceptation → `deleteContactPerson(id)` appelé, liste relue.

E2E (Playwright, montage `docs/testing.md` § *Prérequis Playwright local*, port **3049**, base `kesh_e2e_299`) :

- `frontend/tests/e2e/reconciliation-show-older.spec.ts` (neuf, patron `reconciliation-unreject.spec.ts:79-121`) :
  import par l'API d'un CAMT **synthétique** de **501** transactions (le bloc `<Ntry>` de
  `camt053_v04_minimal.xml` répété), `confirmBalanceMismatch` (la somme de 501 lignes ne retombe pas sur `CLBD`) ;
  page *Réconciliation*, compte sélectionné → 100 lignes, avis et bouton ; clic → 500 lignes, avis plafonné, aucun
  bouton ; la 150e de nos transactions (par ordre décroissant) absente avant, présente après ; import supprimé en
  `finally`. Captures clair et sombre des deux états (`capturePath` de `reconciliation-unreject.spec.ts:125`).
  Contraintes de la fixture, établies par la lentille F de la validation P1 (F11) :
  - **chaque entrée unique** — montant, `AcctSvcrRef` et `EndToEndId` distincts par entrée (et par run) : le
    dédoublonnage par défaut de l'import est un *skip* silencieux (`ConfirmDuplicateLines::Skip`) sur (date, montant,
    référence normalisée, compte) ; une entrée en double disparaîtrait sans erreur et le compte de 501 serait faux. La
    spec vérifie, à la réponse de l'import, que **501** lignes sont créées ;
  - `FrToDt` cohérent avec les dates des entrées ;
  - **pas de contrainte de date** vis-à-vis des autres fixtures : la contrainte « dates antérieures à celles de
    toutes les autres fixtures E2E » de la rédaction initiale est **retirée** — `seedTestState('with-company')` en
    `beforeAll` remet la base à zéro et `workers: 1` (`playwright.config.ts:36-53`) : la spec ne voit que ses propres
    transactions, et l'ancienne hypothèse « moins de 350 autres en attente » devient sans objet.
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
| M6 | `MAX_PROPOSALS_LIMIT = 499` (Rust) | `get_proposals_limit_is_capped_at_500`, `get_proposals_limit_500_exact_has_no_more` |
| M7 | garde `mySeq !== loadSeq` retirée du `catch` du détail | V6 |
| M8 | garde retirée du `try` du détail | V7 |
| M9 | `errorMessage = null` retiré de l'effet du profil | V8 |
| M10 | garde de séquence retirée du `.catch` du profil | V9 |
| M11 | `if (!confirm(…)) return;` → `confirm(…);` | V10 |
| M12 | `return` placé **avant** l'appel quelle que soit la réponse | V11 |
| M13 | `isLoading = true` retiré de l'effet du profil | V14 |
| M14 | `hasMore = r.hasMore` → `if (r.hasMore) hasMore = true` | V12 |
| M15 | l'effet du profil relit `page.params.id` au lieu de `rawId` (`$derived` contourné) | V13 |
| M16 | profil : `isLoading = false` écrit **hors** de la garde de séquence, dans le `.then` et le `.catch` | V15 |
| M17 | profil : `const mySeq = ++loadSeq` placé **après** la branche `!Number.isFinite(id)` | V16 |
| M18 | `has_more = transactions.len() as i64 >= limit` (Rust, `routes/reconciliation.rs:628`) | `get_proposals_limit_500_exact_has_no_more` |
| M19 | profil : garde `updated.id !== Number(rawId)` retirée d'`onUpdated` | V17 |
| M20 | `limit + 1` → `limit` dans l'appel du repository (Rust, `routes/reconciliation.rs:625`) | `get_proposals_limit_is_capped_at_500`, `get_proposals_paginates_at_100_default` |
| M21 | `disabled={busy}` retiré du bouton `reconciliation-show-older-btn` | V18 |

M15 n'est tuée que parce que la fixture **réassigne** `params` (comme SvelteKit) : avec un `$state` profond dont seul
`id` bougerait, l'effet ne se rejouerait pas à id égal, mutation ou non — c'est ce que F4 a relevé. M18 et M6
sont des mutations temporaires de production Rust, restaurées par `git checkout -- f && touch f` comme les autres ; M20
aussi.

Gates : gate ciblé entre les passes ; **exception *kesh-db*** : le commit de T1, qui réécrit le commentaire de
`crates/kesh-db/src/repositories/reconciliation.rs:18-19`, passe le **gate complet** (`scripts/test-fast.sh` sur
`kesh_299` remise à zéro) — à ce commit-là seulement, et à tout patch de revue qui retoucherait ce fichier ; les autres
commits restent au gate ciblé (validation P2, R1 = F1). **Au dernier commit de code** : `scripts/test-fast.sh` complet sur base
`kesh_299` remise à zéro (migrations du worktree, seed), frontend complet (`check`, `lint-i18n-ownership`,
`test:unit`, `build`), E2E complet — sous le verrou `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Ne déclarer
que ce qui a tourné, avec ses chiffres.

*Annotation du 2026-10-14 (amendement #678)* : les mutations M22-M30 et les tests V19-V23 sont ceux de l'AC11 ; ils
entrent dans le gate ciblé de T10 et dans les gates complets du dernier commit de code, sans rien changer à ce qui
précède.

### #678 — chaque case nomme sa transaction, et l'indication est annoncée *(amendement du 2026-10-14)*

**AC11 — Libellé accessible des cases, région d'annonce présente dès le rendu (C-29-9-20, C-29-9-21, C-29-9-22).**

Écrit **sur le composant tel que la 29-6 le laisse** (worktree `kesh-29-6` à `d71c506b` ; numéros relus en T0 après
le rebase, comme les autres). Rien de la 29-6 n'est réécrit : ni la case, ni `toggle`, ni `selectionHasUnmatched`, ni
le `<p>` de l'indication, ni sa clé, ni son repli.

*Le libellé de la case.*

- Chaque `<input type="checkbox" data-testid="tx-checkbox">` (`:357-363` en 29-6) reçoit un `aria-label` qui nomme
  sa transaction par les trois valeurs que la ligne **affiche** (`:365-367`) — date, montant et devise, contrepartie —,
  telles quelles (chaînes de l'API : `bookingDate`, `amount`, `currency`, `counterpartyName`), sans reformatage, si
  bien que le libellé dit ce que l'œil lit sur la ligne.
- **Deux clés à variables** (C-29-9-20), choisies par `p.transaction.counterpartyName ?` (une contrepartie vide ou
  nulle n'est pas nommée — la cellule n'affiche rien non plus, `?? ''` ; la règle est la **véracité** de la valeur,
  pas `!= null`, de sorte que `''` et `null` donnent la même clé) :

  | Clé | fr-CH |
  |---|---|
  | `reconciliation-labels-select-transaction` | `Sélectionner la transaction du { $date }, { $amount } { $currency }, { $counterparty }` |
  | `reconciliation-labels-select-transaction-no-counterparty` | `Sélectionner la transaction du { $date }, { $amount } { $currency }` |

  de-CH, en-CH, it-CH : traductions fidèles, même registre que `reconciliation-labels-*` voisines (`fr-CH:1262-1316`
  en 29-6) ; les variables sont **les mêmes** dans les quatre locales, le site et le repli (garde
  `i18n-entrees-a-variables.test.ts`) ; repli = valeur `fr-CH` au caractère près : **`i18n-repli-divergent-actif.test.ts`
  ne le garde pas ici** (`:146`, `parTexte.size > 1` : une clé à site unique est ignorée, et ces deux clés ne sont pas
  à `REPLIS_A_SITE_UNIQUE`, `:216-233`). C'est donc V19 et V20 qui le tiennent : leur attendu est **lu dans
  `crates/kesh-i18n/locales/fr-CH/messages.ftl`** (précédents : `UnrejectTransactionAction.test.ts:31`,
  `ReconciliationProposals.test.ts:672`), jamais écrit en dur — le mock i18n rend le repli, donc un repli divergent
  rougirait V19/V20 (C-29-9-25).
  Les deux clés sont du dossier `reconciliation` : aucune entrée à `KNOWN_VIOLATIONS`.
- **Forme du site (C-29-9-24).** **Deux appels `i18nMsg` à clé littérale**, chacun dans une branche d'un ternaire sur
  `p.transaction.counterpartyName` : `counterpartyName ? i18nMsg('reconciliation-labels-select-transaction', '…', { date,
  amount, currency, counterparty }) : i18nMsg('reconciliation-labels-select-transaction-no-counterparty', '…', { date,
  amount, currency })` — écriture **schématique** : dans le `{#each proposals as p}` il n'existe pas de variable
  locale `date`, etc. ; les arguments s'écrivent `{ date: p.transaction.bookingDate, amount: p.transaction.amount,
  currency: p.transaction.currency, counterparty: p.transaction.counterpartyName }` (le second sans `counterparty`).
  **Jamais** une clé calculée (`i18nMsg(cond ? 'a' : 'b', …)`) : établi au code des gardes
  (`i18n-entrees-a-variables.test.ts:125` et `i18n-repli-divergent-actif.test.ts:122` :
  `if (site.arg?.kind !== 'literal') continue;`, lecteur `i18n-literal-reader.js`), les gardes qui lisent
  les sites (`i18n-entrees-a-variables.test.ts`, maillons repli ↔ catalogue et site ↔ repli, et
  `i18n-repli-divergent-actif.test.ts`) ne retiennent **que** les appels dont la clé est littérale ; une clé
  conditionnelle leur est invisible. Deux autres ne sont pas dans ce cas : le maillon « entre locales » de
  `i18n-entrees-a-variables.test.ts` (`:176-188`) lit le catalogue, non les sites ; et `i18n-keys.test.ts` voit la clé
  calculée (`site.arg === null`, rangée à `sitesNonResolus`, `:899`, compté exactement) — **M30 le rougit aussi** :
  `sitesNonResolus` passe de 31 à 32, et le recompte de T5 ne doit pas l'absorber. **Contrôle** : un test neuf dans
  `i18n-entrees-a-variables.test.ts`, à côté de la borne des cinq clés de la 23-5 (qu'on ne modifie pas), « les deux
  entrées à variables de la story 29-9 sont bien relevées » : mêmes `sitesAVariables()`, liste nommée
  `reconciliation-labels-select-transaction` et `reconciliation-labels-select-transaction-no-counterparty` ; il est
  rouge au sol (clés absentes) et rougit si le site passe en clé calculée (M30). C'est le test **V23**. Tâche T10.
- Le `data-testid`, le `data-tx-id`, `checked` et `onchange` ne changent pas : les sélecteurs des tests et des E2E
  existants (`[data-testid="tx-checkbox"][data-tx-id=…]`) restent valides.
- L'en-tête vide de la colonne (`<th class="text-left"></th>`) **ne change pas** : chaque case porte son propre nom,
  ce que l'issue demande ; un nom de colonne n'en dirait pas plus (C-29-9-20).

*La région d'annonce.*

- Dans la rangée des boutons, à la place exacte du bloc `{#if selectionHasUnmatched}…{/if}` (`:316-323` en 29-6), et
  donc toujours **dernier enfant** de la rangée (C-29-6-18 tenu) :

  ```svelte
  <div role="status" aria-live="polite" aria-atomic="true" data-testid="reconciliation-selection-status">
  	{#if selectionHasUnmatched}
  		<!-- le <p data-testid="reconciliation-unmatched-not-acceptable-hint"> de la 29-6, à l'octet -->
  	{/if}
  </div>
  ```

  La région est rendue **dès le rendu de la liste** (branche « liste non vide », la seule où l'indication a un objet),
  vide tant qu'aucune ligne sans proposition n'est cochée ; son **contenu** suit `selectionHasUnmatched`, le **nœud**
  reste le même tant qu'une même liste est affichée, d'une sélection à l'autre — c'est ce qui rend l'annonce fiable
  (C-29-9-21). Un `load()` (acceptation, rejet, *Afficher aussi…*) pose `loading = true`, remplace toute la liste, donc
  la région, et vide `selected` (C-29-9-5) : la région est alors recréée, vide, ce qui est voulu. Commentaire au site : #678, pourquoi la région
  englobe le `{#if}` et non l'inverse, et qu'elle ne doit pas entrer dans un `{#key}` ni dans le `{#if}`.
- **Contrat du `data-testid` de la 29-6 : intact.** `reconciliation-unmatched-not-acceptable-hint` reste sur le `<p>`,
  présent si et seulement si `selectionHasUnmatched` : les assertions de présence et d'absence de Vitest
  (`ReconciliationProposals.test.ts:898`, `:939`, `:943`, `:960`, `:962` en 29-6) et de l'E2E
  (`reconciliation-reject-unmatched.spec.ts:160-169` : `toBeVisible`, `toHaveCount(0)`) passent **sans modification**.
- **Un seul test de la 29-6 change, en delta** (C-29-9-21) : le test de position (`:965-986` en 29-6, « l'indication
  est dans la rangée des boutons ») affirme `hint.parentElement` = la rangée et `lastElementChild` = l'indication ;
  l'indication est désormais **dans** la région. Ses deux lignes d'assertion de position (`:977-978`) deviennent :
  `hint.parentElement` = `getByTestId('reconciliation-selection-status')`, le parent de la région = la rangée de
  *Rejeter*, et le dernier enfant de la rangée = la région. Son intention (C-29-6-18 : dans la rangée, après *Rejeter*)
  et son volet « repli = catalogue fr-CH » restent à l'octet ; le commentaire `// C-29-6-18 …` gagne une ligne qui
  nomme #678. C'est le test V22.
- **Rendu** : celui de la 29-6, à une nuance près. Le `div` est un élément de la rangée `flex-wrap gap-2` ; vide, il
  n'a ni largeur ni hauteur, **mais il reçoit l'espacement `gap-2`** après *Rejeter* (un élément flex vide reste un
  élément flex) et peut, à largeur étroite, ajouter une ligne vide à la rangée repliée ; plein, il porte le `<p>` comme la rangée le portait. Vérifié par les captures clair et sombre que
  produit déjà `reconciliation-reject-unmatched.spec.ts` (relancée, non modifiée), comparées à celles de la 29-6 ; écart
  éventuel déclaré au Dev Agent Record.

*Textes.*

- Deux clés neuves ×4 (ci-dessus) ; `sitesTotal` de `i18n-keys.test.ts` recompté à la source en T5 (AC8), après T10.
- **CHANGELOG** : une quatrième puce sous `### Corrigé` de la section de la v0.13.4 (AC8) — sur l'écran de
  réconciliation, chaque case nomme sa transaction pour un lecteur d'écran, et l'indication « ne peut pas être
  acceptée » est annoncée (#678).
- **Manuel : inchangé** (C-29-9-22). Aucun texte **visible** ne change ; la phrase que la 29-6 écrit sur la case et
  l'indication (`user-manual.tex:2086` à `d71c506b` : « porte une case, comme les autres~; \emph{Accepter} ne la traite
  pas --- tant qu'une telle ligne est cochée, il reste grisé et la page le dit ») reste vraie. À vérifier dans le PDF
  régénéré par T6 (contrôle aplati, `grep -o "la page le dit.\{0,80\}"`) qu'elle est toujours présente.

*Tests Vitest* (`ReconciliationProposals.test.ts`, nouveau `describe` après celui de la 29-6 ; écrits **avant** le
correctif et vus rouges, sortie copiée au Dev Agent Record ; le mock i18n du fichier interpole les variables, `:50-55`) :

- V19 deux lignes, l'une avec proposition, l'autre sans (fabriques du fichier) → `getByRole('checkbox', { name })`
  trouve chacune, `name` = la valeur fr-CH **lue dans le catalogue** (C-29-9-25) de `reconciliation-labels-select-transaction` remplie des valeurs de la
  fabrique (sans proposition : `2026-05-15`, `50.00`, `CHF`, `UNKNOWN`). Rouge au sol : aucune case n'a de nom ;
- V20 deux cas, `counterpartyName: null` **et** `counterpartyName: ''` → le nom est exactement la valeur fr-CH de
  `reconciliation-labels-select-transaction-no-counterparty` remplie (aucune virgule finale) ;
- V21 **la région précède le changement** : au premier rendu de la liste, `getByTestId('reconciliation-selection-status')`
  existe, porte `role="status"`, `aria-live="polite"` et `aria-atomic="true"`, son texte est vide, l'indication absente ; référence gardée.
  Cocher la ligne sans proposition → l'indication est **dans** la région (`region.contains(hint)`), et
  `getByTestId('reconciliation-selection-status')` est **le même nœud** que la référence. Décocher → la région est
  toujours là (même nœud), vide. Rouge au sol : pas de région ;
- V22 le test de position de la 29-6, amendé en delta (ci-dessus) ; rouge avant le correctif sur sa nouvelle forme
  (pas de région), vert après.

*E2E* — dans `reconciliation-show-older.spec.ts` (neuf, AC9 ; ses 501 transactions n'ont aucune facture, donc aucune
proposition), **avant** le clic sur *Afficher aussi les plus anciennes* : pour la première ligne, le nom accessible de
sa case (`getByRole('checkbox', { name })`) égale la valeur fr-CH remplie des **textes de ses cellules** date, montant,
contrepartie ; `reconciliation-selection-status` présent, `aria-live="polite"`, vide ; cocher cette ligne →
l'indication visible **dans** la région ; décocher → indication absente, région présente. Puis la spec continue
(AC9). Si une ligne de la spec avait une proposition, viser une ligne sans proposition par `data-tx-id`. **Aucune
capture neuve** : rien de visible ne change (C-29-9-22) ; celles de la 29-6 servent de témoin (ci-dessus).

*Mutations* (mêmes règles que l'AC10 ; motif vérifié unique, `git checkout -- f && touch f`) :

| # | Mutation | Doit tuer |
|---|---|---|
| M22 | `aria-label` retiré de la case | V19 |
| M23 | toujours la clé avec contrepartie (branche `no-counterparty` retirée) | V20 |
| M24 | la région déplacée **dans** le `{#if selectionHasUnmatched}` (rendue avec l'indication) | V21 |
| M25 | `role="status"` et `aria-live="polite"` retirés de la région | V21 |
| M26 | le `<p>` de l'indication sorti de la région, placé après elle | V21, V22 |
| M27 | la région enveloppée d'un `{#key selectionHasUnmatched}` (nœud recréé au changement) | V21 |
| M28 | `aria-atomic="true"` retiré de la région (ou passé à `false`) | V21 |
| M29 | la règle `counterpartyName ?` remplacée par `counterpartyName != null ?` | V20 (cas `''`) |
| M30 | les deux appels littéraux remplacés par un seul appel à clé conditionnelle | V23 (relevé des deux clés), et `i18n-keys.test.ts` (`sitesNonResolus`) |

*Zone partagée avec la 29-6* (§ *Zones partagées*) : le bloc de l'indication (`:316-323`), la case (`:357-363`) et le
test de position (`:965-986`) sont de la 29-6 ; la 29-9 les **enveloppe** ou leur **ajoute** un attribut, en delta sur
le texte rebasé, sans rien en retirer.

*Exclusions* : les autres cases à cocher du frontend (`git grep -n 'type="checkbox"' -- frontend/src` → **24** sites
à `fcd2ae75`, dont un seul dans `features/reconciliation`) ne sont pas revendiquées ; aucune case « tout cocher »
n'est ajoutée ; les autres messages de l'écran (succès, erreur, bilan des échecs) ne passent pas en région d'annonce
— #678 ne vise que l'indication de la 29-6 (C-29-9-22).

## Tasks / Subtasks

- [x] **T0** — Montage : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-9/target` ; `node_modules` par `cp -al` depuis le
  dépôt principal ; bases `kesh_299` et `kesh_e2e_299` (migrations du worktree, `scripts/seed-dev-db.sql`) ; port
  3049. **Rebaser d'abord** sur `main` si la 29-3, la 29-6 ou la 29-8 y sont mergées, puis relire chaque numéro de
  ligne de cette fiche.
- [x] **T1** — #649 contrat (AC4) : constantes frontend **et**, dans le même geste, le mock de
  `ReconciliationProposals.test.ts` passé à `importOriginal` (AC9, étape 0, en delta ; fichier relancé vert) ;
  commentaire de `reconciliation.types.ts:52-55` ; commentaire de module de `kesh-db` `:18-19` réécrit (AC4) ;
  doc-comments Rust, deux tests Rust ; M6, M18, M20 ; **gate complet à ce commit** (exception *kesh-db*, AC10) ; commit
  `test(29-9): … (refs #649)`.
- [x] **T2** — #649 écran (AC1-AC3) : V2-V5 et V18 rouges, puis le correctif ; V1 et V12 exigent une **absence** et
  sont donc verts au sol par construction — ils se déclarent tels, et V12 vaut par M14 ; M1-M5, M14, M21 ; commit
  `fix(reconciliation): … (refs #649)`.
- [x] **T3** — #655 (AC5, AC6) : V6-V9 et V13-V17 rouges (V13 l'est au sol : l'effet actuel relit à id égal), puis les
  deux pages ; M7-M10, M13, M15-M17, M19 ; commit `fix(bank-import): … (refs #655)`.
- [x] **T4** — #646 (AC7) : V10-V11 rouges, puis le correctif ; M11-M12 ; commit `fix(contacts): … (refs #646)`.
- [x] **T10** *(amendement #678 — s'exécute après T4 et avant T5, malgré son numéro)* — AC11 : V19-V23 rouges
  (sortie copiée), puis le libellé des cases, la région d'annonce, le test de position de la 29-6 amendé en delta ;
  deux clés ×4 **posées en deux appels `i18nMsg` littéraux** (C-29-9-24) et le test de relevé des deux clés dans
  `i18n-entrees-a-variables.test.ts` (rouge au sol) ; M22-M30 ; captures de `reconciliation-reject-unmatched.spec.ts` relancée comparées à celles de la
  29-6 ; commit `fix(reconciliation): … (refs #678)`. Si la 29-6 n'est pas mergée au moment de T0, T10 attend : sans
  elle, l'indication n'existe pas (C-29-9-23).
- [x] **T5** — i18n (AC8) : quatre clés, une réécrite, ×4 ; `sitesTotal` recompté ; `KNOWN_VIOLATIONS` de
  `lint-i18n-ownership.js` (+1) ; inventaire des boutons rouges ; gardes i18n nommées à l'AC8 relancées.
- [x] **T6** — Manuel FR, `make fr` sous le verrou, contrôle aplati ; `api-external.md` ; CHANGELOG (quatre puces,
  AC11 compris ; manuel inchangé pour #678, C-29-9-22) ; commit `docs(29-9): … (refs #649, refs #646, refs #678)`.
- [x] **T7** — E2E (AC9) : deux specs, captures clair et sombre (hors dépôt, chemins au Dev Agent Record) ; le volet
  #678 de `reconciliation-show-older.spec.ts` (AC11) ; commit `test(e2e): … (refs #649, refs #646, refs #678)`.
- [x] **T8** — Propagation (§ ci-dessous), puis gates complets au dernier commit de code (AC10).
- [x] **T9** — Dev Agent Record, File List, Change Log ; statut `review` ; registre de sprint. Rapport : demander le
  push, la PR (`closes #649 #655 #646 #678` au corps seulement), le commentaire de #655 (C-29-9-8), et la décision sur une
  issue de pagination au-delà de 500 (C-29-9-3).

## Dev Notes

### Dérogation à la règle de découpage

Modules touchés au sens de la règle : (1) le frontend — trois zones, `features/reconciliation`, `routes/(app)/
bank-import`, `features/contacts` ; (2) `kesh-api` — deux doc-comments et deux tests, **aucun code de production** ;
(3) les catalogues `kesh-i18n` ; (4) la documentation (manuel, PDF, `api-external.md`, CHANGELOG). **Quatre**, sous
la limite de cinq : le critère « scope cross-cutting » n'est pas atteint. Si l'on compte chaque zone du frontend
comme un module, on en compte six : la section tient alors lieu de la `Dérogation règle de splitting`, **déclarée au
Project Lead par le rapport de l'orchestrateur**. *Annotation du 2026-10-14 (validation P2, R1 = F1)* : la réécriture
du commentaire de module de `kesh-db` (AC4) ajoute un module, (5) `kesh-db` — **un commentaire**, aucun code. Cinq dans
la première lecture, ce qui reste sous le seuil (« plus de 5 ») ; sept dans la seconde, couverte par la même
dérogation déclarée. Dans les deux lectures, pas de découpage : les trois correctifs sont
indépendants, chacun tient en un ou deux fichiers de production, aucun ne change une règle métier, et la revue se
fait correctif par correctif (précédent : 28-8, C-28-8-1). Un correctif fautif **sort du périmètre** plutôt que de
retenir les deux autres. C-29-9-1.
*Annotation du 2026-10-14 (amendement #678)* : recompte à la source avec l'AC11. #678 touche
`features/reconciliation` (le composant et son test, déjà comptés pour #649), les catalogues `kesh-i18n` (déjà
comptés) et le CHANGELOG (documentation, déjà comptée) ; sa spec E2E est celle de #649. **Aucun module neuf** :
première lecture, **cinq** (frontend, `kesh-api`, `kesh-i18n`, documentation, `kesh-db`) — sous le seuil ; seconde
lecture, zones du frontend comptées à part, **sept** (`features/reconciliation`, `routes/(app)/bank-import`,
`features/contacts`, `kesh-api`, `kesh-i18n`, documentation, `kesh-db`) — inchangé, couvert par la même dérogation
déclarée. Le quatrième correctif est indépendant des trois autres sauf de #649, avec qui il partage le composant et
la spec E2E, sans partager de ligne (la case et la rangée des boutons pour #678, l'après-`</table>` pour #649). La
dérogation **tient** ; un correctif fautif, #678 compris, sort du périmètre plutôt que de retenir les autres (C-29-9-19).

### Zones partagées avec les fiches voisines (non mergées)

| Fiche | Ce qu'elle touche de commun | Ce que fait la 29-9 |
|---|---|---|
| 29-3 (#527) | `ReconciliationProposals.svelte` : en-tête `:8`, paragraphe de priorité **avant** la table ; `i18n-keys.test.ts` `sitesTotal` +1 ; manuel `:2025-2060`, dont `:2045-2050`, `:2049`, `:2056` | ajout **après** `</table>` ; `sitesTotal` recompté ; phrase `:2045` réécrite **sur la version 29-3** |
| 29-6 (#665) | même composant : case sur toutes les lignes, *Accepter* grisé, indication dans la rangée des boutons, garde d'`onAccept` ; s'appuie sur « `load()` vide `selected` » (état (a)) ; manuel `:2050`, `:2056`, **`:2057`** (paragraphe *Rejeter* réécrit : « écarte la transaction, sans écriture »), `:2901`, `:2910-2911` | ne touche ni la case, ni la rangée, ni `onAccept` ; **garde** la remise à vide de `selected` (C-29-9-5) ; à `:2057`, **delta** sur la parenthèse seule (AC8). *Amendement #678 (2026-10-14)* : **touche** désormais la case (un `aria-label` ajouté) et la rangée (le bloc de l'indication enveloppé d'une région d'annonce), et amende en delta le test de position de la 29-6 (`ReconciliationProposals.test.ts:977-978` à `d71c506b`) ; rien de la 29-6 n'est retiré, ni son `data-testid`, ni sa spec E2E (AC11) |
| 29-8 (#670) | même composant : troisième bouton de ligne, modale `LinkSettlementModal` (fiche 29-8 `:370`, emplacement non prescrit — naturellement après `</table>`, à côté des deux autres) ; manuel `:2021`, **`:2057`** (y garde « une transaction rejetée compte comme non rapprochée »), `:2043-2060` | ne touche pas la ligne de table ; avis et bouton **immédiatement** après `</table>`, avant toute modale, la sienne comprise (AC1 ; validation P2, F8) ; à `:2057`, **delta** sur la parenthèse seule (AC8) |
| 29-3, 29-6, 29-8 — `docs/api-external.md` | 29-3 `:549` et 29-6 `:551` (§ *Accepter des propositions*) ; 29-8 : ses deux routes en paragraphes « à côté de `…/unreject`, `:594` » (fiche 29-8 `:474`) | `:594`, **en delta** sur la phrase rebasée : la parenthèse « (100 par défaut) » seule, les paragraphes de la 29-8 gardés (validation P2, R6) |
| 29-8 — `reconciliation.api.ts`, `reconciliation.types.ts` | `getLinkCandidates`, `linkTransaction` et leurs types (fiche 29-8 `:387-388`) | deux constantes et le défaut de `getProposals` ; commentaire de `hasMore` (`.types.ts:52-55`) — ajouts **à côté**, rien de la 29-8 retiré (R6) |
| 29-3, 29-6, 29-8 — `ReconciliationProposals.test.ts` | fabriques et tests neufs (29-3 `:532`, 29-8 `:550`) ; la 29-6 réécrit le test `:179` (fiche 29-6 `:636`) ; la 29-8 peut ajouter des entrées au mock du module API | mock passé à `importOriginal` **en delta** sur la fabrique rebasée, toutes ses entrées gardées (AC9, étape 0 ; R6) ; tests neufs à la suite |

**Recensement au sol** (`grep -noE "user-manual\.tex:?[0-9]{3,4}(-[0-9]+)?|:2057|:2045|:2050|:2901|:2910"` sur les
trois fiches, 2026-10-14, validation P1 R4) : `:2057` est édité par **trois** fiches (29-6, 29-8, 29-9), `:2045` par
deux (29-3, 29-9) ; `:869-874` par la 29-9 seule. L'emplacement « après `</table>` » n'est prescrit par aucune des trois (le rappel `reconciliation-rejected-hint`
que garde la 29-6 est plus bas, `.svelte:455`, hors de la branche liste non vide).
*Précision du 2026-10-14 (validation P2, F8)* : « prescrit par aucune » reste vrai à la lettre, mais la 29-8 instancie
une modale sans dire où, et son emplacement naturel est ce même point ; d'où la règle de l'AC1 — l'avis et le bouton
d'abord, les modales ensuite, quelle que soit la fiche qui les a posées.

Les numéros de ligne des fiches voisines cités dans ce tableau sont ceux du 2026-10-14 (validation P2) : ces fiches
évoluent, et T0 les relit.

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
- **#678** *(amendement)* : seules la case de la table de réconciliation et l'indication de la 29-6 ; les 23 autres
  cases du frontend, une case « tout cocher » et les autres messages de l'écran ne sont pas revendiqués (AC11,
  *Exclusions* ; C-29-9-22).

### Propagation — le symptôme à greper après chaque patch

```sh
git grep -nE "\b100\b" -- docs/manual/fr/user-manual.tex | sed -n '/20[0-9][0-9]:/p'      # #649, zone réconciliation
git grep -nE "100 plus récentes|100 most recent|100 neuesten|100 più recenti" -- crates docs frontend/src
git grep -nE "pas de UI dédiée|Charger plus|L24\b" -- crates frontend/src docs              # #649, commentaires (Rust ET frontend ; R3/F3)
git grep -n "hasMore" -- frontend/src ':!*.test.ts'                                     # #649 : types + composant
git grep -n "issues/649" -- docs README.md website                                      # lien retiré d'api-external
git grep -n "errorMessage" -- "frontend/src/routes/(app)/bank-import"                    # #655
git grep -n "sans confirmation" -- frontend/src docs                                    # #646
git grep -n "contact-persons-confirm-delete" -- frontend crates/kesh-i18n                # #646 : site, repli, 4 locales, KNOWN_VIOLATIONS
git grep -n "page.params" -- "frontend/src/routes/(app)/bank-import/profiles"           # #655 : l'effet ne relit plus page.params (rawId seul)
git grep -ln "reconciliation.api'" -- frontend/src                                     # mocks du module API (R1/F1)
git grep -n "onUpdated\|onSuccess=" -- "frontend/src/routes/(app)/bank-import/profiles" # #655 : garde d'onUpdated (P2, F2)
git grep -n "hardcodé 100" -- crates                                                    # kesh-db réécrit (P2, R1 = F1) : vide
git grep -n "reconciliation-labels-select-transaction" -- frontend crates/kesh-i18n     # #678 : 2 clés × (site + repli + 4 locales)
git grep -n "reconciliation-selection-status\|aria-live" -- frontend/src/lib/features/reconciliation frontend/tests/e2e  # #678 : région, test, E2E
git grep -n "reconciliation-unmatched-not-acceptable-hint" -- frontend                  # #678 : contrat de la 29-6 inchangé (site, Vitest, E2E)
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "plus récentes.\{0,200\}"
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "personnes de contact.\{0,300\}"
```

Les occurrences légitimes se trient à la main et se citent au Dev Agent Record. Liste attendue **après** la story
(validation P2, R9) :

- le CHANGELOG `[0.13.3]` (publié) ;
- la valeur 100 des constantes (`PROPOSALS_DEFAULT_LIMIT`, `DEFAULT_PROPOSALS_LIMIT`) et des doc-comments qui les
  nomment, dont le commentaire de module de `kesh-db` réécrit (« défaut 100, plafond 500 ») ;
- la clé réécrite `reconciliation-unreject-success`, dans ses **quatre** traductions (« montre **d'abord** les 100 plus
  récentes… », « first », « zuerst », « prima »), et son repli `UnrejectTransactionAction.svelte:49-50` ;
- la phrase neuve du manuel à `:2045` (« les 100 plus récentes d'abord ») et son reflet dans le PDF.

Ce qui ne doit **plus** sortir : `L24` et « hardcodé 100 » (`kesh-db` réécrit, C-29-9-16), « pas de UI dédiée » et
« Charger plus » (doc-comment Rust et `reconciliation.types.ts` réécrits), « 100 plus récentes au plus » et « qui ne
montre que les 100 plus récentes ».

### Pièges connus du dépôt, applicables ici

- **Effet qui suit `limit`** (AC2) : poser `limit` puis appeler `load()` double la requête ; V3 l'attrape.
- **`page.params` est réassigné à chaque navigation où `page_changed` est vrai** — URL, erreur, formulaire ou données
  changés, `client.js:763-772` (AC6) : un effet qui lit `page.params.id` se rejoue à id égal ;
  dériver la **chaîne** (`$derived(page.params.id)`), pas le nombre (`NaN !== NaN`). V13 l'attrape.
- **Mock par fabrique sans `importOriginal`** : un export neuf du module mocké est absent du mock et fait lever
  Vitest à l'accès ; `ReconciliationProposals.test.ts` (AC9, étape 0) — et `profile-page.test.ts`, dont la fabrique
  de `bank-profile.api` doit gagner `updateBankProfile` (et `createBankProfile`) pour V17.
- **Un composant Svelte 5 lit ses props en direct** (`$$props.x` à chaque accès, même démonté) : un numéro de séquence
  capturé dans une fermeture passée en prop ne garde rien (AC6, C-29-9-17).
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
- [#678] (`gh issue view 678`, lue le 2026-10-14, amendement) ; C-29-6-25 (registre de la 29-6,
  `kesh-29-6/_bmad-output/implementation-artifacts/epic-29-choix-autonomes.md`) ; composant et tests de la 29-6 lus à
  `d71c506b`.
- `crates/kesh-api/src/routes/reconciliation.rs:56-90`, `:578-631` ; `crates/kesh-db/src/repositories/reconciliation.rs:18-19`
  (commentaire de module réécrit), `:276-311` ;
  `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116`.
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `reconciliation.api.ts:16-26`,
  `reconciliation.types.ts:52-55`, `ReconciliationProposals.test.ts:22-27` (mock) ; `frontend/scripts/lint-i18n-ownership.js:117-139`.
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

Claude Opus 5.5 (`bmad-dev-story`, autonomie), 2026-10-14/15. Worktree `/home/gcorbaz/devel/kesh-29-9`, bases
`kesh_299` / `kesh_e2e_299`, port 3049, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-9/target`.

### Debug Log References

Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/` : `299-t1-cible.log` (tests Rust ciblés), `299-gate-t1.log`
(gate complet au commit T1, exception *kesh-db*), `299-mut-rust.log`, `299-mut-t2.log`, `299-mut-t3.log`,
`299-mut-t4.log`, `299-mut-t10.log`, `299-mut-m30-bis.log` (mutations), `299-make-fr.log` (PDF),
`299-e2e-cible.log` (trois specs), `299-gate-final.log` (gates au `9a1e5eb0`, `npm run check` rouge),
`299-gate-final2.log` (gates au dernier commit de code `284c1d3b`), `299-e2e-complet.log` (E2E complet au même
commit). *Annotation du 2026-10-15, report sur `1261c4bb`* : les journaux de référence sont désormais
`299-rebase2-gate.log`, `299-rebase2-npm-{check,lint-i18n-ownership,test:unit,build}.log`,
`299-rebase2-e2e-complet.log` (et `-e2e-playwright.log`), `299-rebase2-mut.log`, `299-rebase2-make-fr.log` ; les
précédents ne valent que pour le commit d'avant le report. *Revue P1* : `299-p1fix-make-fr.log` (PDF régénéré). Scripts et motifs des mutations : `kesh-gate-logs/299-scripts/` (`mutate.py`, `mutcamp.py`, `t3.json`,
`t4.json`, `t10.json`, `m30.json`, `mut-rust.sh`, `mut-t2.sh`).

### Completion Notes List

**Étape A — rebase sur `4b358373`** (tête de `story/29-8`, qui contient `main` `12c64c39`). Treize commits de fiche
rejoués ; conflits sur `sprint-status.yaml` (à chaque commit) et sur le fichier d'epic (commits 1, 8, 12), résolus
par `union_sprint.py` (clés de la 29-9 de la branche, autres de la base) et `union_epic.py` (les deux côtés, ligne
v0.13.4 unique). Deux défauts de l'union, corrigés au commit `08b57ee7` : les en-têtes de la 29-9 rejoués **sans
conflit** se sont doublés (le 135 deux fois, deux versions du 136) — ramenés à **164** et **163** en tête ; l'union du
fichier d'epic a gardé les **versions successives** de lignes que la branche réécrivait (titre « Trois… » et
« Quatre… », trois variantes de la ligne de release de la 29-9) — seule la dernière est gardée. Fiche et registre de
la 29-9 **identiques** à `b4794e7c` (`git diff --quiet`), clé de `sprint-status.yaml` identique, `yaml.safe_load`
vert, aucun marqueur. Non touché : une ligne orpheline du fichier d'epic qui vient de la base (« `29-6, R7)* : Guy
les éprouve…` », l. ~185, déjà à `4b358373`) — relève de l'orchestrateur.

**Étape B — prémisses relues sur la base.** Exactes sur le fond ; numéros de ligne décalés (lus au code, non
reportés dans le corps de la fiche) :
- composant de réconciliation : la case est sur toute ligne (29-6), l'indication est le dernier enfant de la rangée
  des boutons, `LinkSettlementModal` (29-8) suit les deux autres modales après `</table>` ; la 29-8 ajoute un
  troisième bouton de ligne (*Rattacher à un règlement*, crédits en CHF) et un quatrième geste qui fait sortir une
  transaction de la liste — **d'où C-29-9-26** : l'avis plafonné et le manuel le nomment (texte, aucune règle
  métier) ; le mock du module API de `ReconciliationProposals.test.ts` porte toujours quatre entrées (la modale de
  la 29-8 est une doublure) ;
- `routes/reconciliation.rs` : constantes `:62-67`, `has_more` `:93-97`, `limit + 1` `:707`, comparaison `:710` ;
  commentaire de `kesh-db` à `:35-36` (« hardcodé 100 … L24 », toujours faux) ; le doc-comment de
  `DEFAULT_PROPOSALS_LIMIT` portait aussi « cf. L24 », retiré ;
- `docs/api-external.md` : la parenthèse « (100 par défaut) » est au paragraphe *unreject* (`:610`) ;
- manuel : `:2087` (« au plus ») et `:2099` (parenthèse) ; la phrase de la 29-6 « la page le dit » à `:2092` ;
- les deux pages de #655, `ContactPersonsManager` et les catalogues : conformes au constat.

**Tests vus rouges avant le correctif** (sorties dans la conversation de développement ; résumé exact) :
*Annotation du 2026-10-15, revue P1 (A-7) — écart à l'AC9 (« sortie copiée au Dev Agent Record ») : les sorties
rouges ne sont **pas** copiées ici, seulement résumées. Les sorties **des tests rouges sous mutation** sont dans les
journaux `299-mut-t2.log`, `299-mut-t3.log`, `299-mut-t4.log`, `299-mut-t10.log`, `299-mut-rust.log` ; le rouge
**avant correctif** de T2 à T4 et de T10 n'a, lui, de trace que dans la conversation de développement (aucun journal
ne le porte).*
- T2 : V2, V3, V4, V5, V12, V18 rouges (`Unable to find … reconciliation-more` / `…show-older-btn`) ; V1 vert au sol
  (il exige une absence) ;
- T3 : V6, V7, V8, V9, V13, V14, V15, V16, V17 rouges, chacun sur son symptôme (bannière présente ; lignes rejetées
  périmées ; formulaire absent ; message ancien ; `getBankProfile` appelé 2 fois au lieu d'une ; « Chargement… »
  absent ; « ID invalide » écrasé ; `Banque 5` au lieu de `Banque 6`) ;
- T4 : V10 et V11 rouges (`confirm` appelé 0 fois) — V11 l'est parce qu'il affirme aussi la question (C-29-9-28) ;
- T10 : V19 et V20 rouges d'abord sur l'absence de la clé au catalogue, puis, les deux clés posées, sur l'absence du
  nom accessible (`Unable to find an accessible element with the role "checkbox" and name "Sélectionner la
  transaction du 2026-05-15, 100.00 CHF, ACME GMBH"`) ; V21 et V22 rouges (pas de région) ; V23 rouge (deux clés non
  relevées).
- Rust : `get_proposals_limit_is_capped_at_500` et `get_proposals_limit_500_exact_has_no_more` figent un
  comportement **déjà** en place : verts au sol, leur pouvoir se prouve par M6, M18, M20.

**Mutations — 30 + 2 rejouées, toutes tuées** (motif vérifié unique, restauration `git checkout -- f` puis `touch f`) :
- M1 (V2-V5, V12, V18), M2 (V3, V4, V5), M3 (V3, V5), M4 (V4), M5 (V2, V3, V5), M14 (V12), M21 (V18) ;
- M6 (les deux tests Rust neufs), M18 (`…_500_exact_has_no_more`), M20 (`…_capped_at_500` et
  `…_paginates_at_100_default`, comme la fiche le prévoit) ;
- M7 (V6), M8 (V7), M9 (V8), M10 (V9, V16), M13 (V14, V17), M15 (V13), M16 (V15), M17 (V16), M19 (V17) ;
- M11 (V10), M12 (V11) ;
- M22 (V19, V20, V23), M23 (V20), M24 (V21), M25 (V21), M26 (V21, V22), M27 (V21), M28 (V21), M29 (V20), M30 (V20,
  V23 ; rejouée après T5 : aussi `i18n-keys.test.ts`, « la collecte ne passe pas à vide » et « l'inventaire des sites
  non résolus est borné ») ;
- M15 rejouée sur la fixture définitive (C-29-9-27) : tuée par V13.
- *Annotation du 2026-10-15, revue P1 (A-6)* : **M9, M10, M13, M16, M17 et M19 ont tourné sur la `class FakePage`**
  (`299-mut-t3.log`, 23:50:45), **avant** `85d8d9ea` (23:54:23), qui la remplace par l'objet à accesseurs
  (C-29-9-27) ; **seule M15 a été rejouée** sur la doublure définitive. Les six autres n'ont **pas** été rejouées
  depuis : le « toutes tuées » vaut, pour elles, pour un arbre qui n'est pas celui du commit (les deux doublures ont
  la même sémantique, un `$state.raw` réassigné ; aucun rejeu n'a vérifié l'équivalence).
⚠️ **Écart de procédure, sans effet sur les résultats** : la campagne Rust (M6, M18, M20) a tourné **avant** le commit
de T1 ; sa restauration par `git checkout` a ramené `routes/reconciliation.rs` à HEAD et **effacé les doc-comments**
non commités de T1, réécrits ensuite à l'identique avant le commit. Les mutations portaient sur du code identique à
celui du commit (seuls des commentaires diffèrent). Les campagnes suivantes ont toutes tourné sur un arbre commité.

**Gates.**
- Commit T1 `affe1dd1` (exception *kesh-db*) : `scripts/test-fast.sh` sur `kesh_299` remise à zéro — fmt, clippy,
  nextest **3732 passés, 6 ignorés** (`299-gate-t1.log`).
- Au `9a1e5eb0` (`299-gate-final.log`) : test-fast vert (3732/3732), `test:unit` vert, `build` vert, lint vert,
  **`npm run check` rouge** — une erreur de type dans `ContactPersonsManager.test.ts` (`vi.fn(() => false)` sans
  paramètre, `calls[0][0]` sur un tuple vide). Corrigé au `284c1d3b`, qui devient le dernier commit de code.
- **Au dernier commit de code `284c1d3b`** (`299-gate-final2.log`, sous verrou et mem-guard, base `kesh_299` remise à
  zéro dans le même journal) : `scripts/test-fast.sh` — fmt, clippy `-D warnings`, nextest **3732 passés, 6
  ignorés** ; `npm run check` **0 erreur** (29 avertissements, tous dans des fichiers que la story ne touche pas) ;
  `npm run lint-i18n-ownership` vert ; `npm run test:unit` **1680 passés, 149 fichiers** ; `npm run build` vert.
- **E2E complet au `284c1d3b`** (`299-e2e-complet.log`) : **291 passés, 7 échoués, 19 ignorés** (11,8 min ; montage de `docs/testing.md`, port 3049, base `kesh_e2e_299` remise à zéro, backend arrêté par PID). Les sept échecs, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les **sept KF-029 (#97)**, ni KF-052 ni pollution ; borne basse de la fourchette 7 à 10. Les deux specs neuves et `reconciliation-reject-unmatched.spec.ts` passent (journal Playwright : `kesh-gate-logs/299-e2e-complet-playwright.log`).
- Gardes i18n nommées à l'AC8, vertes dans `test:unit` : `i18n-entrees-a-variables.test.ts`,
  `i18n-repli-divergent-actif.test.ts`, `i18n-keys.test.ts`, `i18n-harvest.test.ts`.

**Décomptes** (périmètre : de `4b358373` au `284c1d3b`, lignes `it(` et `#[sqlx::test` comptées aux deux bornes) :
- Vitest : **+22** — `ReconciliationProposals.test.ts` 34 → 44 (+10 : V1-V5, V12, V18, V19-V21 ; V22 amendé, non
  compté), `bank-import-detail-page.test.ts` 5 → 7 (V6, V7), `profile-page.test.ts` 2 → 9 (V8, V9, V13-V17),
  `ContactPersonsManager.test.ts` 0 → 2 (V10, V11), `i18n-entrees-a-variables.test.ts` 5 → 6 (V23) ;
- Rust : **+2** (`reconciliation_e2e.rs` 106 → 108) ;
- E2E : **+2 specs** (73 → 75 fichiers `*.spec.ts` au premier niveau de `frontend/tests/e2e/` ; *annotation du
  2026-10-15, revue P1* : 74 → 76 en comptant `security/`, le delta est le même), un test chacune ;
- i18n : **six clés neuves** ×4 (`reconciliation-labels-more-hidden`, `-actions-show-older`, `-labels-more-capped`,
  `-labels-select-transaction`, `-select-transaction-no-counterparty`, `contact-persons-confirm-delete`) et **une
  réécrite** ×4 (`reconciliation-unreject-success`) ; `sitesTotal` **2180 → 2186** (+5 dans
  `ReconciliationProposals.svelte`, +1 dans `ContactPersonsManager.svelte`), recompté par `grep -o "i18nMsg("` aux
  deux bornes et égal à la collecte ; `KNOWN_VIOLATIONS` **+1** (dette #30 : `contact-persons-confirm-delete`).

**Écarts à la fiche** (choix au registre) :
- C-29-9-26 — l'avis plafonné et le manuel nomment aussi « rattachées à un règlement » (29-8) ;
- C-29-9-27 — la fixture `profile-page-state.svelte.ts` est un objet à accesseurs, pas la `class FakePage`
  prescrite : `i18n-libelle-en-dur.test.ts` interdit toute classe dans `src/` (vu rouge au gate ciblé) ;
- C-29-9-28 — V11 affirme aussi que `confirm` est appelé ;
- C-29-9-29 — seul `user-manual.pdf` est versé (`admin-manual.pdf`, `marketing-brochure.pdf` restaurés : octets
  régénérés, `.tex` inchangés) ;
- C-29-9-30 — E2E de #649 : un seul import de 501, chaque état capturé en clair et en « sombre » ; Kesh n'a pas de
  thème sombre (C-28-1-7), la capture « sombre » est donc claire, comme celles de la 29-6.
- Le `<p>` de l'indication de la 29-6 est **ré-indenté** d'un cran dans la région (octets de blancs seulement ; clé,
  repli, classe et `data-testid` inchangés).
- Manuel `:2045` (fiche) : rédigé en deux temps — « les 100 plus récentes d'abord » dans la phrase, puis un paragraphe
  après la liste des candidates qui dit l'avis, le bouton, la sélection vidée et l'avis plafonné.

**Recette visuelle.** Captures hors dépôt, `kesh-gate-logs/299-captures/` : `avis-et-bouton-{clair,sombre}.png`,
`avis-plafonne-{clair,sombre}.png` (jetons du thème : `text-text-muted`, `border-text`), et
`29-6-ligne-cochee-*`, `29-6-apres-rejet-*` relancées par `reconciliation-reject-unmatched.spec.ts` non modifiée :
**mêmes dimensions** que celles de la 29-6 (`kesh-gate-logs/298-captures/`, 1008×134 et 1008×92) et même disposition,
aux données près — la région vide n'ajoute aucun écart visible. Aucune capture de la confirmation (dialogue natif,
C-29-9-10) ; pages de #655 : rendu inchangé.

⚠️ **Écart à l'AC2, déclaré le 2026-10-15 (revue P1, A-2) — saut de défilement (C-29-9-15), capture non prise.**
Tel qu'il est au code : *Afficher aussi les plus anciennes* ne fait que poser `limit = 500` ; l'`$effect` relance
`load()`, qui pose `loading = true` ; `{#if loading}` remplace alors la table, l'avis et le bouton par le message
« Chargement des propositions… », le document se raccourcit et la page remonte ; la réponse revient, `loading`
retombe, la liste à 500 lignes réapparaît **sans restauration de la position** de défilement. `load()` n'a pas
changé sur ce point (C-29-9-15). Le saut **n'a pas été observé en séance ni capturé** : l'AC2 exigeait « la capture »
au Dev Agent Record, elle manque, et les quatre captures `avis-*` ne montrent que l'avis. Le comportement se lit au
code, il n'a pas été constaté à l'écran.

**Propagation** (grep de la fiche, § *Propagation*) : ne sortent plus « hardcodé 100 », `L24`, « pas de UI dédiée »,
« Charger plus », « 100 plus récentes au plus », « qui ne montre que les 100 », `issues/649` ; sortent seulement les
occurrences légitimes attendues (constantes et doc-comments, `reconciliation-unreject-success` ×4 et son repli,
« les 100 plus récentes d'abord » au manuel, la ligne « score de 100 % » `:2083`, sans rapport). PDF aplati : les
trois phrases neuves présentes, l'ancienne absente (`grep -c` → 0), « la page le dit » présente.

**Demandes à l'orchestrateur** : la PR (après le merge de la 29-8 et ton `git rebase --onto origin/main 1261c4bb8725fec33f6b7920947ed005ee8bd41b story/29-9`
— *corrigé le 2026-10-15, revue P1 (A-1) : la commande d'origine portait `4b358373`, base d'avant le report ; la
branche repose sur `1261c4bb`, dont l'arbre est identique à `origin/main` `1b136fad`*),
corps avec `closes #649 #655 #646 #678`, message de squash rédigé à la main ; le commentaire de #655 (défaut latent sur
la page d'un profil, C-29-9-8) ; la décision sur une issue de pagination au-delà de 500 (C-29-9-3) ; la ligne
orpheline du fichier d'epic (ci-dessus).

### File List

- `crates/kesh-api/src/routes/reconciliation.rs` (doc-comments seulement)
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-db/src/repositories/reconciliation.rs` (commentaire de module seulement)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts`
- `frontend/src/lib/features/reconciliation/UnrejectTransactionAction.svelte`
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte`
- `frontend/src/routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts`
- `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte`
- `frontend/src/routes/(app)/bank-import/profiles/[id]/profile-page.test.ts`
- `frontend/src/routes/(app)/bank-import/profiles/[id]/profile-page-state.svelte.ts` (neuf)
- `frontend/src/lib/features/contacts/ContactPersonsManager.svelte`
- `frontend/src/lib/features/contacts/ContactPersonsManager.test.ts` (neuf)
- `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts`
- `frontend/scripts/lint-i18n-ownership.js`
- `frontend/tests/e2e/reconciliation-show-older.spec.ts` (neuf)
- `frontend/tests/e2e/contact-persons-delete.spec.ts` (neuf)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `docs/api-external.md`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md`, `epic-29-choix-autonomes-29-9.md`,
  `sprint-status.yaml`, `29-9-review-prompt-p1.md` ; `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`
  (rebase)

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie), sur `fcd2ae75`. Prémisse du second cas de
  #655 rectifiée (défaut latent, C-29-9-8). Choix C-29-9-1 à 12.

- 2026-10-14 — **Validation P1 remédiée** (Opus 5.5 ; passe Sonnet, lentilles R et F, rapports
  `kesh-gate-logs/29-9-validate-p1-{R,F}.md`). Bilan de la passe : R **0/0/4/5**, F **0/0/5/6** (CRITICAL/HIGH/MEDIUM/LOW),
  soit 20 findings dont trois paires convergentes (R1=F1, R2=F2, R3=F3) : **17 défauts distincts**, 6 MEDIUM et 11 LOW.
  **Tous traités, aucun refus.**
  - R1/F1 (MEDIUM) — mock de `./reconciliation.api` passé à `importOriginal` **avant** V1-V5 (AC9 étape 0, T1).
  - R2/F2 (MEDIUM) — `contact-persons-confirm-delete` entre dans `KNOWN_VIOLATIONS` (AC8, T5) ; dette #30 +1.
  - R3/F3 (MEDIUM) — commentaire de `reconciliation.types.ts:52-55` réécrit (AC4) ; grep de propagation étendu à
    `crates frontend/src docs` ; occurrence `kesh-db` triée et laissée (C-29-9-16).
  - F4 (MEDIUM, décision de l'orchestrateur) — prémisse de C-29-9-8 **fausse au code** (SvelteKit réassigne
    `page.params` à chaque navigation), corrigée par annotation datée (fiche et registre) ; AC6 réécrit : `$derived` de
    la chaîne `page.params.id`, remise à zéro et `isLoading = true` au début d'un chargement réel, établi sans effet sur
    le formulaire à id égal ; fixture qui réassigne `params` ; V13 (même id : rien ne se relance) ; C-29-9-13.
  - F5 (MEDIUM) et R8 (LOW) — V14/M13 (`isLoading = true`), V15/M16 (garde sur `isLoading`) ; `loading` du détail non
    gardé, sans effet observable (C-29-9-14, avec F7).
  - R4 (MEDIUM) — zones partagées complétées (29-3 `:2049`/`:2056`, 29-6 `:2056-2057`/`:2901`/`:2910-2911`, 29-8
    `:2021`/`:2057`) ; AC8 `:2057` écrit en delta sur la phrase rebasée.
  - LOW : R5 (C-29-9-12 corrigé par annotation : 501, 150e) ; R6 (gardes i18n nommées, apostrophe du repli) ; R7
    (séquence avant la branche « ID invalide », V16/M17, et au détail) ; R9 (numéros de ligne `:112-119`, `:290-298`) ;
    F6 (V12/M14) ; F7 (C-29-9-14) ; F8 (second test Rust à **exactement 500**, M18) ; F9 (saut de défilement
    **déclaré**, C-29-9-15) ; F10 (ordre de l'assertion de V3, `beforeEach` de la fixture) ; F11 (entrées CAMT
    uniques, 501 vérifiées à l'import ; « dates antérieures » retiré, `seedTestState` remet la base à zéro).
  - Recomptes à la source après remédiation : 10 AC, T0-T9 (10), **V1-V16** (16 ; +5 : V12-V16), **M1-M18** (18 ;
    +6 : M13-M18), choix **C-29-9-1 à 16** (+4 : 13-16). Symptômes grepés : `pas de UI dédiée|Charger plus|L24` (4
    sites : deux doc-comments Rust prescrits, `reconciliation.types.ts:54` désormais prescrit, `kesh-db` trié) ;
    `101` dans fiche et registre (C-29-9-12 seul, corrigé) ; `ne se rejoue que si` (fiche et registre, annotés) ;
    `111-119`, `291-299` (aucun résidu).

- 2026-10-14 — **Validation P2 remédiée** (Opus 5.5 ; passe Opus, lentilles R et F, rapports
  `kesh-gate-logs/29-9-validate-p2-{R,F}.md`). Bilan de la passe : R **0/0/1/8**, F **0/0/2/7**, soit 18 findings dont
  quatre paires convergentes (R1=F1, R2=F3, R3=F4, R5=F9) : **14 défauts distincts**, 2 MEDIUM et 12 LOW. Tendance :
  P1 6 MEDIUM → P2 2 MEDIUM. **Tous traités, aucun refus** ; décisions de l'orchestrateur consignées comme telles.
  - R1 = F1 (MEDIUM, **décision de l'orchestrateur**) — le commentaire de module `kesh-db/src/repositories/
    reconciliation.rs:18-19`, faux depuis la 8-4, est **réécrit en T1** (AC4) ; C-29-9-16 annoté et renversé.
    `kesh-db` entre dans le diff pour un commentaire : l'exception *kesh-db* ne vaut que pour le commit de T1 (et un
    patch de revue qui retoucherait ce fichier), gate complet à ce commit-là (AC10, T1) ; module de plus déclaré
    (§ *Dérogation*, C-29-9-1 annoté).
  - F2 (MEDIUM, **décision de l'orchestrateur**) — `onUpdated` gardé (AC6, C-29-9-17), V17, M19. ⚠️ La décision dit
    « la même garde de séquence que le chargement » ; sa forme littérale (numéro capturé dans une fermeture passée au
    formulaire) ne tient pas, le formulaire lisant ses props en direct (`$$props.onSuccess`, établi par compilation d'un
    composant d'essai sous Svelte 5.55.1). La garde est donc évaluée à l'appel, sur l'identifiant courant
    (`updated.id !== Number(rawId)`) — même effet voulu : une sauvegarde qui finit après une navigation vers un autre
    profil n'écrit plus `profile`. Signalé à l'orchestrateur.
  - LOW : R2 = F3 (la mutation `limit` pour `limit + 1` n'est pas tuée par le test à 500 mais par celui à 501 ; phrase
    corrigée, mutation inscrite M20) ; R3 = F4 (`:627` → `:628`) ; R4 (repli `reconciliation.limits.ts` : import direct
    par le composant ; `importOriginal` fait entrer `api-client` et ses imports, écrit à l'étape 0) ; R5 = F9
    (`KNOWN_VIOLATIONS` : bloc `:117-139`, insertion après `:139`, deux sites) ; R6 (trois zones partagées ajoutées :
    `api-external.md`, `reconciliation.api.ts`/`.types.ts`, `ReconciliationProposals.test.ts` ; étape 0 et `:594` en
    delta sur le texte rebasé) ; R7 (`sprint-status.yaml` : « 1 à 18 ») ; R8 (bouton désactivé pendant un lot : V18,
    M21) ; R9 (liste des occurrences légitimes attendues après la story, et de celles qui ne doivent plus sortir) ; F5
    (`page_changed`, `client.js:763-772` : formulation exacte, cinq sites vivants corrigés, deux annotations datées) ;
    F6 (commentaire au site plutôt qu'`untrack`, C-29-9-18) ; F7 (V1 et V12 verts au sol, « V2-V5 rougissent ») ; F8
    (avis et bouton immédiatement après `</table>`, avant toute modale dont celle de la 29-8 ; C-29-9-11 annoté).
  - Recomptes à la source après remédiation : 10 AC, T0-T9 (10), **V1-V18** (18 ; +2 : V17, V18), **M1-M21** (21 ;
    +3 : M19-M21), choix **C-29-9-1 à 18** (+2 : 17, 18). Symptômes grepés : `chaque navigation|toute navigation`
    (fiche et registre ; sites vivants corrigés, entrées datées annotées, Change Log P1 laissé) ; `:627`, `117-133`
    (aucun résidu vivant) ; `reste vrai` (registre C-29-9-16, annoté et renversé) ; `V1-V5` (étape 0 corrigée ; Change
    Log P1 laissé) ; `C-29-9-1 à 12` (`sprint-status.yaml`, corrigé).

[#646]: https://github.com/guycorbaz/kesh/issues/646
[#649]: https://github.com/guycorbaz/kesh/issues/649
[#655]: https://github.com/guycorbaz/kesh/issues/655
[#678]: https://github.com/guycorbaz/kesh/issues/678
- 2026-10-14 — **Validation P3, passe ciblée** (Haiku 5.5, une lentille sur `13ac02c5` ; rapport
  `kesh-gate-logs/29-9-validate-p3-ciblee.md`) : 0/0/0/1. **Validation close.** Le LOW est à appliquer à l'étape 0 du
  développement : P3-1 (l. 506, les citations « 29-3 :549 » et « 29-6 :551 » désignent `docs/api-external.md`, pas les
  fiches — écrire « `docs/api-external.md:549`, cité par la fiche 29-3 l. 534 », idem 29-6 l. 347). Trend : P1 R 0/0/4/5
  F 0/0/5/6 → P2 R 0/0/1/8 F 0/0/2/7 → P3 ciblée 0/0/0/1.

- 2026-10-14 — **Amendement #678** (Opus 5.5, décision de l'orchestrateur), après la clôture de la validation par la
  P3 ciblée. Un quatrième correctif entre dans la story : [#678] (`bug`, `P4`), cases sans libellé accessible et
  indication de la 29-6 non annoncée. **AC11** neuf (libellé par case à deux clés à variables ×4, région
  `role="status"` `aria-live="polite"` présente dès le rendu de la liste et englobant l'indication, contrat du
  `data-testid` de la 29-6 intact, son test de position amendé en delta) ; tâche **T10** (après T4, avant T5) ; V19-V22,
  M22-M27 ; E2E ajouté à `reconciliation-show-older.spec.ts`, aucune capture neuve ; manuel inchangé, une puce de
  CHANGELOG. Prémisses de #678 établies au code (exactes ; le second cas n'existe qu'avec la 29-6). Titre « Quatre
  petits correctifs », **clé inchangée** `29-9-trois-petits-correctifs`. Renvois datés dans le statut, l'énoncé, AC8
  (CHANGELOG, `sitesTotal`), AC10, T6, T7, T9, la dérogation, les zones partagées, les exclusions, la propagation ;
  AC1-AC10 inchangés sur le fond. Modules recomptés : cinq / sept, aucun neuf, dérogation tenue. Choix **C-29-9-19 à
  23**. Recomptes à la source : **11 AC**, T0-T10 (**11**), **V1-V22** (22 ; +4), **M1-M27** (27 ; +6), choix
  **C-29-9-1 à 23** (+5). Passe ciblée P4 à lancer : `29-9-validate-prompt-p4-ciblee.md` (Haiku, une lentille).

- 2026-10-14 — **Validation P4, passe ciblée remédiée** (Opus 5.5 ; Haiku 5.5, une lentille sur l'amendement #678 ;
  rapport `kesh-gate-logs/29-9-validate-p4-ciblee.md`) : **0/0/2/6**. Tous traités, aucun refus.
  - P4-1 (MEDIUM) — le corps de la section 29-9 du fichier d'epic dit « −4 issues, −4 bugs, quatre défauts » ; les
    quatre labels `bug` relevés par `gh issue view` (#649, #655, #646 : `bug,P3` ; #678 : `bug,P4`).
  - P4-3 (MEDIUM) — établi au code : les trois gardes i18n ne retiennent que les appels à clé littérale
    (`i18n-entrees-a-variables.test.ts:125`, `i18n-repli-divergent-actif.test.ts:122`). AC11 prescrit la forme (deux
    appels littéraux sous un ternaire, jamais une clé calculée), un test de relevé des deux clés, M30, T10 (C-29-9-24).
  - LOW : P4-2 (énoncé « quatre défauts », registre : issues #649, #655, #646, #678) ; P4-4 (rendu : le `div` vide
    reçoit le `gap-2`, écrit) ; P4-5 (`aria-atomic` dans V21, M28) ; P4-6 (V20 à deux cas `null` et `''`, règle de
    véracité écrite, M29) ; P4-7 (commande de contrôle des mots-clés en `…:?\s+#[0-9]+`, message de squash à rédiger à
    la main) ; P4-8 (« même nœud » précisé : d'une sélection à l'autre, un `load()` recrée la région).
  - Laissé : la consigne du prompt P4 (`29-9-validate-prompt-p4-ciblee.md:88`) garde l'ancienne commande, relevé daté de
    la passe ; l'annotation de l'epic (l.117) reste un relevé daté. L'observation sans finding sur l'AC9 (cellule
    contrepartie vide) n'est pas traitée : la fixture CAMT porte `Acme SA`.
  - Recomptes à la source : **11 AC**, T0-T10 (**11**), **V1-V22** (22, inchangé), **M1-M30** (30 ; +3 : M28-M30),
    choix **C-29-9-1 à 24** (+1). Symptômes grepés : « trois défauts », « −3 », « Trois défauts », liste d'issues sans
    #678 (résidus vivants corrigés ; entrées datées et prompts de validation passés laissés).

- 2026-10-14 — **Validation P5, passe ciblée remédiée** (Opus 5.5 ; une lentille sur la remédiation P4, commit
  `5d930a20` ; prompt `29-9-validate-prompt-p5-ciblee.md`, rapport `kesh-gate-logs/29-9-validate-p5-ciblee.md`) :
  **0/0/1/5**. Tous traités, aucun refus ; principe : retirer plutôt qu'ajouter.
  - P5-1 (MEDIUM) — vérifié au code : `i18n-repli-divergent-actif.test.ts:146` ignore une clé à site unique non
    inscrite à `REPLIS_A_SITE_UNIQUE` ; l'affirmation « repli = valeur `fr-CH` gardé par cette garde » est retirée de
    l'AC11 et de l'AC8. AC11 : V19/V20 lisent leur attendu dans le catalogue `fr-CH` (précédents cités) ; AC8 : angle
    mort assumé (C-29-9-25).
  - LOW : P5-2 (ce que voit chaque garde, `i18n-keys.test.ts` `sitesNonResolus` rougi aussi par M30) ; P5-3 (M22-M30
    à l'annotation de l'AC10, Status, epic « quatre fermetures », en-tête de `sprint-status.yaml`) ; P5-4 (test de
    relevé numéroté V23) ; P5-5 (arguments du site écrits avec les champs de `p.transaction`) ; P5-6 (commande de
    contrôle élargie aux formes `owner/repo#N` et URL d'issue).
  - Laissé : entrées datées du Change Log (P1-P4, « V1-V22 », « M22-M27 ») et prompt P4 inchangés, relevés datés.
  - Recomptes à la source : **11 AC**, T0-T10 (**11**), **V1-V23** (23 ; +1), **M1-M30** (30), choix **C-29-9-1 à 25**
    (+1). Symptômes grepés : « M22-M27 », « trois gardes », « trois fermetures », « repli-divergent » / `REPLIS_A_SITE_UNIQUE`.
- **2026-10-14 — Validation P6 (passe ciblée, Haiku, prompt `29-9-validate-prompt-p6-ciblee.md`) : 0/0/2/4, remédiée par
  l'orchestrateur ; VALIDATION CLOSE.**
  - F-1 (MEDIUM) : `sprint-status.yaml` disait encore « C-29-9-19 à 24 » et ne citait pas P5 — corrigé. Le bloc
    `## Status` de la fiche est un relevé daté que l'annotation de l'amendement prolonge (C-29-9-19 à 25, P4, P5) :
    laissé tel quel.
  - F-2 (MEDIUM) : l'AC8 déclarait un angle mort pour `more-hidden`/`more-capped` puis affirmait « l'AC11 la tient par
    V19/V20 » — faux, V19/V20 ne visent que les deux clés de l'AC11. Affirmation **retirée** (D5 : sortie par
    retrait), la phrase dit désormais ce que V19/V20 tiennent et pour quelles clés.
  - LOW laissés : F-3/F-4 (renvois de ligne `:146` → `:141`, plage `:216-233` → `:234` ; sans effet sur la
    prescription), F-5 (forme `closes:#1` sans espace, que la commande ne rattrape pas — le message de squash se rédige
    à la main), F-6 (V23 défini en prose à l'AC11 et cité à T10 et M30, absent d'une liste récapitulative).
  - Clôture : la remédiation est un retrait de texte et une ligne de statut, aucune ligne de code ni de prescription
    nouvelle ; P4 → P6 n'ont trouvé que des résidus des remédiations précédentes sur les comptes rendus, aucun défaut
    de conception.

- **2026-10-15 — Développement** (Opus 5.5, autonomie, `bmad-dev-story`). Rebase sur `4b358373` (tête de la 29-8),
  union des registres, en-têtes 163-164. T0-T10 dans l'ordre prescrit (T10 entre T4 et T5) ; dernier commit de code
  `284c1d3b`. Tests rouges d'abord (V1, V12 : verts au sol, absence exigée) ; M1-M30 toutes tuées (+ M15 et M30
  rejouées). Gates au `284c1d3b` : test-fast 3732/3732 (6 ignorés), Vitest 1680/1680 (149 fichiers), check, lint,
  build verts ; E2E 291 passés, 7 KF-029, 19 ignorés. Choix C-29-9-26 à 30. Statut `review` ; prompt de la revue de
  code P1 : `29-9-review-prompt-p1.md`.

- **2026-10-15 — Report sur la tête finale de la 29-8** (Opus 5.5, autonomie). La revue de la 29-8 a ajouté cinq
  commits (`38d5f3fc` … `1261c4bb`, PR #681) ; `git rebase --onto 1261c4bb 4b358373 story/29-9` : 25 commits reportés,
  dernier commit de code `284c1d3b` → **`3e5dfed8`**. Conflits, tous résolus par union :
  - `sprint-status.yaml`, aux sept commits qui touchent la clé de la 29-9 : ligne `29-8` de la base (`done`), lignes
    et en-têtes de la 29-9 de la branche ; contrôlé à chaque arrêt (lignes de la 29-9 identiques au commit reporté,
    reste identique à `1261c4bb`, `yaml.safe_load`). La base n'ajoute aucun en-tête : 163-165 restent en tête, sans
    renumérotation ; aucun doublon d'en-tête neuf (les doublons 105 … 140 sont ceux de la base, inchangés).
  - `docs/api-external.md`, paragraphe *unreject* : l'ajout de la 29-8 (« — et rattachée à un règlement déjà
    enregistré … ») **et** celui de la 29-9 (« 100 par défaut, 500 au plus ; `hasMore` … », lien #649 retiré).
  - `user-manual.tex`, puce *Rejeter* : la parenthèse de la 29-9 (« parmi les plus anciennes s'il le faut … ») **et**
    l'ajout de la 29-8 (« affectée manuellement, éclatée ou rattachée à un règlement déjà enregistré
    (\S\ref{sec:rattacher-reglement}) »). PDF repris de la base pendant le conflit, puis `touch` du `.tex` et `make fr`
    (mem-guard, verrou) : stable en 2 passes, commit `d2c64df9`. PDF aplati : phrases de la 29-9 (« Aﬀicher aussi les
    plus anciennes » ×2, « l’étend à 500 », « rattachées à un règlement », « les 100 plus récentes d’abord », « aucun
    écran ne permet de la rétablir ») et de la 29-8 (« rattachée à un règlement déjà enregistré (§14.3) », « La fenêtre
    rappelle la transaction à sa date de paiement ») présentes ; aucun « ?? ».
  - Sans conflit textuel : `reconciliation.rs`, `messages.ftl` ×4 (`NOT_LINKABLE` de la base, clés de la 29-9) ; fichier
    d'epic non touché par la base (ligne v0.13.4 unique) ; registres : C-29-8-1 à 41 au registre de l'epic (versés par
    la base), C-29-9-1 à 30 au registre propre de la 29-9. `sitesTotal` inchangé (2186), Vitest vert.
  - Contrôle de la résolution : les lignes modifiées de `2157772b..d2c64df9` (PDF exclu) sont celles de
    `4b358373..1261c4bb`, à deux lignes près — les deux sites réunis à la main ci-dessus, où la ligne d'arrivée porte
    les deux ajouts (18 fichiers, +1100/−468 des deux côtés).
  - Tenu sur la nouvelle base : AC11 (région `role="status"` `aria-live="polite"` `aria-atomic="true"`, `aria-label`
    des cases) et l'avis plafonné qui nomme le rattachement (C-29-9-26, ×4 locales).
  - **Gates au HEAD reporté `d2c64df9`** (code = `3e5dfed8`), base `kesh_299` remise à zéro dans le même journal
    (`kesh-gate-logs/299-rebase2-gate.log`) : test-fast **3733/3733** (6 ignorés ; +1 = T17-bis de la 29-8) ; `check`
    0 erreur (29 avertissements, 5 fichiers) ; `lint-i18n-ownership` PASS ; Vitest **1688/1688**, 149 fichiers (+8 =
    tests de `LinkSettlementModal` de la 29-8 : six `it` et un `it.each` à deux cas) ; `build` vert. **E2E complet**
    (`299-rebase2-e2e-complet.log`, base `kesh_e2e_299` remise à zéro, port 3049) : **291 passés, 7 échoués, 19
    ignorés** — les sept KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`,
    `:150`), ni KF-052 ni pollution.
  - **Mutations rejouées** (`299-rebase2-mut.log`) : M30 et M30-bis (i18n) tuées ; M6, M18, M20 (`reconciliation.rs`,
    fusionné automatiquement avec la base) tuées ; arbre restauré (`git checkout` puis `touch`). Aucune mutation ne
    porte sur un fichier touché par la résolution des conflits (documentation et `sprint-status.yaml` seulement).
  - Prompt de la revue de code P1 mis à jour : diff `1261c4bb..3e5dfed8` hors `_bmad-output`, PDF contrôlé au HEAD,
    journaux du report.

- **2026-10-15 — Revue de code P1** (trois lentilles B, E, A, contexte frais ; rapports
  `kesh-gate-logs/299-review-p1-{B,E,A}.md`) : **B 0/0/0/5, E 0/0/0/9, A 0/0/2/5** (CRITICAL/HIGH/MEDIUM/LOW). Aucun
  défaut de code de production ; les deux MEDIUM, de la lentille A, portent sur le compte rendu et le manuel.
  Décision de l'orchestrateur : **cette remédiation ne change aucune ligne de code de production ni de test** (fiche,
  manuel et son PDF seulement).
  - **Corrigé** : A-1 (MEDIUM) la commande de rebase des *Demandes à l'orchestrateur* est
    `git rebase --onto origin/main 1261c4bb8725fec33f6b7920947ed005ee8bd41b story/29-9` (`1261c4bb` a l'arbre de
    `origin/main` `1b136fad`) ; la seule autre occurrence de `4b358373` à la suite de `--onto` est l'entrée *Report*
    ci-dessus, relevé daté et juste à sa date ; A-2 (MEDIUM) le saut de défilement de l'AC2 (C-29-9-15) est déclaré au
    Dev Agent Record, **capture non prise**, écrit comme écart à l'AC2 ; A-5 = L5 le manuel dit de nouveau la
    réserve : la puce *Rejeter* (`user-manual.tex:2101`) et la phrase de rattachement d'une transaction rejetée
    (`:2134`) disent « jusqu'à la 500\textsuperscript{e} transaction en attente », et la puce ajoute qu'au-delà la
    transaction ne reparaît qu'à mesure que les premières sont traitées (cohérent avec `docs/api-external.md:610`,
    relu et inchangé) ; PDF régénéré (`299-p1fix-make-fr.log`, 2 passes, stable), seul `user-manual.pdf` bouge ;
    A-3 (Status et *Debug Log References* connaissent le report et les journaux `299-rebase2-*`) ; A-4 (« 74 → 76 »
    avec `security/`) ; A-6 (M9, M10, M13, M16, M17, M19 ont tourné sur la `class FakePage`, avant `85d8d9ea` ;
    seule M15 a été rejouée sur la doublure définitive ; non rejouées ici) ; A-7 (écart : les sorties des rouges
    sont résumées, journaux nommés).
  - **Laissé, chacun motivé** : B-1/L1, B-2/L3, B-3, B-4, B-5, L2, L4, L6, L7, L8, L9 — LOW qui demandent du code de
    production ou de test, exclu de cette remédiation par décision de l'orchestrateur ; ils restent tels que les
    rapports les décrivent, à reprendre si une passe ultérieure les juge nécessaires.
  - **Contrôles** : PDF aplati en pipe (`pdftotext … | tr '\n' ' ' | tr -s ' '`) : « jusqu’à la 500e transaction en
    attente » présent **2 fois** (puce et phrase de rattachement), « ?? » **0**, « la page le dit » ×2 ; gate ciblé
    `cargo nextest run -E 'binary(textes_coherents)'` (base `kesh_299`, sous verrou et mem-guard) : **15 passés, 0
    ignoré** (`299-p1fix-textes.log`).
  - **Écart** : les lentilles E et A ont écrit un fichier aplati du PDF dans `kesh-gate-logs`, hors dépôt (le prompt
    P1 l'autorisait ; le prompt P2 ne l'autorise plus).
  - Prompt de la passe ciblée P2 (une lentille, sur le seul diff de cette remédiation) :
    `29-9-review-prompt-p2-ciblee.md`.
