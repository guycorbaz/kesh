# Prompt — revue de code P1, Story 29-5 (à score égal, la facture échue la première passe en tête, art. 87 CO)

*Versionné le 2026-10-14. Trois lentilles — B (Blind), E (Edge cases), A (Acceptance) —, contexte frais chacune,
lecture seule stricte. Modèle : **Sonnet** pour les trois (passe complète ; rotation D6 de la rétrospective 25 :
Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée de fin de boucle).*

Dépôt `/home/gcorbaz/devel/kesh-29-5`, branche `story/29-5` (rebasée sur la tête de `story/29-3`, elle-même sur
`main` `c36f39bc`, la 29-2 mergée). Le code à relire : `git -C /home/gcorbaz/devel/kesh-29-5 diff 7983d76b HEAD`
(`7983d76b` = validation close de la fiche, dernier commit avant le développement). Langue du rapport : français.

Fiche : `_bmad-output/implementation-artifacts/29-5-departage-exigibilite.md` (AC1 à AC9, Dev Agent Record) ;
registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-5.md` (C-29-5-1 à 12 de la fiche, les
suivants neufs au développement). Issue #668 et son commentaire de rectification (`gh issue view 668 --comments`) ;
voisines #529 (29-2, ancre de la fenêtre), #527 (29-3, rang des candidates), #548 (28-7, C-28-7-10). Règles :
`CLAUDE.md` (§ Review Iteration Rule et sous-sections : manuel nommé, axes déclarés, sites non résolus,
propagation, recompte ; § Migration breaking policy — aucune migration attendue ; § Un appariement automatique
propose, il ne crée jamais) ; `docs/agents-orchestration.md` § 6.

**Art. 87 CO** (RS 220), en substance, tel que la fiche le retient (Fedlex non relu à la source) : faute de
déclaration du débiteur (art. 86), le paiement s'impute sur la dette exigible ; entre plusieurs, sur celle qui a fait
l'objet des premières poursuites, à défaut sur la dette échue la première (al. 1) ; entre dettes échues en même temps,
proportionnellement (al. 2) ; si aucune n'est échue, sur celle qui présente le moins de garanties (al. 3).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate ; ni `python3`, ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui écrit ;
  `gh issue view` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext` vers la
  sortie standard uniquement.

## Axes communs aux trois lentilles

1. **La règle** : `invoice_exigibility_date`, `imputation_order`, `invoice_exigibility_date_sql!` /
   `INVOICE_EXIGIBILITY_DATE_SQL`, l'`ORDER BY` de `find_unpaid_invoices_for_window` et la macro d'ancre
   `invoice_payment_anchor_sql!` qui compose désormais la macro d'exigibilité
   (`crates/kesh-db/src/repositories/reconciliation.rs`) ; le tri de `propose_matches`
   (`crates/kesh-reconciliation/src/matching.rs`). Les deux écritures (Rust, SQL) disent-elles la même chose sur
   **toute** entrée (échéance nulle, antérieure, égale, postérieure) ? L'ancre de la 29-2 est-elle **inchangée** ?
   Le score reste-t-il la première clé ? L'ordre est-il total ? Une voie atteignable par laquelle, à score égal, une
   facture non échue passerait devant une facture échue du même client, ou par laquelle la troncature écarterait,
   parmi des candidates de même score, celle que `propose_matches` placerait en tête, est au moins **HIGH**.
2. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — § *Algorithme de matching*, le
   paragraphe neuf `\label{sec:imputation-score-egal}` et le suivant (limites : désignation à montant égal, « peut
   passer devant » hors même client, poursuites, imputation proportionnelle, clients différents), la phrase de
   § *Accepter ou rejeter les propositions* (« à score égal, celle qu'impute la règle de la loi ») — **et le PDF**
   `docs/manual/fr/user-manual.pdf`, aplati, ligatures normalisées :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`
   (« échue la première » ≥ 1, « art. 87 » ≥ 1, renvoi résolu, aucun « ?? »). Même exigence pour
   `docs/api-external.md` (§ *Accepter des propositions de rapprochement*, la phrase de l'ordre de `candidates` : seules
   les deux parties que la 29-5 rend fausses devaient changer) et `CHANGELOG.md` (`## [0.13.4] — Non publié` : l'entrée
   #668 sous « Corrigé », l'entrée #527 sous « Modifié » rectifiée). Une promesse que le code ne tient pas est au
   moins **MEDIUM** ; une affirmation juridique plus forte que ce que la fiche établit (art. 75, 86, 87) aussi.
3. **Propagation** : greper les **valeurs** de l'ancien départage (`invoice_id ASC`, `identifiant croissant`,
   `tie-break`, `i.date ASC, i.id ASC`, `date ASC, id ASC`, `(date, id)`, `plus récentes`, `C-29-3-12`,
   `il ne change pas`, `celui des versions précédentes`, `passe devant`) sur `crates`, `frontend/src`, `docs/`,
   `CHANGELOG.md` : reste-t-il une phrase qui décrit l'ancien ordre comme l'ordre courant ?
