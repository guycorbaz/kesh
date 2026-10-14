# Prompt — revue de code P2 ciblée, Story 29-2 (un paiement reçu après l'échéance, ou après un rappel, est proposé)

*Versionné le 2026-10-14. Passe ciblée (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, **Haiku**, contexte
frais, lecture seule stricte, braquée sur le **seul diff de la remédiation de la revue P1**. Rotation (D6, rétro 25) :
P1 Sonnet (B/E/A), P2 ciblée Haiku.*

Dépôt `/home/gcorbaz/devel/kesh-29-2`, branche `story/29-2`. Code livré par la story : `8f483388` et `d45279f7` ;
revue P1 sur `git diff c50f56e8 e5d76144`.

**Objet, et lui seul** : la remédiation P1, deux commits, **lus comme un seul diff aplati** :

```sh
git diff e5d76144 e8b94efc
```

(soit `729a1dc9` — test et commentaires — puis `e8b94efc` — manuel et PDF, fichier d'epic, registre, Dev Agent
Record). Ne lis **pas** les deux commits séparément : leurs hunks se recouvrent dans
`crates/kesh-db/tests/reconciliation_repository.rs`. Le commit qui suit `e8b94efc` (Change Log, statut,
`sprint-status.yaml`, ce prompt) est un compte rendu : lis-le pour l'axe 7 seulement (`git show HEAD`).

Rapports remédiés (à lire pour savoir ce qui était demandé, **pas** à refaire) :
`/home/gcorbaz/devel/kesh-gate-logs/29-2-review-p1-B.md`, `-E.md`, `-A.md` ; leur traitement : Change Log de la fiche
`_bmad-output/implementation-artifacts/29-2-paiement-tardif.md`, entrée « Revue de code P1 ».

⚠️ **Haiku et les diffs** (`CLAUDE.md` § *Haiku-specific guardrails*) : toute affirmation de **présence** ou
d'**absence** d'un texte ou d'un code (« le doc-comment dit encore X », « le cas l n'existe pas ») doit être **étayée
par la sortie d'un `grep -nF "<chaîne>" <fichier>`** recopiée dans le rapport, **sur le fichier final** — jamais sur
les numéros de ligne du diff. Un finding sans cette sortie ne compte pas.

## Lentille unique — chasseur de régressions de la remédiation P1

Pour chaque point : le texte ou le test neuf est-il **vrai au code**, **cohérent** avec le reste, **complet** ?

1. **Cas l de l'AC5** (`the_anchor_is_the_latest_of_date_due_date_and_last_reminder`) : `D` = 2026-06-15,
   `F = D − 25`, `E = D − 50`, rappel non annulé à `D − 60`. Recalcule l'ancre du code livré
   (`invoice_payment_anchor_sql!`, `grep -nF "GREATEST(i.date, COALESCE(i.due_date, i.date))"
   crates/kesh-db/src/repositories/reconciliation.rs`) : vaut-elle `D − 25`, et la facture est-elle candidate
   (`A ≥ D − 30`, `F ≤ D + 1`) ? Sous M14 (premier terme réduit à `COALESCE(i.due_date, i.date)`), l'ancre vaut-elle
   `D − 50` (2026-04-26), comme le journal `/home/gcorbaz/devel/kesh-gate-logs/292-mutations-M14-revue-p1.log` le
   rapporte ? Le montant du cas l (rang 11, `101.00 + 11 × 1.00`) entre-t-il en collision avec un autre cas, ou avec
   la tolérance de 5 centimes ? Le cas l change-t-il le sens d'un autre cas (l'ordre d'insertion, une facture d'un
   cas précédent qui deviendrait candidate au montant de l) ?
2. **Doc-comment de `invoice_payment_anchor_sql!`** : le texte neuf est-il juste (le repli `i.date` du terme du
   rappel ne vaut `F` que sans rappel) ? Est-il vrai qu'**aucune** contrainte n'interdit un rappel antérieur à la
   facture (`crates/kesh-api/src/routes/dunning_reminders.rs`, `grep -nF "sent_at >"`) ? Reste-t-il, ailleurs dans le
   dépôt **hors relevés datés**, une affirmation « M14 équivalente » propre à la 29-2 (`grep -rnF "M14"
   crates _bmad-output/implementation-artifacts/29-2-paiement-tardif.md
   _bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md`) ? Les mentions « M14 » d'autres stories
   (`util.rs`, `reports.rs`, `bank_import_delete.rs`…) sont hors sujet.
3. **Doc-comment de `invoice_payment_anchor_join_sql!` (C-29-2-17)** : l'affirmation « la dérivée ne peut pas
   référencer `i` » et « un filtre demanderait un paramètre lié de plus dans les deux lecteurs » sont-elles vraies
   (lis la requête de proposition et `INVOICE_PAYMENT_ANCHOR_SCALAR_SQL`, `sed -n`) ? La mention de
   `split_materialized` est-elle présentée comme **non mesurée**, et non comme un fait établi ?
