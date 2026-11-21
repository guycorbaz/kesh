# Validation P3 de la fiche 48-14-solde-dette-epic — prompt de la passe

*Passe **complète**, contexte frais, lecture seule, confiée à **Sonnet** (la P2 était Opus : le modèle alterne,
`CLAUDE.md`, § *Review Iteration Rule*). **Pas de passe ciblée** : la P2 a constaté un **signal de recyclage** (deux de
ses trois MEDIUM nés de la remédiation de la P1). Rédigé le 2026-11-19 avec la remédiation de la P2. P1 : 0/0/4/7 ;
P2 : 0/0/3/6 ; tous appliqués (entrées 0.2 et 0.3 du Change Log).*

---

Tu valides la fiche **`_bmad-output/implementation-artifacts/48-14-solde-dette-epic.md`** de Kesh (release 0.20.0,
Epic 48, story de solde), dans le worktree **`/home/gcorbaz/devel/kesh-48-14`**, branche `story/48-14-solde-dette-epic`.
Tu travailles en **français**. Tu n'as pas d'autre contexte que ce prompt et les fichiers qu'il nomme : lis-les.

## À lire d'abord

1. `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` (et `e0190-commun.md` qu'il nomme), puis le
   `CLAUDE.md` **du worktree**.
2. `_bmad-output/planning-artifacts/epic-48-corrections-0-20-0.md` : la section « 48-14 », le tableau
   « Fichiers et modules touchés », l'ordre de sortie. **Ils font foi pour le périmètre** : plafond de **quatre**
   issues (#1263, #1262, #1261, #1255).
3. Les issues : `gh issue view <N> --json number,title,labels,milestone,body,comments` pour 1263, 1262, 1261, 1255
   (et 1007, citée). Lecture seule.
4. **D'abord les remédiations de la P1 et de la P2** : les rapports
   `/home/gcorbaz/devel/kesh-gate-logs/e4814/validation-p1.md` (V1-1 à V1-11) et `validation-p2.md` (V2-1 à V2-9), les
   entrées 0.2 et 0.3 du Change Log ; `git -C /home/gcorbaz/devel/kesh-48-14 show HEAD~1` et `show HEAD` montrent les
   deux diffs de remédiation. Pour chaque finding : la remédiation le ferme-t-elle, **au code**, sans en ouvrir un
   autre ? Les deux recyclages de la P2 venaient de corrections écrites sans relire le mécanisme exact : **relis
   toi-même le mécanisme** avant de juger chaque correctif. En particulier : D4 contre `fautes_en_dernier`
   (`bank_account_closed_inventory.rs:1342`–`:1390`), `the_last_acquisition_rule_sees_each_locking_token` (`:1781`–`:1860`),
   `the_last_acquisition_rule_reports_each_fault_on_built_cases` (`:1666`) ; D3 contre `closeStale`
   (`GroupSettlementModal.svelte:366`–`:373`), les chemins de succès (`:660`–`:661`), `ReconciliationProposals.svelte`
   (`load()` `:176`–`:200`, gestionnaires `:298`–`:335`, `confirmGesture`/`cancelGesture` `:438`–`:456`, `onOpenChange`
   des modales `:872`–`:928`) et le report par `queueMicrotask` (une relecture double ou perdue est-elle encore
   possible ? l'extinction du drapeau avant le premier `await` est-elle tenable ?) ; les doublures de T5 contre
   `ModalSuccessStub*.test.svelte` et `ReconciliationProposals.test.ts:45`–`:65` (M3, M4, M9, M11 rougissent-ils
   vraiment ?) ; AC13 contre l'écrivain (`admin_auto_backup_e2e.rs:95`–`:120`) ; AC9 contre les audits réellement
   écrits par `reverse_in_tx` et la dissolution du lettrage.
5. La fiche, en entier — pas seulement le diff : une passe complète relit tout.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Périmètre contre le fichier d'épique** : quatre issues, pas une de plus ; tout écart au tableau des fichiers
   (la fiche en déclare un : `ReconciliationProposals.svelte`) est-il justifié, et dit ? La fiche ajoute-t-elle en
   douce une dette hors plafond ?
2. **Prémisses relues au code** : chaque `chemin:ligne` cité se relit **à la tête du worktree** ; la ligne copiée
   doit y être. Vérifie au moins quinze citations, dont toutes celles qui portent une décision (D1, D3, D4, D6) :
   `GroupSettlementModal.svelte:537`, `:555`, `:570`, `:577`, `:655` ; `ReconciliationProposals.svelte:182`, `:184`, `:555`, `:922`,
   `:568`, `:930` ; `grouped_settlements.rs` (les deux bras) ; `bank_account_closed_inventory.rs` (registres).
3. **La cause de KF-063 (#1261)** : la fiche affirme une chaîne en six liens (écrivain qui compte les réécritures →
   `update_local_file_header` qui recule et écrit 4 octets → erreur après recul → `Drop` → `debug_assert!` de
   `write.rs:1078` → `JoinError` → `Failed`). Relis la source de zip 2.6.1 dans
   `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zip-2.6.1/src/write.rs` (`finish_file`, `start_entry`,
   `update_local_file_header`, `finalize`, `Drop`, `abort_file`, `ok_or_abort_file`) et
   `crates/kesh-api/src/admin_backup/export.rs`, `auto_backup.rs`. Cherche **un autre chemin** qui produirait la même
   panique ou le même `Failed` et que l'écrivain de D6 ne fermerait pas. L'affirmation « la production classe
   correctement en `--release` » tient-elle (débordement de `file_end - self.stats.start`, refus « Large file
   option », `Drop` qui avale) ? L'explication de la variabilité (horodatages du montage, non `-j1`) est-elle
   compatible avec les deux occurrences rapportées (dont le commentaire de Guy du 2026-11-19) ?
4. **Inventaire par le symptôme** : pour chaque lot, la commande citée rend-elle ce que la fiche copie ? Relance-la.
   Cherche un site **que la commande ne voit pas** (autre forme du même défaut) : autre affichage de « Choisissez le
   compte de l’écart » sans sélecteur ; autre relecture de liste qui démonte une modale ; autre écrivain de test à
   cumul ; autre copie du dé-rapprochement. Chaque site trouvé est-il dans le périmètre ou hors périmètre avec une issue
   demandée ?
5. **D3 (#1262)** : la relecture différée est-elle correcte dans tous les chemins de fermeture (Annuler, Échap, succès,
   refus de la transaction, confirmation « déjà comptabilisée » `pendingGesture`) ? Une relecture double ou perdue
   est-elle possible ? Le cas « modale d'un autre genre ouverte » est-il couvert ?
6. **D4 (#1255)** : l'extraction préserve-t-elle l'ordre des verrous de chaque bras ? Les gardes statiques
   (`bank_account_closed_inventory.rs` — lis l'extracteur et les trois registres —, `letterings_lexical.rs`,
   `audit_acteur_garde.rs`) réagissent-elles comme la fiche le dit ? Une entrée de registre oubliée ?
7. **AC constatables** : chaque AC se constate-t-il par un test, une commande ou une capture nommés ? Un AC qui ne se
   vérifie pas est un finding.
8. **Preuve du rouge** : chaque test neuf rougit-il sans le correctif (mutation nommée, journal attendu dans
   `/home/gcorbaz/devel/kesh-gate-logs/e4814/`) ? Les filets déclarés (T13, T14, T8–T10 sur l'ancien code) sont-ils
   honnêtement déclarés ? Une assertion « vraie par construction » ?
9. **Écritures réelles** : le règlement groupé tient des écritures réelles en production. AC9 prouve-t-il que les
   règlements et écritures existants restent identiques (dé-rapprochement à comportement égal, groupe non touché
   intact) ? Manque-t-il un champ comparé (lettrages, audit par facture, numéro d'écriture inverse) ?
10. **Manuel, PDF compris** : `docs/manual/fr/user-manual.tex` § `sec:reglement-groupe` et le PDF aplati
    (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), contre le code. La fiche affirme qu'une
    phrase est déjà fausse (« le sélecteur s'affiche quand même ») : vérifie-le au code. Un autre passage du manuel
    (utilisateur ou d'administration — sauvegarde automatique, « disque plein ») décrit-il un comportement que la story
    touche ?
11. **E2E et recette visuelle** : T6 et T7 sont-ils jouables (interception `page.route`, fermeture par Échap pendant
    l'envoi) ? Les captures exigées couvrent-elles chaque état visible touché ?
12. **Splitting et migrations** : le compte de cinq modules est-il juste (règle : plus de cinq découpe) ? Aucune
    migration n'est-elle exigée ?
13. **Gates** : décrits selon le `CLAUDE.md` (exception `kesh-db` : gate complet même en cours de boucle ; `gate-ci`
    et E2E complet au dernier commit de code ; bases `kesh_*_e4814`) ?

## Interdits

- **Lecture seule.** Tu n'écris **aucun fichier** sauf ton rapport (ci-dessous). Pas d'`Edit`, pas de `Write` ailleurs.
- **Aucune commande qui écrit ou qui lance un gate** : ni `cargo` (build, test, nextest, clippy, fmt, run), ni `npm`,
  `npx`, `make`, ni `scripts/*` (dont `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`,
  `scripts/mem-guard.sh`, `scripts/recette-montee.sh`), ni `docker`, `docker compose`, `mariadb`, `sqlx`.
- Aucun `git` qui écrit (`commit`, `checkout`, `switch`, `stash`, `reset`, `merge`, `rebase`, `push`, `worktree`) ;
  `git show`, `git log`, `git grep`, `git diff` sont permis.
- Aucun `gh` qui écrit (`issue create|edit|comment|close`, `pr …`). `gh issue view` et `gh issue list` sont permis.
- Rien sous `~/kesh/` (production). Aucun autre worktree (`kesh-49-1c`, `kesh-49-1d` travaillent en parallèle).

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e4814/validation-p3.md`, puis rends son contenu.
Ne modifie ni `validation-p1.md` ni `validation-p2.md`.

- **Verdict chiffré** : nombre de findings par sévérité (CRITICAL / HIGH / MEDIUM / LOW), recomptés depuis ta liste.
- Chaque finding : identifiant `V3-n`, sévérité, axe, **prémisse citée** (`chemin:ligne` et la ligne copiée, ou la
  sortie de la commande), ce qui ne va pas, le correctif proposé pour la fiche. Un finding CRITICAL ou HIGH qui
  affirme une absence se prouve par un `grep -nF` cité.
- Pour chaque finding V1-1 à V1-11 et V2-1 à V2-9 : **fermé** ou **non fermé** (avec la prémisse qui le montre).
- Pour chaque finding neuf : dis s'il **naît d'une remédiation** (et de laquelle) — c'est la mesure du recyclage.
- **La liste des axes exercés et non exercés** (les treize ci-dessus, plus l'axe « remédiations P1 et P2 »), avec pour chaque axe non exercé sa raison. Un
  « 0 finding » sans cette liste ne compte pas comme passe.
- Les commandes que tu as lancées, dans l'ordre.
