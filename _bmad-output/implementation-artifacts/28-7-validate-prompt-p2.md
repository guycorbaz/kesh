# Prompt — validation P2 de la spec, Story 28-7 (une facture postérieure au paiement n'est plus proposée)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était Sonnet : P2 est
**Opus** pour les deux lentilles. Passe **complète**, pas ciblée : la remédiation P1 a ajouté une clause SQL, un
test, cinq mutations et un choix, et retiré un mécanisme — elle change une règle de la fiche.*

Dépôt `/home/gcorbaz/devel/kesh-28-7`, branche `story/28-7` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-7-facture-posterieure-paiement.md` (commit `ad14c149`).
Remédiation P1 à relire en premier : `git show ad14c149` (fiche, registre, `sprint-status.yaml`).
Rapports P1 (hors dépôt) : `/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p1-R.md` et `…-p1-F.md`.
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-7* et § *Ordre*. Issues #548 (l'objet) et
#529 (voisine, **non absorbée**) : `gh api repos/guycorbaz/kesh/issues/548`, `…/529`.
Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-7-1 à 10, **C-28-7-4 retiré**) ;
antécédent : `epic-15-choix-autonomes.md` C-15-5c-3.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch) ; `docs/agents-orchestration.md`.

⚠️ La 28-5 et la 28-6 touchent le même `routes/reconciliation.rs` et seront mergées avant le développement : les
numéros de ligne de la fiche sont relevés sur `5aad8b35`, ce qui est le bon état pour cette passe.

## Décisions de l'orchestrateur prises en P1 (à ne pas rouvrir, mais à vérifier dans leur écriture)

- **Clamp à la plage `DATE` retiré** (C-28-7-4 retiré ; AC1, AC6, ancienne M10, T1).
- **`ORDER BY i.date ASC, i.id ASC` intégré** avant le `LIMIT 50` (C-28-7-10, AC2, test de dépôt
  `find_unpaid_invoices_orders_by_date_then_id_before_limit`, mutations M10, M13, M14).
- **CHECK `chk_invoices_paid_at_after_date`** ajoutée comme troisième détentrice de la borne basse (inventaire (b),
  AC9, C-28-7-1).

Un désaccord de fond avec ces décisions se rapporte en **note à l'orchestrateur**, pas en finding ; un défaut de
leur **écriture** (contradiction, résidu, test qui ne prouve pas, mutation intuable) est un finding.

## Axe premier — les régressions de la remédiation P1 (les deux lentilles)

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Lis `git show ad14c149`
ligne à ligne, et cherche :

1. **Résidus du clamp** : toute phrase, AC, tâche, mutation, test ou choix qui suppose encore un ramènement de
   borne, `checked_*_days`, la plage `[1000-01-01, 9999-12-31]`, ou un « `500` évité ». Greppe la **valeur**
   (`9999`, `1000-01`, `checked_`, `clamp`, `ramen`, `plage`) sur la fiche, le registre, l'epic et
   `sprint-status.yaml`. L'arithmétique prescrite à l'AC1 (« ordinaire, sans ramènement ») peut-elle **paniquer**
   sur une date réellement atteignable ? (`NaiveDate` de chrono 0.4.45 au `Cargo.lock`, `Sub<Days>` /
   `checked_sub_days`, `Duration` ; dates lues de colonnes `DATE`.) L'affirmation « `check` compare exactement ce
   que compare l'acceptation actuelle, à toute date » est-elle vraie ?
2. **L'`ORDER BY`** : la clause est-elle **syntaxiquement placée** juste dans la requête réelle
   (`crates/kesh-db/src/repositories/reconciliation.rs:128-140` — `HAVING` sans `GROUP BY`, `LIMIT 50`, jointures
   de `amount_due_derived_joins()` : y a-t-il un `GROUP BY` ou une sous-requête qui rendrait `i.date`/`i.id`
   ambigus ou non agrégés) ? L'affirmation « ordre de fait de l'index puis de la clé primaire » est-elle étayée
   (DDL de l'index, `test-schema/0001_schema_squash.sql:745`), ou supposée ? Les tests existants qui lisent
   `candidates[0]` ou un ordre (`reconciliation_repository.rs`, `invoice_write_off.rs`, `reconciliation_e2e.rs`) —
   l'`ORDER BY` en change-t-il un ? Le tri aval (`kesh-reconciliation/src/matching.rs:104-110`) est-il bien
   `score DESC, invoice_id ASC` comme cité ?
3. **Le test à 52 factures** : est-il **réalisable** avec les helpers du fichier
   (`crates/kesh-db/tests/reconciliation_repository.rs` — création de facture validée, écriture comptable liée,
   exercice) ; « même reste dû » et insertion « en dates décroissantes » suffisent-ils à faire diverger `id ASC`
   et `date ASC` ; les assertions (« 50, les 50 premières de `(date, id)` dans l'ordre, les deux dernières
   absentes ») tuent-elles **M13** et **M14** ; M10 est-elle honnêtement décrite (possiblement équivalente au
   plan) ou faut-il un autre moyen de la tuer ? Le test garde-t-il un sens si la fenêtre `[D−30, D+1]` ne compte
   que 32 jours pour 52 factures ?
4. **M11 et M12** : tuables par les tests nommés ? M12 (`let paid_at_candidate = bank_transaction.booking_date;`)
   et l'assertion `settled_on == B − 5` de l'AC5 (b) — la colonne `invoice_settlements.settled_on` existe-t-elle
   sous ce nom (`test-schema/0001_schema_squash.sql`), est-elle écrite depuis `paid_at_candidate`
   (`routes/reconciliation.rs:1994`) ? M11 se distingue-t-elle de M5 et M6 ?
5. **La CHECK et `write_off_invoice`** : les citations (`migrations/20260419000001_invoice_paid_at.sql:32-33`,
   squash `:755`, `invoice_settlements_write.rs` `:67`, `:115`, `:429`, `:489`) sont-elles exactes ; « ne se
   desserre pas sans migration, donc sans bump `min_required` » est-il juste au regard de P1-P4 du `CLAUDE.md`
   (une CHECK assouplie est-elle « breaking » ?) ; la description de `write_off_invoice` (« solde par escompte,
   perte, frais ou arrondi ») est-elle celle du code ?
6. **AC3 et `Duration`** : les huit sites de `paid_at_candidate` et les emplois de `Duration` sont-ils exacts
   (`grep -nw`) ; la consigne « retirer `Duration` de l'import s'il n'a plus d'emploi » est-elle complète ?
7. **Textes nouveaux** : en-CH « it cannot settle this invoice », la phrase CHANGELOG ajoutée (deux sens du
   changement de date de référence, et « les 50 plus anciennes ») — vraies au code prescrit ? Le CHANGELOG
   annonce-t-il désormais un changement de comportement (l'ordre) que rien d'autre n'écrit (manuel) ?
8. **Recomptes de la remédiation** : 11 AC, T0-T8, **M1-M14**, **10 choix dont 9 en vigueur**, cinq modules,
   « tests neufs prescrits » — recompte-les depuis la source. Le Change Log P1 dit-il vrai sur ses propres
   chiffres (« 5 MEDIUM dont 4 distincts, 12 LOW dont 10 distincts ») contre les deux rapports P1 ?

## Axes communs (inchangés depuis la P1, à refaire en entier)

1. **Périmètre minimal** : la fiche ferme-t-elle #548 **dès elle-même**, sans rien de plus ? Chaque AC et chaque
   tâche est-il nécessaire à #548 ou à la non-régression — l'`ORDER BY` est désormais un ajout **assumé** par
   l'orchestrateur (C-28-7-10) : ne le rapporte pas comme finding de périmètre, mais tout **autre** ajout non
   nécessaire est au moins **MEDIUM** ; une exclusion qui laisse #548 ouvert, **HIGH**.
2. **Inventaire clos des sites** (Dev Notes § *Inventaire (a)* et *(b)*) : refais les `grep` copiés et
   **élargis-les** — toute requête qui filtre `invoices` par date en présence d'une transaction (`i.date`,
   `invoices.date`, `QueryBuilder`, `format!`), tout code qui compare `invoice.date` à une date de transaction
   (`value_date`, `booking_date`, `paid_at`, `settled_on`), toute contrainte de schéma (CHECK, trigger) portant
   sur `invoices.date` et une date de paiement, `kesh-reconciliation`, `kesh-seed`, `test_endpoints.rs`, le
   frontend. Un site qui choisit ou valide une facture pour une transaction et que la fiche ignore est au moins
   **HIGH**.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Algorithme de matching*
   (l. 1917-1921, dont le paragraphe `\label{sec:facture-posterieure}`), FAQ l. 2787, rappels l. 1299 ; et
   `docs/manual/fr/marketing-brochure.tex:167`. Ce que le manuel dit **aujourd'hui** est-il vrai au code de
   `5aad8b35` ? Ce que l'AC8 prescrit d'écrire sera-t-il vrai au code que la fiche prescrit (y compris la phrase
   de l'AC8 sur le règlement depuis la fiche, réécrite en P1) ? Contrôle aussi le **PDF**, aplati : `pdftotext
   docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas est au
   moins MEDIUM.
4. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — appels de
   `find_unpaid_invoices_for_window` (« dix autres appels de tests »), `#[sqlx::test]` de `reconciliation_e2e.rs`
   (81), AC, tâches, mutations, choix, modules, numéros de ligne cités (dont ceux ajoutés en P1).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #548 et du § *28-7* de l'epic couverte par un AC testable, chaque AC par une tâche, chaque AC
  par un test **qui rougirait** ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ;
  les textes prescrits (quatre `.ftl`, repli de `failed-proposal-label.ts` et la garde G13
  `i18n-repli-divergent-actif.test.ts`, manuel, CHANGELOG) confrontés à ce que le code prescrit fera ; les gardes
  existantes (`i18n-keys.test.ts` et ses compteurs, `lint-i18n-ownership`, `textes_coherents.rs`,
  `test_schema_guard.rs`, les tests `post_accept_rejects_payment_date_*`) ; les tests existants que la nouvelle
  fenêtre **ou le nouvel ordre** feraient rougir (§ *Inventaire (c)* et T4, quatre fichiers nommés).
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : refais l'algèbre
  `D − 30 ≤ F ≤ D + 1` contre `routes/reconciliation.rs:1656-1682`, bornes incluses ou exclues, **sans le
  clamp** ; la démonstration « #529 non aggravé » ; les tableaux de l'AC4 et de l'AC5 cas par cas, **avec les
  dates concrètes prescrites en P1** (`D = B = 2026-06-15` : toutes les dates de facture et de paiement tombent-
  elles dans l'exercice 2026 ouvert par `insert_fake_fiscal_year`, et l'exigence d'exercice porte-t-elle sur la
  date du paiement seule ou aussi sur la facture ?) ; la réalisabilité de l'AC4 (sept couples dans un seul
  `POST /accept`, unicité des montants au-delà de 0.05, références distinctes, verrou de compte, ordre de
  traitement) ; l'usage de l'index `idx_invoices_company_validated_unpaid_date` par `BETWEEN ? AND ?` **et** par
  l'`ORDER BY` (filesort ou non — raisonné, `EXPLAIN` interdit) ; les mutations **M1-M14** toutes tuables par les
  tests nommés, et s'il en manque une évidente ; le libellé raccourci (C-28-7-5) pour les chemins qui l'atteignent
  encore ; le croisement avec la 28-6 (garde de signe à l'entrée d'`accept_one_invoice`).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il vient de la
**remédiation P1** (`ad14c149`) ou de la conception d'origine. Toute affirmation de présence ou d'absence = sortie
de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas
comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL (dont
`EXPLAIN`), `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
