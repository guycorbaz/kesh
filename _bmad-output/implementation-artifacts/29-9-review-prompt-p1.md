# Prompt — revue de code P1, Story 29-9 (quatre petits correctifs : #649, #655, #646, #678)

*Versionné le 2026-10-15. Trois lentilles (B, E, A), contexte frais chacune, lecture seule. Modèle : celui que désigne
l'orchestrateur (passe complète : Sonnet ou Opus, jamais Haiku — D6 de la rétrospective de l'Epic 25).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9`.
**Le diff à revoir** : `git diff 1261c4bb8725fec33f6b7920947ed005ee8bd41b 3e5dfed8 -- . ':(exclude)_bmad-output'` —
`1261c4bb` = tête **finale** de `story/29-8` (revue close, PR #681 ; elle contient `main` `12c64c39`), sur laquelle la
29-9 est reportée le 2026-10-15 (elle l'était auparavant sur `4b358373`, tête de la 29-8 avant sa revue) ;
`3e5dfed8` = **dernier commit de code** (report de `284c1d3b`). Les commits qui le suivent ne portent que la fiche, le
registre, `sprint-status.yaml`, ce prompt et le **PDF du manuel régénéré après le report** (`d2c64df9`) : le PDF se
contrôle donc **au HEAD**, pas à `3e5dfed8` (où il est encore celui de la 29-8, repris pendant le conflit). ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`. Ce qui vient de la 29-8 (bouton *Rattacher à un règlement*, `LinkSettlementModal`,
routes de rattachement, table des refus de `docs/api-external.md`) **n'est pas** l'objet de la revue — sauf aux deux
sites où le report a réuni les deux côtés à la main (paragraphe *unreject* de `docs/api-external.md`, puce *Rejeter*
du manuel) : vérifie que chacun porte **et** l'ajout de la 29-8 (« rattachée à un règlement ») **et** celui de la 29-9.
Fiche : `_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md` — AC1 à AC11, tâches T0-T10, V1-V23,
M1-M30, Dev Notes (*Zones partagées*, *Propagation*, *Ce qui doit être préservé*), et le **Dev Agent Record** (prémisses
relues après rebase, tests rouges, mutations, gates, écarts). Choix C-29-9-26 à C-29-9-30 au registre
`epic-29-choix-autonomes-29-9.md` (nés du développement ; C-29-9-1 à 25 sont ceux de la fiche). Règles : `CLAUDE.md`
du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une passe doit
NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § *Migration breaking policy*
(aucune migration attendue : à vérifier). Issues : #649, #655, #646, #678 — la PR les fermera par son corps ; **aucun
mot-clé de fermeture dans un message de commit**, même en prose : à greper avec la commande de la fiche (l. ~25-27),
`git log 1261c4bb..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+(([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)?#[0-9]+|https://github\.com/\S+/issues/[0-9]+)"`.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/`. Les captures `kesh-gate-logs/299-captures/*.png` sont synthétiques : tu peux
les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **#649 — l'écran** (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`). `limit` et `hasMore`
   en état ; `load()` lit `limit` **avant** son premier `await` (l'`$effect` le suit) ; le bouton ne fait **que** poser
   `limit` (une requête, pas deux) ; `hasMore` posé sur une réponse **non périmée** seulement (garde `loadGen`) ; l'avis
   et le bouton **immédiatement** après `</table>`, avant les trois modales (dont `LinkSettlementModal` de la 29-8) ;
   `disabled={busy}` ; l'avis plafonné sans bouton et sans « rejet ». Cherche l'état faux : `hasMore` vrai resté d'une
   réponse ancienne ; une réponse en erreur qui laisse un avis ; un lot (*Accepter*, *Rejeter*), une affectation, un
   éclatement **ou un rattachement** (29-8) qui relirait à 100 ; le `{#key selectedId}` de `routes/(app)/reconciliation/
   +page.svelte` qui remet `limit` à 100 au changement de compte (voulu, écrit au site).
2. **#649 — le contrat** : constantes `PROPOSALS_DEFAULT_LIMIT`/`PROPOSALS_MAX_LIMIT` (`reconciliation.api.ts`) contre
   `DEFAULT_PROPOSALS_LIMIT`/`MAX_PROPOSALS_LIMIT` (`crates/kesh-api/src/routes/reconciliation.rs`) ; **aucun code de
   production Rust ne change** (seuls des doc-comments — vérifie-le au diff, `kesh-api` **et** `kesh-db`) ; les deux
   tests Rust neufs de `reconciliation_e2e.rs` et leur aide `seed_pending_and_get_proposals` (frontière 500 exacte,
   501 avec `?limit=1000`) ; le mock `importOriginal` de `ReconciliationProposals.test.ts`.
3. **#655 — les deux pages** (`routes/(app)/bank-import/[id]/+page.svelte`, `routes/(app)/bank-import/profiles/[id]/
   +page.svelte`). Détail : `mySeq` en **première** instruction, gardes dans le `try` et le `catch`, `loading` non gardé
   (C-29-9-14) — est-ce encore vrai au code (aucun `loading = true` ailleurs) ? Profil : `$derived` de la **chaîne**,
   l'effet ne lit plus `page.params` ; `mySeq` avant la branche « ID invalide » ; remise à zéro et `isLoading = true` ;
   gardes du `.then` et du `.catch` ; garde d'`onUpdated` sur `Number(rawId)` (C-29-9-17). Cherche une boucle d'effet
   (l'effet lit-il un état qu'il écrit ?), une saisie perdue à id égal, un « ID invalide » écrasé, une sauvegarde tardive
   qui écrirait le mauvais profil, et ce que fait `onUpdated` quand `rawId` est invalide (`NaN`).
4. **#646 — la confirmation** (`features/contacts/ContactPersonsManager.svelte`) : refus ⇒ aucun appel, liste et
   message d'erreur inchangés ; nom dans la question ; bouton, classe et `data-testid` inchangés
   (`boutons-rouges-inventory.test.ts`, motif réécrit).
5. **#678 — accessibilité.** `aria-label` de la case : deux appels `i18nMsg` à clé **littérale** sous un ternaire sur la
   **véracité** de `counterpartyName` (`''` et `null` → la même clé) ; valeurs telles que la ligne les affiche. Région
   `role="status" aria-live="polite" aria-atomic="true"` : rendue dès la liste non vide, **dernier enfant** de la rangée
   des boutons, englobant le `{#if}` et le `<p>` de la 29-6 (son `data-testid`, sa clé, son repli **inchangés**, à
   l'indentation près — vérifie-le au diff) ; jamais dans un `{#key}`. Le test de position de la 29-6 amendé en delta
   (V22) garde-t-il son intention (C-29-6-18) et son volet « repli = catalogue » ? Un lecteur d'écran annoncerait-il
   vraiment le changement (nœud stable d'une sélection à l'autre) ?
6. **Les tests prouvent-ils ce qu'ils nomment.** V1-V23 contre la fiche ; les fixtures (`profile-page-state.svelte.ts`,
   objet à accesseurs et non classe — C-29-9-27 : la réassignation de `params` est-elle bien celle de SvelteKit ?) ; la
   campagne M1-M30 (journaux `kesh-gate-logs/299-mut-*.log`, motifs et scripts dans `kesh-gate-logs/299-scripts/`) —
   cherche la **mutation plausible qui resterait verte** (exemples : `hasMore` posé hors de la garde `loadGen` ; le
   bouton rendu pendant `loading` ; la garde du détail placée **après** `errorMessage = null` ; `isLoading = false`
   oublié sur la branche « ID invalide » ; `confirm` appelé avec un autre nom ; `aria-label` sur la ligne au lieu de la
   case). Un test qui passerait à vide (sélecteur absent → `toBeNull()` vrai par défaut ; `setTimeout(0)` trop court
   pour laisser la réponse ancienne arriver ?).
7. ⛔ **LA DOCUMENTATION — MANUEL UTILISATEUR (PDF compris), `docs/api-external.md`, CHANGELOG.** Chaque phrase neuve
   ou modifiée dit-elle vrai **au code** ? Au manuel utilisateur (`docs/manual/fr/user-manual.tex`) : § *Accepter ou
   rejeter les propositions* — « les 100 plus récentes d'abord », le paragraphe neuf (avis, bouton jusqu'à 500, la
   sélection vidée, l'avis au-delà de 500 et les gestes qu'il nomme, dont le rattachement — C-29-9-26), la parenthèse
   réécrite d'*Annuler le rejet* (le reste du paragraphe, de la 29-6 et de la 29-8, doit être **inchangé**) ; la
   `keshnote` des personnes de contact (*Supprimer* demande confirmation, aucun écran ne rétablit). Cherche une phrase
   des manuels, du README ou de `website/` que la story rend **fausse** et n'a pas touchée (`grep -n "100" docs/manual/
   fr/user-manual.tex`, `grep -rn "100 plus récentes\|100 most recent" docs README.md website`). La phrase de la 29-6
   « la page le dit » reste-t-elle vraie (C-29-9-22) ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/29-9-review-p1-<lettre>-user-manual-flat.txt` — ⚠️ `pdftotext` rend « ﬀ »
   (ligature : « Aﬀicher ») et « ’ » (apostrophe typographique) : un `grep` sur « Afficher » ou « l'avis » rend un faux
   négatif. Le PDF est-il celui du HEAD (régénéré en `d2c64df9`, après la dernière modification du `.tex`, `95939a71` — report de `52e9318c` — et la
   réunion de la puce *Rejeter* au report) ? Seul
   `user-manual.pdf` est versé (C-29-9-29) : les deux autres PDF n'ont-ils vraiment rien à refléter ?
   `docs/api-external.md` (paragraphe *unreject*, « 100 par défaut, 500 au plus ; `hasMore` ») ; CHANGELOG § `[0.13.4]`,
   *Corrigé* : quatre puces (#649, #655, #646, #678) ; `[0.13.3]` **intact**.
8. **i18n, gardes et décomptes.** Six clés neuves et une réécrite, ×4 locales (`grep -c "^reconciliation-labels-more-\|
   ^reconciliation-actions-show-older\|^reconciliation-labels-select-transaction\|^contact-persons-confirm-delete"
   crates/kesh-i18n/locales/*/messages.ftl`), traduites réellement, mêmes variables, replis égaux à fr-CH au caractère
   près (apostrophe droite de « jusqu'à », « d'abord ») ; `KNOWN_VIOLATIONS` (+1, `lint-i18n-ownership.js`) ;
   `sitesTotal` 2180 → 2186 (`i18n-keys.test.ts`) ; le test V23 (`i18n-entrees-a-variables.test.ts`). **Recompte depuis
   la source** tout décompte du Dev Agent Record (tests neufs par fichier aux **deux** bornes `1261c4bb` / `3e5dfed8`,
   clés, mutations tuées, chiffres des gates) et vérifie le **périmètre** de chaque nombre.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les courses de chaque correctif (deux
  relectures, deux navigations, navigation pendant une sauvegarde, clic sur le bouton pendant un `load()` déjà en vol,
  changement de compte pendant le rechargement à 500) ; Svelte 5 (`$derived` d'une chaîne, `$state.raw`, effets qui
  écrivent ce qu'ils ne lisent pas, props lues en direct) ; `window.confirm` absent ou bloqué ; le rendu (la région vide
  reçoit le `gap-2` de la rangée flex : écart visuel ? captures `299-captures/29-6-*` contre `298-captures/29-6-*`).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Exactement 100, 101, 500, 501
  transactions ; `hasMore` vrai avec une liste vide ; contrepartie `null`, `''`, espaces seuls ; identifiant `'0'`,
  `'-1'`, `'5abc'` pour le profil ; rejoue mentalement la campagne et cherche la mutation verte (axe 6) ; l'E2E
  (`reconciliation-show-older.spec.ts` : 501 créées vérifiées, la 150e, le nom accessible lu dans les cellules ;
  `contact-persons-delete.spec.ts` : `page.once('dialog')`, nettoyage).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1 à AC11) contre le code :
  satisfait, partiel, absent ; V1-V23 et M1-M30 présents et conformes ; les écarts à la fiche sont-ils tous déclarés
  au Dev Agent Record (C-29-9-26 à 30, fixture, V11, PDF, captures « sombre » claires — C-28-1-7) ; les chiffres de
  gates et d'E2E déclarés correspondent-ils aux journaux du report :
  `kesh-gate-logs/299-rebase2-gate.log`, `299-rebase2-npm-*.log` et `299-rebase2-e2e-complet.log` (au HEAD
  `d2c64df9`, dont le code est celui de `3e5dfed8`) — ceux de `284c1d3b` (`299-gate-final2.log`,
  `299-e2e-complet.log`) sont des relevés d'avant le report ; les échecs E2E jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus* ; `sprint-status.yaml` : clé `29-9-trois-petits-correctifs` en `review`,
  commentaire daté.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-9-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag` — **aucun `git
checkout` ni `git switch` sous aucune forme**, pas même `--detach` —, `sqlx`, `cargo` (aucune sous-commande), `npm`,
`npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même les scripts de mutation), `perl`, `ruby`, `php`, `sed -i`,
`awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget`, `gh` en écriture,
`pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. ⛔ Ne touche pas au worktree
`/home/gcorbaz/devel/kesh-29-8`. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `cmp`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`, l'ouverture des PNG de
`kesh-gate-logs/299-captures/` et `kesh-gate-logs/298-captures/`.
