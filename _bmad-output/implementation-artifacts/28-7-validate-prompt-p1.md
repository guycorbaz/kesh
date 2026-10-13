# Prompt — validation P1 de la spec, Story 28-7 (une facture postérieure au paiement n'est plus proposée)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-7`, branche `story/28-7` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-7-facture-posterieure-paiement.md` (commit `8c3be468`).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-7* et § *Ordre*. Issues #548 (l'objet) et
#529 (voisine, **non absorbée**) : `gh api repos/guycorbaz/kesh/issues/548`, `…/529`.
Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-7-1 à 9) ; antécédent :
`epic-15-choix-autonomes.md` C-15-5c-3.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch) ; `docs/agents-orchestration.md`.

⚠️ La 28-5 et la 28-6 touchent le même `routes/reconciliation.rs` et seront mergées avant le développement : les
numéros de ligne de la fiche sont relevés sur `5aad8b35`, ce qui est le bon état pour cette passe.

## Axes prioritaires (communs aux deux lentilles)

1. **Périmètre minimal** (leçon de l'Epic 28, impérative) : la fiche ferme-t-elle #548 **dès elle-même**, sans
   rien de plus ? Chaque AC et chaque tâche est-il nécessaire à #548 ou à la non-régression ? Les exclusions sont-
   elles écrites (#529, acompte, règlement depuis la fiche, `LIMIT 50`, captures, Playwright) et chacune
   **justifiée** ? Les modules sont-ils bien comptés (fiche § *Modules touchés* : cinq) — recompte-les depuis les
   AC. Un ajout non nécessaire à #548 est au moins **MEDIUM** ; une exclusion qui laisse #548 ouvert, **HIGH**.
2. **Inventaire clos des sites** (Dev Notes § *Inventaire (a)* et *(b)*) : refais les `grep` copiés et
   **élargis-les** — un détecteur part du symptôme, pas d'une liste : toute requête qui filtre `invoices` par
   date en présence d'une transaction (`i.date`, `invoices.date`, `QueryBuilder`, `format!`), tout code qui
   compare `invoice.date` à une date de transaction (`value_date`, `booking_date`, `paid_at`, `settled_on`),
   `kesh-reconciliation`, `kesh-seed`, `test_endpoints.rs`, le frontend (filtre côté client des candidates ?).
   Un site qui choisit ou valide une facture pour une transaction et que la fiche ignore est au moins **HIGH**.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Algorithme de matching*
   (l. 1917-1921, dont le paragraphe `\label{sec:facture-posterieure}`), FAQ l. 2787, rappels l. 1299. Ce que le
   manuel dit **aujourd'hui** est-il vrai au code de `5aad8b35` ? Ce que l'AC8 prescrit d'écrire sera-t-il vrai au
   code que la fiche prescrit ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - |
   tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas est au moins MEDIUM.
4. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — appels de
   `find_unpaid_invoices_for_window` (« dix autres appels de tests »), `#[sqlx::test]` de `reconciliation_e2e.rs`
   (81), AC (11), tâches (T0-T8), mutations (M1-M10), choix (9), modules (5), numéros de ligne cités.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #548 et du § *28-7* de l'epic (« une seule borne partagée […] même date de référence », « tests
  aux bornes J−1, J, J+1, `value_date` absente », « `kesh-db` touché → gate complet ») couverte par un AC
  testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** ; chaque fait, chemin et numéro de
  ligne **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (quatre `.ftl`, repli de
  `failed-proposal-label.ts`, manuel, CHANGELOG) confrontés à ce que le code prescrit fera ; les gardes
  existantes qui pourraient changer de sens (`i18n-keys.test.ts` et ses compteurs, `lint-i18n-ownership`,
  `textes_coherents.rs`, `test_schema_guard.rs`, les tests `post_accept_rejects_payment_date_*`) — la fiche les
  nomme-t-elle ? Les tests existants que la nouvelle fenêtre ferait rougir sont-ils tous repérés (§ *Inventaire
  (c)*) — cherche un `get_proposals_*` ou une spec E2E dont la facture est datée après `D + 1` ou avant `D − 30`.
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : la règle `D − 30 ≤ F ≤ D + 1`
  est-elle **exactement** celle de l'acceptation actuelle (`routes/reconciliation.rs:1656-1683`) — refais
  l'algèbre, bornes incluses ou exclues ? La démonstration « #529 non aggravé » (fiche § *Effet sur #529*) est-
  elle juste, y compris quand `value_date` précède ou suit `booking_date` ? Les tableaux de l'AC4 et de l'AC5
  sont-ils **exacts** cas par cas (recalcule chaque ligne) et l'AC4 est-il **réalisable** (sept couples dans un
  seul `POST /accept`, exercice couvrant les dates, unicité des montants sous la tolérance de 0.05, verrou de
  compte, ordre de traitement) ? La nouvelle signature de `find_unpaid_invoices_for_window` et le `BETWEEN ? AND
  ?` changent-ils l'usage de l'index `idx_invoices_company_validated_unpaid_date` ? Le clamp de C-28-7-4 est-il
  nécessaire, et correct (comportement de sqlx et de MariaDB pour `NaiveDate` hors `[1000-01-01, 9999-12-31]`,
  ancien comportement de `DATE_ADD` à `9999-12-31`) ? Les mutations M1-M10 sont-elles toutes **tuables** par les
  tests nommés, et en manque-t-il une évidente (ex. `for_transaction` qui ignore `value_date`, appel de `check`
  supprimé dans `accept_one_invoice`) ? Le libellé raccourci (C-28-7-5) reste-t-il exact pour les chemins qui
  l'atteignent encore (facture redatée — est-ce possible pour une facture validée ? —, clé d'API) ? Croise
  aussi avec ce que prépare la 28-6 (garde de signe à l'entrée d'`accept_one_invoice`).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh`
en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