4. **Commentaires de fenêtre** (`crates/kesh-api/tests/reconciliation_e2e.rs`, en-tête de la section 28-7 ;
   `crates/kesh-db/tests/reconciliation_repository.rs`, doc de `find_unpaid_invoices_filters_by_30_day_window`) :
   disent-ils la règle actuelle (`D − 30 ≤ A`, `F ≤ D + 1`) sans contredire les assertions des tests qu'ils
   coiffent ? Aucune assertion n'a-t-elle changé (`git diff e5d76144 e8b94efc -- crates | grep -E '^[+-]\s*assert'`
   doit être vide) ?
5. **Manuel `:1305` et PDF** : la phrase neuve (« au plus 30 jours après la plus tardive de ces dates : le dernier
   rappel, l'échéance, la date de la facture ») est-elle juste au code et cohérente avec `:2029` ? Contrôle du PDF,
   **aplati, par un tube** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF
   "<motif>" | wc -l` — la phrase neuve 1, « si l’une est plus tardive » 0 (apostrophe typographique), `??` 0. Le
   PDF est-il bien régénéré depuis le `.tex` final (une autre phrase modifiée du `.tex` absente du PDF) ?
6. **Contrôle de pré-release de la v0.13.4** (`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`,
   § *Releases*) : les deux commandes, rejouées **en lecture** sur l'arbre courant, rendent-elles les trois sites
   annoncés (`user-manual.tex:2029`, `:2031`, `CHANGELOG.md:15`) ? Une formulation du conseil de rejet leur
   échapperait-elle (`grep -rn "rejet" docs/manual/fr/user-manual.tex CHANGELOG.md docs/api-external.md`, trier) ? Le
   contrôle est-il exécutable par qui prépare le tag sans connaître la 29-2 ? La condition « 29-5 mergée » ajoutée est-
   elle fidèle à C-29-2-13 ?
7. **Comptes rendus** (Dev Agent Record annoté, registre C-29-2-7/8/15/17, Change Log « Revue de code P1 »,
   `sprint-status.yaml`) : les annotations **ajoutent** sans réécrire les relevés passés (compare la ligne M14 du
   tableau et l'inventaire (c) avec `git show e5d76144:_bmad-output/implementation-artifacts/29-2-paiement-tardif.md`)
   ? Recompte depuis la source : bruts 4 MEDIUM / 10 LOW, distincts 2 MEDIUM / 7 LOW ; 21 mutations (18 tuées, 2
   équivalentes) ; 17 choix (`grep -c '^### C-29-2-' _bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md`)
   ; inventaire (c) à `e8b94efc` (rejoue la commande du § *Inventaire (c)* de la fiche en lecture). Les chiffres de
   gate (3611 passés, 6 ignorés ; 146 fichiers, 1610 tests) sont-ils ceux des journaux
   `/home/gcorbaz/devel/kesh-gate-logs/292-gate-backend-revue-p1.log` et `292-gate-frontend-revue-p1.log` ?
   La justification de l'E2E non rejoué (aucune ligne exécutable de production, aucune spec, aucun frontend touchés)
   est-elle vraie au diff (`git diff --stat e5d76144 e8b94efc`) ?
8. **Production** : la remédiation ne touche-t-elle **aucune ligne exécutable** de production ? `git diff e5d76144
   e8b94efc -- crates/*/src` ne doit contenir que des lignes `///` ou `//`.

Un défaut introduit par la remédiation qui touche un AC, un test ou un texte prescrit est **au moins MEDIUM**.

## Ce que tu rends

Findings numérotés `P2-n` (CRITICAL / HIGH / MEDIUM / LOW), chacun avec `fichier:ligne`, **la preuve** (sortie de
`grep -nF` ou de `sed -n` recopiée, sur le fichier final), son **origine** (remédiation P1, ou conception antérieure)
et la correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les huit points ci-dessus, un par un,
avec ce qui a été lancé) est **obligatoire** : un « 0 finding » sans elle ne compte pas comme passe. Déclare tout
écart au protocole.

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/29-2-review-p2-ciblee.md` ; dernier message : le chemin, le
bilan chiffré, une ligne par finding MEDIUM ou plus.

## Interdits — lecture seule stricte

⛔ N'écris **aucun** fichier, sauf le rapport ci-dessus. Aucune commande qui écrit dans le dépôt, dans une base ou sur
GitHub, ni qui compile ou exécute : **aucun `git checkout` / `switch` / `restore` / `reset` / `stash` / `commit` /
`add` / `rebase` / `tag`**, sous aucune forme ; `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`,
`latexmk`, `cargo`, `npm`, `npx`, `sqlx`, `mariadb`, `docker`, `python3`, `sed -i`, `perl -pi`, `gh issue create` /
`comment` / `edit`, `gh pr …`, SQL. Aucun fichier temporaire (les contrôles passent par un tube). Autorisés : lecture,
`grep`, `sed -n`, `awk`, `pdftotext … -` vers un tube, `git log` / `show` / `diff`, `gh api` en lecture.
