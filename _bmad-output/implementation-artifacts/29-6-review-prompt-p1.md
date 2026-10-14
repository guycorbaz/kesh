# Prompt — revue de code P1, Story 29-6 (la réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser)

*Versionné le 2026-10-14. Trois lentilles — B (Blind), E (Edge cases), A (Acceptance) —, contexte frais chacune,
lecture seule stricte. Modèle : **Sonnet** pour les trois (passe complète ; rotation D6 de la rétrospective 25 :
Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée de fin de boucle).*

Dépôt `/home/gcorbaz/devel/kesh-29-6`, branche `story/29-6`, rebasée sur `main` `81b1b526` (29-2, 29-3, 29-1a et
29-5 mergées). Le code à relire : `git -C /home/gcorbaz/devel/kesh-29-6 diff 6f132a98 HEAD` (`6f132a98` = validation
close de la fiche après le rebase, dernier commit avant le développement ; ce diff ne contient que la 29-6). Langue du
rapport : français.

Fiche : `_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md` (AC1 à AC10, Dev Agent Record, Change
Log) ; registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-6.md` (C-29-6-1 à 19 de la fiche,
les suivants neufs au développement). Issues #665 (la story), #670 (angle mort assumé : une transaction rejetée reste
en attente — Story 29-8), #476 (`classify_payment` à double borne), #548 (28-7, une seule fenêtre de dates). Règles :
`CLAUDE.md` (§ Review Iteration Rule et sous-sections : manuel nommé, axes déclarés, sites non résolus, propagation,
recompte ; § Pattern batch ; § Migration breaking policy — aucune migration attendue) ; `docs/agents-orchestration.md`
§ 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate ; ni `python3`, ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui écrit ;
  `gh issue view` est permis.
- Aucune redirection de sortie vers un fichier autre que ton rapport (`pdftotext` vers la sortie standard seulement).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -`.

## Axes communs aux trois lentilles

1. **La règle** : `HALF_CENTIME`, `least_due_without_overpayment` et sa démonstration
   (`crates/kesh-db/src/repositories/invoice_settlements.rs`), le `HAVING` et ses binds de
   `find_unpaid_invoices_for_window` (`crates/kesh-db/src/repositories/reconciliation.rs`). La borne basse est-elle
   **exactement** la réciproque de `classify_payment` sur toute entrée atteignable (montant au centime, positif ; reste
   à quatre décimales, nul, négatif) ? Le filtre est-il **dans la requête**, avant `ORDER BY … LIMIT 50` ? Le côté du
   paiement partiel (reste supérieur de 5 centimes au plus) est-il inchangé ? Une voie atteignable par laquelle la
   proposition offrirait encore une facture que l'acceptation refuse en `RECONCILIATION_OVERPAYMENT`, ou cesserait de
   proposer une facture qu'elle accepte en solde exact ou avec écart d'arrondi, est au moins **HIGH**.
2. **L'écran** : `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` — case sur toute ligne,
   `selectionHasUnmatched`, `disabled` d'*Accepter* (et **pas** de *Rejeter*), garde de tête d'`onAccept`, indication
   dans la rangée des boutons (C-29-6-18), ordre rangée → phrase de priorité de la 29-3 → table. Une voie par laquelle
   *Accepter* enverrait une ligne sans proposition, ou *Rejeter* serait grisé par elle, est au moins **HIGH**.
3. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — § *Algorithme de matching* (condition de
   montant, contournement du trop-perçu, phrase de la 29-2 sur le rejet réécrite), § *Accepter ou rejeter les
   propositions* (ligne « Aucune correspondance », paragraphe *Rejeter*), FAQ `sec:faq-aucun-match` (première puce,
   « Que faire ensuite ») — **et le PDF** `docs/manual/fr/user-manual.pdf`, aplati et normalisé :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"`.
   Aucune phrase ne doit dire que le rejet **règle** la transaction ou la sort de l'attente (#670) ; aucune ne doit
   pousser un paiement de client vers *Affecter manuellement* ou *Éclater* (C-29-6-16). Même exigence pour
   `docs/api-external.md` (§ *Accepter des propositions de rapprochement*) et `CHANGELOG.md` (`## [0.13.4] — Non
   publié`, entrée #665 sous « Corrigé »). Une promesse que le code ne tient pas est au moins **MEDIUM**.
4. **Propagation** : greper les **valeurs** — `à 5 centimes près`, `± 0.05`, `amount_tolerance`, `au plus 0.005`,
   `665`, `pas de case`, `sélectionnable`, `écarte la proposition`, `sans quoi`, `ne peut qu` — sur `crates`,
   `frontend/src`, `frontend/tests`, `docs/`, `CHANGELOG.md`, `website/` : reste-t-il une phrase qui décrit l'ancien
   comportement comme courant ?
5. **Le compte rendu** (Dev Agent Record, Change Log, registre) : chaque décompte et chaque chiffre de gate se
   recompte-t-il depuis la source et les journaux (`/home/gcorbaz/devel/kesh-gate-logs/29-6-*.log`, dont
   `29-6-mut-*.log`) ? Les mutations déclarées tuées le sont-elles par les tests nommés ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention : Rust (`Decimal::from_parts` et son échelle, ordre des
binds contre les `?` du SQL, liens de doc `[...]` résolubles, imports), Svelte 5 (`$derived` sur `proposals` et
`selected`, réactivité du `Set` réassigné, `flex-wrap` de la rangée, accessibilité d'une case sans libellé), tests
qui prouvent ce qu'ils disent (une assertion qui passerait aussi sans le code ? un état grisé prouvé par l'attribut et
non par l'absence d'appel après `fireEvent.click` ? attendus calculés sans la fonction testée ? le spec E2E
cible-t-il par `data-testid` et nettoie-t-il en `finally` ?), code mort, nommage.

## Lentille E — Edge cases

Parcours chaque branche et chaque borne : restes `P − 0.0051`, `P − 0.0050`, `P − 0.0001`, `P`, `P + 0.0049`,
`P + 0.05`, `P + 0.0500` (échelle), `P + 0.06` ; `P = 0.01` ; reste nul ou négatif ; transaction débitrice ou non CHF
(aucune facture cherchée : ligne sans proposition) ; ligne à **règle** seule (avec proposition) ; exactement 50 et
51 candidates dont des trop-perçus ; facture modifiée entre la proposition et l'acceptation ; sélection qui mêle
lignes avec et sans proposition, décochage, rechargement après un lot, erreur de `rejectProposals`, `busy` ; compte
clos ou archivé au rejet (pré-vol de `…/reject`) ; la transaction rejetée puis *Annuler le rejet*. Un cas non couvert
par un test et dont le comportement est faux est au moins **MEDIUM**.

## Lentille A — Acceptance (la fiche contre le livré)

Chaque AC (1 à 10) et chaque tâche (T0 à T6, T3bis) : livré, testé par un test **qui rougirait**, déclaré honnêtement
au Dev Agent Record ? En particulier : AC1 (forme exacte de `HALF_CENTIME`, doc-comments : démonstration,
précondition, lecteur, ligne « Réciproque » de `classify_payment`), AC2 (paramètre renommé, binds dans l'ordre, filtre
4 et en-tête du module réécrits, phrase fausse « au plus 0.005 » disparue, `WHERE`/`ORDER BY`/`LIMIT 50` inchangés,
doc-comment d'`AMOUNT_TOLERANCE_HUNDREDTHS`, commentaire de l'étape (c), doc-comment d'`order_candidates`), AC3
(`git diff` d'`accept_one_invoice` : commentaires seulement), AC4 (grille de 8 × 1401, trois assertions en dur), AC5
(deux tests, attendus en dur, `LIMIT 50` lu au SQL), AC6 (cas a à h, `accept_couples_296` neuf, `proposed ⊆
accepted`), AC7 (textes, PDF contrôlé positivement puis négativement, API, CHANGELOG), AC8 (M1 à M13, chacune avec le
test qui la tue et la sortie du run), AC9 (gates : chiffres recomptés, E2E jugé fichier par fichier contre
`docs/testing.md` § *Les échecs attendus*), AC10 (case, *Accepter* grisé et gardé, *Rejeter* actif, indication, clé
en quatre locales mot pour mot, `sitesTotal` recompté, en-tête du composant, commentaire de
`reconciliation-deleted-import.spec.ts`, spec E2E et captures clair et sombre décrites). Les écarts à la fiche
déclarés au Dev Agent Record et au registre sont-ils justifiés et sans effet sur un AC ? Les zones partagées avec la
29-2, la 29-3 et la 29-5 (manuel `§ Algorithme de matching`, composant, `i18n-keys.test.ts`, CHANGELOG § 0.13.4)
ont-elles été reprises sans écraser leur texte ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-6-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne,
  preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas ;
- rien d'autre n'est écrit nulle part.
