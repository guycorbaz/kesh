# Prompt — validation P1 de la spec, Story 28-8 (cinq petits correctifs indépendants)

*Versionné le 2026-10-13. **Une seule lentille, R et F combinées**, contexte frais, en lecture seule. Rotation (D6,
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet**
(la fiche a été écrite par Opus). Une lentille au lieu de deux : la story est faite de correctifs mécaniques revus
fichier par fichier (fiche, § Dérogation à la règle de découpage).*

Dépôt `/home/gcorbaz/devel/kesh-28-8`, branche `story/28-8` (base `6de5a7ff`, `origin/main`, 28-7 fusionnée).
Fiche : `_bmad-output/implementation-artifacts/28-8-petits-correctifs.md` (commit `9a661dfb`).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-8* et § *Ordre*.
Issues : #421, #561, #595, #612, #615 — `gh api repos/guycorbaz/kesh/issues/<N>` et `…/<N>/comments` ; voisines
#424 (KF-052), #578, #232.
Tri d'origine (hors dépôt, lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/triage-issues-28.md`.
Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` § *Story 28-8* (C-28-8-1 à 8).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus, « Recompter n'est pas réécrire
l'historique » ; § Règle de splitting préventif ; § Issue Tracking Rule, mots-clés sur la PR) ;
`docs/agents-orchestration.md`.

## Axes prioritaires

1. **Chaque issue entièrement réglée, et rien de plus** (leçon de l'Epic 28, impérative). Pour chacune des cinq :
   relis l'issue **et ses commentaires** en entier ; chaque demande de l'issue est-elle couverte par un AC, ou
   écartée par une exclusion **justifiée** ? Une demande de l'issue laissée sans AC ni exclusion, qui empêcherait
   de la fermer, est au moins **HIGH**. Un ajout non nécessaire à l'issue est au moins **MEDIUM**. Contrôle en
   particulier : #421 « chercher les jumeaux » ; #595 « et la liste voisine `:434` » (relevée sur `056997b0` —
   `git show 056997b0:docs/api-external.md | sed -n 420,440p`) ; #612 « point voisin » ; #615 « dire clairement quoi
   y écrire sur une installation neuve ».
2. **Exactitude des constats au code** : refais chaque `grep`/`sed -n` du § *Le constat, au sol* et du § *La chaîne
   du refus*. En particulier :
   - #561 : `accounts::delete_all_by_company` a-t-elle vraiment zéro appelant, y compris par import groupé
     (`use …::accounts::{…}`), re-export (`pub use`), macro, `kesh-seed`, `kesh-api/tests`, frontend/outillage ?
     Sa suppression laisse-t-elle un import inutilisé (que `clippy -D warnings` refuserait) ?
   - #595 : la route d'annulation du règlement fournisseur **et** celle de l'annulation de facture fournisseur
     rendent-elles bien `409 MATCHED_BANK_TRANSACTION` avec `details.documentId` ? À quel rang exact par rapport
     à `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `ACCOUNT_ARCHIVED`, `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`,
     `BANK_ACCOUNT_CLOSED` (28-4a, `ensure_open_in_tx`) ? L'affirmation « refus défensif — aucun geste de Kesh ne
     rapproche l'écriture de règlement ou d'achat d'une facture fournisseur » est-elle **vraie** : recense
     **toutes** les écritures de `matched_entry_id` (pas seulement les cinq de la fiche — `kesh-seed`,
     `test_endpoints.rs`, import, restauration, lots de paiement, `UPDATE … SET … matched_entry_id`) ?
   - #421 : la route `record_manual_reminder` n'a-t-elle aucune autre garde de date (date de la facture, échéance,
     dernier rappel, exercice) qu'une date d'une heure avant pourrait heurter — par exemple autour de minuit UTC,
     quand la facture du test est datée « aujourd'hui » ? Le format `toISOString().slice(0, 19)` est-il accepté par
     le désérialiseur `NaiveDateTime` de la route ?
   - #615 : le texte prescrit pour le listing et la prose est-il **vrai** au code (`config.rs`
     `is_template_placeholder` et ses appelants ; `docker-compose.yml`, `${MARIADB_PASSWORD:?` ; `.env.example`) ?
     La fiche corrige une affirmation de la consigne de story (« le gabarit `<…>` est refusé au démarrage ») :
     cette correction est-elle juste ?
   - #612 : la livraison en v0.8.0 est-elle établie (`CHANGELOG.md` `[0.8.0]`, `README.md`, `user-manual.tex`) ? La
     puce prescrite promet-elle quelque chose que le code ne tient pas, ou omet-elle une réserve nécessaire (#232) ?
3. **Manuel et documentation contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` (§ *Étape 3 — Créer
   le fichier .env*, l. 231-280) et `docs/manual/fr/marketing-brochure.tex` (l. 378-445), **et leurs PDF**,
   aplatis : `pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' '`. Ce que la fiche prescrit d'écrire
   sera-t-il vrai ? Ce qu'elle laisse tel quel (paragraphe des chevrons acceptés, `DATABASE_URL` entre chevrons,
   intitulé « Backlog (Epic 13 à 15) ») reste-t-il vrai après le changement ? `docs/api-external.md` l. 516-536 ;
   `docs/testing.md` l. 328-440 (la § *Les échecs attendus* entière) : la mise à jour prescrite à l'AC2 est-elle
   cohérente d'un bout à l'autre de la section (chapeau, tableau, paragraphes, formule, § *Comment lire un rouge*) et
   respecte-t-elle « Recompter n'est pas réécrire l'historique » ? La formule « 7 à 10 » est-elle fondée ?
4. **Gardes et tests existants qui liraient ces fichiers** : `crates/kesh-api/tests/textes_coherents.rs` (G-gardes
   sur les trois `.tex`, `api-external.md`, `README.md`) et `crates/kesh-api/tests/configuration_transmise.rs`
   (lit `admin-manual.tex` et `.env.example`) — une modification prescrite en ferait-elle rougir une ? La fiche
   les nomme-t-elle là où il faut ? La garde G19 (URL au tag) est-elle préservée ?
5. **Gates et consignes** : gate complet (kesh-db), PDF régénérés et contrôlés aplatis, E2E complet au dernier
   commit de code avec la liste des échecs attendus mise à jour, CHANGELOG dans la section **unique** `## [0.13.3]
   — Non publié`, PR `closes #421 #561 #595 #612 #615` écrite **comme consigne pour l'orchestrateur et jamais dans
   un message de commit**, commits en `refs`. Les consignes communes (port 3042, bases `kesh_288` /
   `kesh_e2e_288`, verrou, pas de `docker restart`) sont-elles respectées par les tâches ? La dérogation au
   découpage est-elle recevable au regard du § *Règle de splitting préventif* ?
6. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — AC (9), tâches (T0-T8), choix (8),
   modules (6), occurrences de `MATCHED_BANK_TRANSACTION` dans `api-external.md` (6 → 8), numéros de ligne cités
   (échantillonne-en au moins quinze), « dix-huit montages » de `journal_entries::delete_all_by_company`.

## Lentille unique — R + F

- **R (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ; chaque demande des cinq issues
  couverte par un AC testable ou vérifiable, chaque AC par une tâche, chaque AC de code par une preuve **qui
  rougirait** (mutation M1, compilation) ; chaque fait, chemin et numéro de ligne revérifié.
- **F (adversaire de la conception)** : la correction prescrite est-elle la bonne, la plus petite, et vraie à
  toute heure / dans tout état ? Les textes prescrits (testing.md, api-external, brochure, manuel, CHANGELOG)
  disent-ils exactement ce que fait le code, sans promesse ni omission ? Les choix C-28-8-1 à 8 tiennent-ils ?

## Ce que tu rends

Findings numérotés `RF1-…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-8-validate-p1.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh`
en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`/`blame`/`grep`, `gh api` et `gh issue view` en lecture, `pdftotext` vers la sortie
standard.
