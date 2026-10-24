# Prompt — validation P1, Story 31-5 (quatre défauts courts)

*Versionné le 2026-10-24. **Deux lentilles, R et F**, chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-5`, branche `story/31-5`. Fiche :
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md`. Règles : `CLAUDE.md` (§ *Review Iteration Rule*,
*Règle de splitting préventif*, *Exception kesh-db*, *Le prompt d'une passe doit NOMMER le manuel*,
*Inventorier les sites NON RÉSOLUS*), `docs/agents-orchestration.md` § 3 et § 6. La fiche traite #707, #589, #504 et
#469 et écarte #687 et #645 : lis chacune (`gh issue view <N>`, lecture seule).

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **#707.** `insert_with_defaults_in_tx` n'écrit-il vraiment aucun compte de TVA ? `seed_demo`/`final_body`
   l'appellent-ils seuls ? La validation d'une facture avec TVA passe-t-elle bien par un contrôle qui exige
   `default_vat_payable_account_id` (lequel : `vat_payable_account_for_write`, `invoices.rs:1783`, autre) ? Les rôles
   `VatPayable`/`VatRecoverable`/`VatSettlement` existent-ils dans le plan **PME** chargé par la démonstration, avec les
   types que l'AC1 exige ? La condition `invoice_settings_created` tient-elle (la ligne peut-elle préexister à l'étape 2
   d'une démonstration — reset, rejeu de `final_attempt`) ? L'ordre des verrous est-il sûr face à `finalize` et à une
   modification concurrente d'un compte ?
2. **#589.** La garde serveur `reopen` refuse-t-elle bien tout exercice suivi d'un clos, si bien que seul le **plus
   récent** est rouvrable ? La fiche a-t-elle raison de dire que la reproduction de l'issue est inexacte ? L'inventaire
   de l'AC6 (sites qui nomment un exercice à rouvrir) est-il **clos** — cherche toute autre infobulle, bandeau, message
   serveur ou clé FTL qui nomme un exercice clos à rouvrir (`grep -rn "reopen\|rouvr" frontend/src crates/kesh-i18n`).
3. **#504.** Toute la communication passe-t-elle par `build_qrbill_inputs` ? Y a-t-il un autre site qui écrit
   « Facture » dans une QR **émise** (avoir, rappel, autre document) ? Le rapprochement est-il vraiment insensible au
   préfixe (`reference_score`, normalisation, et l'import CAMT/CSV qui remplit `reference`) ? L'affirmation sur
   l'isolation Fluent (aucun `set_use_isolating`) est-elle juste ? `pdf_contains` voit-il le texte du récépissé ?
4. **#469.** Les six sites sont-ils les seuls `AppError::Validation` du journal d'audit (liste **et** export) ? Les dix
   cas du test existant couvrent-ils les six sites ? Le bras `VALIDATION_ERROR` rend-il bien le message tel quel ?
5. **Écartées.** Les motifs d'écart de #687 et #645 tiennent-ils au code ? Une des deux serait-elle en fait plus courte
   que l'une des retenues ?
6. **Manuel.** Les lignes citées de `docs/manual/fr/user-manual.tex` (182-184, 192-204, 848, 3258) disent-elles ce que
   la fiche prétend ? Les retouches prescrites sont-elles exactes **après** correctif (en particulier le message de
   refus cité à l'AC3) ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "indique lequel rouvrir"` (et la
   phrase de la l. 3258). Le manuel d'administration dit-il quelque chose de contraire (démonstration, TVA, journal
   d'audit) ?
7. **Tests et mutations.** Chaque test prescrit rougit-il **avant** le correctif pour la bonne raison (et non par un
   montage) ? Chaque mutation M1-M6 est-elle tuée par un test nommé, ou équivalente ? Un test qui passerait à vide ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses des issues** : chacune des quatre causes est-elle établie au code, ou supposée ? Une prémisse que la
   fiche corrige à tort ?
2. **Périmètre et découpage** : les cinq modules sont-ils bien cinq ? Les quatre volets sont-ils indépendants (aucun
   fichier de code commun hors catalogues) ? Une migration est-elle nécessaire, en réalité ?
3. **Faisabilité** : `t_args` peut-elle passer `pub(crate)` sans conflit ? `build_qrbill_inputs` a-t-il bien les sept
   appels de test annoncés ? La clé `invoice-qr-message-prefix` entre-t-elle en conflit avec une garde (clé FTL inutilisée,
   préfixe de propriété `lint-i18n-ownership`, `I18N_KEYS`) ? Le test HTTP de l'AC2 peut-il créer contact et facture
   sur une démonstration (compte bancaire, exercice, taux 8,1 % présents) ?
4. **Décomptes** : recompte chaque nombre de la fiche (140 issues, six sites, dix cas, sept appels, sept clés, 289
   lignes) à sa source ; signale celui qui ne se retrouve pas.
5. **Cohérence interne** : AC1-AC16, T0-T6, registre C-31-5-0 à 12, *Hors périmètre* — contradiction, critère
   invérifiable, commande de contrôle mal formée (essaie mentalement chaque `grep` de l'AC16 contre le code actuel) ?
6. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (par exemple un test qui attend
   « Facture » ou `nearestLaterClosed`, ou les messages français du journal d'audit), une conséquence pour la
   recette visuelle, le CHANGELOG.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F6) — sans elle,
« 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un
défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire. Aucun `git checkout`/`switch` sous aucune forme ;
aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en
lecture, `git log|show|diff|blame`, `gh issue view`, `pdftotext … -`.