4. **Le compte rendu** (Dev Agent Record, Change Log, registre) : chaque décompte et chaque chiffre de gate se
   recompte-t-il depuis la source et les journaux (`/home/gcorbaz/devel/kesh-gate-logs/295-*.log`, dont
   `295-mutations.log`) ? Les mutations déclarées tuées le sont-elles par les tests nommés ? M15, déclarée
   équivalente ou tuée, l'est-elle pour la raison écrite ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention : Rust (la paire `(&Invoice, MatchProposal)` du tri —
emprunts, coût, `sort_by` stable —, `partial_cmp` des scores flottants et `unwrap_or(Equal)`, visibilité des
fonctions, liens de doc `[...]` résolubles, dépendance `kesh-reconciliation` → `kesh-db::repositories`), SQL (le
`CAST`, `GREATEST`/`COALESCE` et les `NULL`, l'expression dans l'`ORDER BY` d'une requête à `HAVING`, la composition
`concat!` de l'ancre), tests qui prouvent ce qu'ils disent (une assertion qui passerait aussi sans le code ? montages
`assert_ne!` réellement discriminants ? attendus calculés sans la fonction testée ?), code mort, nommage.

## Lentille E — Edge cases

Parcours chaque branche et chaque borne : échéance nulle, antérieure à la date, égale, lointaine (2099) ; facture
sans échéance datée après le paiement (`F = D + 1`) ; égalités de `X` à `F` différentes, égalités `(X, F)` ;
exactement 50 et 51 candidates ; scores égaux par des voies différentes (0,40 + 0,10 contre 0,50 ; flottants
`0.5 + 0.1`) ; clients différents à score égal ; facture désignée par la référence mais à quelques centimes ;
référence qui contient deux numéros ; l'acceptation (`accept_one_invoice`, qui rescore une facture : l'ordre y
est-il réellement sans objet ?) ; `order_candidates` de la 29-3 (stable : laisse-t-il passer l'ordre intact ?). Un cas
non couvert par un test et dont le comportement est faux est au moins **MEDIUM**.

## Lentille A — Acceptance (la fiche contre le livré)

Chaque AC (1 à 9) et chaque tâche (T0 à T6) : livré, testé par un test **qui rougirait**, déclaré honnêtement au
Dev Agent Record ? En particulier : AC1 (macro **avant** la macro d'ancre, littéral exact, `const`, doc-comments :
art. 87 en substance, art. 75 et convention C-29-5-12, équivalence « échues d'abord ⇔ `X` croissante » et sa
condition, deux écritures et leur test, ce que la règle ne fait pas), AC2 (signature et forme JSON inchangées,
commentaire `:97-98` réécrit), AC3 (`ORDER BY` exact, doc-comment : la victime de la troncature change), AC4 (unitaires,
trois tests d'intégration, attendus en dur, `assert_ne!` de montage), AC5 (cas 1 à 9, ordre d'entrée « l'écartée en
premier », sauf cas 7), AC6 (rouge au sol constaté), AC7 (texte du manuel, inventaire clos, PDF, API, CHANGELOG),
AC8 (M1 à M15, chacune avec le test qui la tue et la sortie du run), AC9 (gates : chiffres recomptés, E2E jugé
fichier par fichier). Les écarts à la fiche déclarés au Dev Agent Record et au registre sont-ils justifiés et sans
effet sur un AC ? Le rebase sur la 29-2 et la 29-3 (T0) a-t-il laissé une zone partagée mal reprise
(`repositories/reconciliation.rs`, `routes/reconciliation.rs` doc-comment d'`order_candidates`, manuel
`:2027-2060`, `docs/api-external.md:549`, `CHANGELOG.md` § 0.13.4) ? Les `Vec` ordonnés des tests de la 29-2
(inventaire (c) de la fiche) changent-ils vraiment de sens, ou non, comme le Dev Agent Record l'affirme ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-5-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne,
  preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas ;
- rien d'autre n'est écrit nulle part.
