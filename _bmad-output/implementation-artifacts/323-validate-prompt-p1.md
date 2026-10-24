# Prompt — validation P1, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3` (partie de `275e9b97`). Fiche :
`_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md` (C-32-3-1 à 16). Contexte de l'epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-3*, § *Ce que le code dit déjà*, § *Points
d'attention*) ; spike : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 4.1-4.4, § 5.3.1, § 6) ; registres
`epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `epic-32-choix-autonomes-322.md`. Règles : `CLAUDE.md` (§ *Review
Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —, *Règle de
splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`, *Le prompt d'une passe doit NOMMER le
manuel*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes rendus*, *Priorités des défauts*),
`docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas ; aucune
migration ; au plus cinq modules ; **la décision Q11 (lecture seule de production) est en attente de Guy et la story ne
doit pas en dépendre** — la fabrication d'une `Lecture` en lecture seule est à la 32-6 ; aucune donnée réelle.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 10** (§ *Ce que le code dit*) : 18 handlers ; sept fonctions d'audit et leurs lignes ; 18 sites
   d'appel (recompte-les et confronte-les, un par un, au tableau de l'AC6 : handler, route, usage, fonction d'audit,
   arguments) ; aucun générateur de `kesh-report` n'écrit (refais la recherche, et cherche aussi ce qu'elle pourrait
   manquer : `query(` composé, appel d'un dépôt `kesh-db` qui écrit, `FOR UPDATE` dans un dépôt appelé par un
   générateur) ; l'ordre rendu-puis-trace des exports ; l'écart du grand livre (fait 5) ; L2 ; `list_accounts` et ses
   trois lectures ; RBAC de chaque route dans `lib.rs` (l'exigence API de chaque ligne du tableau de l'AC6 égale-t-elle
   le routeur où la route est montée ?) ; couverture actuelle de la trace (fait 9) ; construction littérale d'`AppState`
   dans des tests (fait 10 — trouve ces sites).
2. **Le motif et sa faisabilité en Rust** : `Rapport<T, U>` à champs privés, `afficher`/`exporter`, un service générique
   sur `U` dont l'`Exigence` dépend de l'usage — compilable sans contorsion ? `exporter` avec une fermeture de rendu
   couvre-t-il les neuf exports (contexte PDF chargé avant, `journal_filter_label` de l'export des journaux,
   `resolve_type_slug`, noms de fichier qui lisent la période ou les bornes d'un projet) ? `use … FormatExport as
   ExportFormat` rend-il `ExportFormat` visible au `mod tests` de `reports.rs` par `use super::*` sans modifier ce bloc ?
   `Lecture::pool()` en `pub(crate)` suffit-il aux générateurs et à `letterable_account_ids` (connexion acquise) ?
3. **Comportement inchangé** : pour chaque handler, l'ordre des validations et des erreurs après déplacement dans le
   service (format, `fiscalYearId`, période, `accountIds`, projet), les deux régimes de pagination du grand livre, le
   `span` d'export, la date `as_of`, l'erreur `Internal` de `list_accounts`. Un test existant pourrait-il rougir ?
   Lesquels frappent ces routes (refais le `grep` de l'AC7 — la liste de neuf fichiers est-elle complète, et les
   comptes `sqlx::test` exacts) ?
4. **Le contrôle d'accès** : `Exigence`, `exiger` et la comparaison de rôle (`rbac.rs`, `Role::level`) ; le refus
   **avant** la lecture et le test (c) de l'AC8 qui le mesure (un `fiscalYearId` inexistant produit-il bien, sans le
   contrôle, une erreur distincte de `Forbidden` pour chaque rapport ? et pour la balance âgée, qui n'a pas de
   période ? et pour `lister` ?) ; les cas (a) à (e) sont-ils constructibles et non vides pour les dix services ?
5. **La trace et la séparation** : l'AC9 — une `Lecture` en `SET SESSION TRANSACTION READ ONLY` fait-elle réussir les
   neuf générateurs et `lister` (lecture, `acquire`, transactions implicites) ? Les `details_json` attendus des huit
   chemins sont-ils exactement ceux du code (relis les sept corps) ? M1 tue-t-elle bien les huit assertions par le
   mécanisme décrit (1792 avalé par le `warn!`) ? Une trace écrite sur une `Trace` qui partagerait par erreur le pool de
   la `Lecture` de test serait-elle vue ?
6. **Inventaire clos (AC11)** : est-il vraiment clos ? Cherche toute autre écriture atteignable depuis les 19 chemins
   (`touch_last_used`, `last_login`, rafraîchissement de jeton, `get_or_create_*`, journalisation en base, verrou) et
   dis si l'AC9 (a) l'exécuterait.
7. **Mutations (AC12)** : M1 à M10 (M1′, M2′) — chacune est-elle réalisable telle qu'écrite et tuée par l'assertion
   **nommée** ? Une passerait-elle à vide ? M7 : le test HTTP cité reste-t-il bien vert ?
8. **Manuel et documentation** : la fiche dit « aucun changement visible ». Vérifie qu'aucun texte ne décrit ce qui
   bouge : `docs/manual/fr/admin-manual.tex` et `user-manual.tex` (journal d'audit, `report.generated`, rapports,
   exports), `docs/api-external.md`, et **les PDF aplatis** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}report\.generated.{80}"`
   (et `journal d'audit`, `audit`). Un texte promet-il, à propos de la trace d'un export, ce que le fait 5 contredit ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chacune des affirmations « établi au code » tient-elle ? Une prémisse du kickoff ou de la spike
   reprise sans vérification (prémisse 6 : `touch_last_used` « sur les chemins de lecture visés ») ?
2. **Périmètre et découpage** : quatre modules (`services`, `routes/reports`, `routes/accounts`, `helpers`) plus
   `lib.rs` et un fichier de test — juste ? `kesh-report` vraiment non touché ? Le choix C-32-3-1 (les 18 handlers,
   exports compris) est-il défendable contre C-32-0-4, ou la story devient-elle trop large pour un motif ? Aucune
   migration, aucun dépôt `kesh-db` touché (l'exception « gate complet en boucle » ne s'appliquerait pas) ?
3. **Indépendance de Q11** : quelque chose dans la fiche préjuge-t-il de la forme de la lecture seule (types, noms,
   commentaires prescrits, AC15) ? L'AC15 (`grep -rniE "transaction read only|tx_read_only" crates/kesh-api/src` → vide)
   est-il juste aujourd'hui et tenable après le développement ?
4. **Décomptes** : recompte à la source chaque nombre — 18 handlers, sept fonctions, 18 sites, 9 + 9 + 1 chemins, huit
   chemins de trace, dix services, neuf fichiers de test et leurs comptes, quatre modules, C-32-3-1 à 16.
5. **Cohérence interne** : AC1 à AC15, T0 à T6, registre, *Hors périmètre*, *Fichiers touchés*, Change Log —
   contradiction, critère invérifiable, commande mal formée (essaie mentalement **chaque** commande des AC7, AC10,
   AC11, AC15 contre le code actuel **et** contre un correctif plausible : rend-elle ce que la fiche annonce ?).
6. **Ce que la fiche omet** : un appelant des fonctions déplacées hors de `reports.rs` (`exports.rs`, `util.rs`,
   `routes/exports`), un test unitaire de `reports.rs` qui dépend d'une fonction déplacée, une garde existante qui lit
   ces sources (`audit_route_registry.rs`, `textes_coherents.rs`, `audit_label_registry.rs`), la recette visuelle (aucun
   écran ne change ?), le CHANGELOG, l'issue demandée au C-32-3-8 (le défaut est-il réel, et sa priorité juste ?).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (R1 à R8, F1 à F6) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement
n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie standard.
