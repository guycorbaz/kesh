# Prompt — revue de code P1, Story 29-8 (une transaction réglée depuis la fiche de la facture se rattache à ce règlement)

*Versionné le 2026-10-14. Trois lentilles — B (Blind), E (Edge cases), A (Acceptance) —, contexte frais chacune,
lecture seule stricte. Modèle : **celui qu'indique l'orchestrateur** (passe complète : Sonnet ou Opus, rotation D6 de
la rétrospective 25 ; Haiku réservé à la passe ciblée de fin de boucle).*

Dépôt `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8`, rebasée sur `main` `e889346c` (29-2, 29-3, 29-1a, 29-5,
29-4a et 29-6 mergées). Le code à relire : `git -C /home/gcorbaz/devel/kesh-29-8 diff 4af8f8ca HEAD` (`4af8f8ca` =
validation close de la fiche après le rebase, dernier commit avant le développement ; ce diff ne contient que la
29-8). Langue du rapport : français.

Fiche : `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` (AC1 à AC10, Dev Notes dont la
table *Absence de cycle*, Dev Agent Record, Change Log) ; registre :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-8.md` (C-29-8-1 à 29 de la fiche, 30 et suivants au
développement). Issues #670 (la story), #671 (angles morts L2 à L7), #672 (détacher sans annuler, L1), #401 (frais
de rappel), #651 (clôture concurrente, 29-4). Règles : `CLAUDE.md` (§ Review Iteration Rule et sous-sections : manuel
nommé, axes déclarés, sites non résolus, propagation, recompte ; § *Un appariement automatique propose, il ne crée
jamais* ; § Migration breaking policy — **aucune** migration attendue) ; `docs/agents-orchestration.md` § 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate (dont `298-mutations.py`) ; ni `python3`, ni `perl`, ni `sed -i`, ni
  `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche, ni `--detach`), aucun `git`
  qui écrit (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui
  écrit ; `gh issue view` est permis.
- Aucune redirection de sortie vers un fichier autre que ton rapport (`pdftotext` vers la sortie standard seulement).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -`.

## Axes communs aux trois lentilles

1. **La règle, une seule définition** : `crates/kesh-db/src/repositories/reconciliation_link.rs` —
   `transaction_can_be_linked`, `candidates_sql` (table dérivée par écriture en jointure interne, `HAVING … > 0`,
   `NOT EXISTS`, bornes `0 ≤ écart ≤ tolérance`, `ORDER BY gap, days_apart, s.id`, `LIMIT 50` après filtre et tri),
   l'ordre des **binds** contre les `?`, `rescale(2)` de la sortie. Le `GET` et le `POST` lisent-ils **le même texte**
   (le second restreint à `s.id`) ? La tolérance vient-elle de `AMOUNT_TOLERANCE_HUNDREDTHS` sans copie ? Un
   règlement rattachable que la règle de la fiche refuse, ou l'inverse — en particulier un **faux rattachement** (autre
   compte bancaire, autre société, compte interne, ancien compte du grand livre, écriture déjà rapprochée, écart
   négatif) —, est au moins **HIGH**.
2. **Les verrous et la naissance de la vue** (B5 de la rétrospective 28) : `link_in_tx` — ordre verrou nommé →
   `bank_transactions` → `invoice_settlements` → `journal_entries`, trois `FOR UPDATE` par clé primaire, puis
   `super::bank_accounts::ensure_open_in_tx` comme **première lecture ordinaire**, puis la requête de la règle. Aucune
   lecture verrouillante sur `matched_entry_id`. Relis le doc-comment de `link_in_tx` contre le code et la table
   *Absence de cycle* des Dev Notes (et sa copie au Dev Agent Record) : un cycle de verrous neuf, une lecture ordinaire
   avant le dernier verrou, ou un double rattachement atteignable sont au moins **HIGH**. Les tests T16 et T17 **ne
   prouvent pas** l'ordre d'acquisition (C-29-8-25) : cet axe est **à toi**.
3. **Les routes** : `crates/kesh-api/src/routes/reconciliation.rs` — `link_preflight` (archivé → clos → non lié),
   `get_link_candidates`, `post_link_transaction`, `link_once` ; ordre des refus identique au `GET` et au `POST`
   (AC1, AC2) ; rejeu `retry_app_on_deadlock` ; `conclude_locked_attempt` ; `AppError::ReconciliationSettlementNotLinkable`
   et son repli fr-CH (`errors.rs`) ; routes de `lib.rs` (groupe Comptable, clés). Registres :
   `audit_route_registry.rs` (assertions 119 / 113 / 95 / 122 / 122, prose, point (vi) « quatorze »),
   `bank_account_closed_inventory.rs` (`REGISTRE`, `sites.len()` 32, `FENETRE` et `G` à 11), `audit_labels.rs`,
   `textes_coherents.rs`.
4. **L'écran** : `frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte` (aucune présélection,
   *Rattacher* grisé, avertissement d'écart avant confirmation, liste relue sur `409
   RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, garde de génération, échec du chargement) et
   `ReconciliationProposals.svelte` (`canLink`, `onLinkSuccess` et le compteur, C-29-8-32) ; `reconciliation.api.ts`,
   `reconciliation.types.ts` ; clés `reconciliation-link-*` dans les **quatre** `messages.ftl`, replis égaux au fr-CH.
5. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — sous-section neuve
   `sec:rattacher-reglement`, introduction de la réconciliation, *Algorithme de matching* (conseils de rejet réécrits),
   imputation à score égal, *Accepter ou rejeter*, fin de *Réconciliation manuelle*, *Annuler un rapprochement*, tuile
   d'écart, clôture du compte bancaire, `sec:reglement-client`, avertissement des frais de rappel, FAQ « Que faire
   ensuite », glossaire — **et le PDF** `docs/manual/fr/user-manual.pdf`, aplati et normalisé :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"`.
   Aucune phrase vivante ne doit conseiller de **rejeter** une transaction réglée depuis la fiche (grep `rejet` et
   formes voisines) ; aucune ne doit promettre ce que le code ne tient pas (conditions, écart, exercice clos, rejetée,
   annulation qui retire le règlement, ce qui ne se rattache pas). Même exigence pour `docs/api-external.md` (§
   *Rattacher une transaction…*, table des erreurs, note du compte clos, `kind`, paragraphe des interblocages) et
   `CHANGELOG.md` (`## [0.13.4] — Non publié` : entrée *Ajouté*, entrées des 29-2 et 29-6 réécrites). Une promesse que
   le code ne tient pas est au moins **MEDIUM**.
6. **Propagation** : greper les **valeurs** — `\bcinq\b` près de « chemin », `quatre chemins`, `viennent de créer`,
   `existerait déjà`, `NEUVE`, `rejetez`, `rejetant`, `ne relie pas`, `670`, `RECONCILIATION_SETTLEMENT_NOT_LINKABLE`,
   `reconciliation.linked`, `\b(118|121|112|94)\b` dans `audit_route_registry.rs`, `\b(31|10)\b` dans
   `bank_account_closed_inventory.rs` — sur `crates`, `frontend/src`, `frontend/tests`, `docs/`, `CHANGELOG.md`,
   `website/` : reste-t-il un texte qui décrit l'état d'avant comme courant ?
7. **Le compte rendu** (Dev Agent Record, Change Log, registre) : chaque décompte et chaque chiffre de gate se
   recompte-t-il depuis la source et les journaux (`/home/gcorbaz/devel/kesh-gate-logs/298-*.log`, dont
   `298-mut-*.log` et `298-mutations-result.txt`) ? Les mutations déclarées tuées le sont-elles par les tests nommés ?
   Les survivantes et équivalentes ont-elles une raison vraie ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention : Rust (binds contre `?`, types décodés —
`CAST(… AS SIGNED)`, `DECIMAL` de `? - m.movement` —, `Copy` de `LinkRequest`, erreurs remontées, imports, liens de
doc), SQL (précédence des `AND`, alias dans `ORDER BY`, `GROUP BY` de la table dérivée), Svelte 5 (`$effect` de
réouverture, `$derived`, garde de génération, état `submitting` et `loading`), tests qui prouvent ce qu'ils disent
(une assertion verte sans le code ? un refus asserté par son code **et** l'absence d'écriture ? un témoin qui rougirait
sans le rattachement ? T13 (b) observe-t-il réellement l'attente avant de clore ?), code mort, nommage.

## Lentille E — Edge cases

Parcours chaque branche et chaque borne : écart `0.00`, `0.01`, `0.05`, `0.06`, négatif ; transaction `0.00`,
`-0.00`, débit, EUR ; transaction rejetée ; règlement rattaché puis annulé puis re-rattaché ; deux règlements de même
écart et même jour ; 50 et 51 candidats ; règlement d'une facture créditée, soldée (write-off), annulée ; compte bancaire
archivé, clos, non lié, lié à un compte inactif ou partagé ; lien changé par `PATCH` entre le pré-vol et la
transaction (C-29-8-31) ; exercice clos du règlement ; course rattachement / annulation de règlement / annulation de
rapprochement / suppression d'import / clôture du compte ; corps `{}`, `settlementId` `0`, négatif, chaîne, très
grand ; clé `read` au `GET` et au `POST`. Un cas non couvert par un test et dont le comportement est faux est au moins
**MEDIUM**.

## Lentille A — Acceptance (la fiche contre le livré)

Chaque AC (1 à 10) et chaque tâche (T0 à T8) : livré, testé par un test **qui rougirait**, déclaré honnêtement au Dev
Agent Record ? En particulier : AC1 (forme de la réponse, deux décimales, tri, limite, pré-vol commun, ordre des refus),
AC2 (ordre des refus et des lectures, `UPDATE`, audit et ses clés, registres et leurs chiffres, `FENETRE`), AC3 (T1 et
T15 : clôture débloquée, aucune écriture, `kind`, annulation depuis la fiche refusée par le socle et témoin qui
s'annule, dévalidation en préservation, K2, *Annuler le rapprochement* qui retire le règlement), AC4 et AC5 (écran,
textes, quatre locales), AC6 (inventaire **par symptôme** au Dev Agent Record, chaque site trié ; positifs et négatifs du
PDF), AC7 (les douze sites, chacun réécrit ou déclaré), AC8 (T1 à T20 vus rouges, M1 à M19 chacune avec le test qui la
tue et la sortie du run, survivantes déclarées), AC9 (spec E2E, captures clair et sombre), AC10 (aucune migration,
gates : chiffres recomptés, E2E au dernier commit de code jugé fichier par fichier contre `docs/testing.md` § *Les
échecs attendus*). Les écarts à la fiche déclarés au Dev Agent Record et au registre (C-29-8-30 et suivants) sont-ils
justifiés et sans effet sur un AC ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/298-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne,
  preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas ;
- rien d'autre n'est écrit nulle part.
