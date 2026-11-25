# Prompt — validation P3 de la fiche 49-5, passe **ciblée** sur la remédiation de la P2

Tu fais une **passe ciblée** de validation de la fiche de la story de solde de l'**Epic 49** de Kesh (comptabilité suisse ; Rust/Axum,
MariaDB, SvelteKit), release **0.20.0** (vendredi 2026-11-27). **Une seule lentille, contexte frais, modèle Sonnet.** Langue du
rapport : français. Kesh tient des livres réels : rien d'existant ne doit changer ; aucune migration.

Worktree `/home/gcorbaz/devel/kesh-49-solde`, branche `story/49-5-solde-dette-epic`. Code lu à `59fb28712`.
**Objet de la passe : le seul dernier commit de la branche** — la remédiation de la validation P2 (`git show HEAD`,
`git show --stat HEAD`). Le reste de la fiche a passé deux passes complètes (P1 Sonnet 0/1/2/9, P2 Opus 0/0/1/8) : ne le relis que
là où le commit le touche ou s'y appuie. *Motif : la sévérité se déplace vers ce qu'on vient d'écrire.*

## À lire

1. `git show HEAD` (le diff entier) ; puis, dans la fiche `_bmad-output/implementation-artifacts/49-5-solde-dette-epic.md`, les
   sections qu'il touche : lot C (point 2 du « Ce qui se lit au code », D-C1, D-C2, AC-C2, AC-C3, tableau des tests), *Issues
   écartées*, *Issues à ouvrir par l'orchestrateur* (I-6, I-7, nouveau corps de #1499), Change Log.
2. Le rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p2.md` (findings V2-1 à V2-9).
3. Le code que le commit cite, à `59fb28712` : `scripts/tests/recette-montee.test.sh` (`:11`, `:19-20`, `:385-395`, `:864-873`),
   `scripts/recette-montee.sh` (`:86-97`, `:153-157`), `crates/kesh-db/tests/recette_montee_script.rs`,
   `docs/agents-orchestration.md:189-201`, `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:864-874`,
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte:1405-1470`.
4. `CLAUDE.md` du worktree : § *Review Iteration Rule* (*La passe ciblée*, *Inventorier les sites NON RÉSOLUS*), *Issue Tracking
   Rule*, *Priorités des défauts*.

## La lentille : le remède de la P2 est-il juste, et n'a-t-il rien cassé ?

Axes (chacun **exercé**, ou **déclaré non exercé** avec sa raison) :

- **C1 — V2-1** : T5 tel qu'écrit (cas i à iv, dossiers sous `$WORK`, `KESH_RECETTE_TEST_VERROUS` en affectation préfixée à un
  appel de **fonction** bash) laisse-t-il vraiment deux copies simultanées vertes ? Une ressource partagée reste-t-elle entre deux
  copies (ports 24310-24390 vérifiés à `ss` par les deux, fichiers `<prêt>`, `T5_PIDS`) ? Les attentes sont-elles toutes relatives ?
  M6 rougit-elle T5 (iv), et seulement lui ?
- **C2 — V2-2** : les tenants de fond (`flock -n <fichier> bash -c 'touch …; sleep 60'`) sont-ils synchronisés, vérifiés, tués et
  attendus ; le `trap … EXIT` étendu reste-t-il correct sous `set -euo pipefail` et avec le `rm -rf "$WORK"` existant (`:20`) ; un
  tenant peut-il retenir la sortie lue par l'enveloppe Rust (`timeout -k 10 600`) ?
- **C3 — V2-3** : le relâchement de `p` quand `p + 1` est refusé et la garantie reformulée du dossier par utilisateur (dossier
  squatté) sont-ils exacts et sans contradiction avec le point 2 de D-C1 (`DOSSIER_VERROUS=${KESH_RECETTE_TEST_VERROUS:-…}`) ?
- **C4 — V2-4, V2-5** : la prise de T6 (`flock /tmp/station-heavy.lock flock …/.gate.lock scripts/mem-guard.sh …`) est-elle celle
  du § 5.1 ; les lignes `:86-97`/`:96` de `recette-montee.sh` sont-elles justes ?
- **C5 — V2-6 à V2-9** : #1363 bien classée ; I-6 et I-7 exactes au code (`chemin:ligne` copiés), labels `bug`, `triage`, P4 ;
  le nouveau corps de #1499 est-il autonome, exact, et cohérent avec I-1 et avec « une issue, une release » ?
- **C6 — propagation** : un symptôme corrigé survit-il ailleurs dans la fiche (greper la **valeur** : `Projet`, `24310`, `24312`,
  `:73-80`, `à sa fermeture`, `.gate.lock scripts/mem-guard`, `Six issues`) ? Le Change Log dit-il juste (décomptes recomptés) ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun
`cargo`, `npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `flock`, `bash scripts/…`, aucun `scripts/*` (en particulier
ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni
`scripts/recette-montee.sh`, ni `scripts/tests/recette-montee.test.sh`), aucun script de `kesh-gate-logs/`. Aucun `git commit`,
`git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git reset`, `git worktree`, `git tag`. Aucun `gh` qui écrit
(`issue create/edit/comment/close`, `pr …`, `project …`, `api` avec `-X POST|PATCH|DELETE` ou une mutation GraphQL) ; une lecture
`gh` au besoin, **une minute au moins entre deux requêtes**. Rien sous `~/kesh/` (production). `git show`, `git grep`, `git log`,
`sed -n`, `grep` et `python3 -I` en lecture sont admis ; une sonde `bash -c` qui n'écrit rien et ne lance aucun script du dépôt est
admise pour éprouver une sémantique de bash (à citer).

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p3-ciblee.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V3-n`, sévérité, axe, prémisse **citée** (`chemin:ligne`, ligne copiée, ou ligne du diff), remède ;
- la liste des **axes exercés et non exercés** (C1 à C6), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette liste
  ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
