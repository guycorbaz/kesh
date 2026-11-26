# Validation P4 (passe ciblée) — fiche 53-1a « La saisie ne passe pas deux fois » (#1355, P1)

**Modèle : Haiku. Passe ciblée, une seule lentille, contexte frais.** (`CLAUDE.md` § *La passe ciblée* : la P3 (Sonnet) a rendu un MEDIUM et onze LOW, tous remédiés par **un seul commit**, qui ne touche que des fiches ; la sévérité se déplace vers ce qu'on vient d'écrire.)

## Ce que tu relis — et rien d'autre

Le **seul commit de remédiation de la P3**, sur la branche `story/53-1-ecriture-unique` (worktree `/home/gcorbaz/devel/kesh-53-1`). L'orchestrateur te fournit **en entrée** le diff aplati, produit par :

```sh
git -C /home/gcorbaz/devel/kesh-53-1 diff 6d0f6b73e HEAD -- \
  _bmad-output/implementation-artifacts/53-1a-saisie-pas-deux-fois.md \
  _bmad-output/implementation-artifacts/53-1b-conversion-pas-deux-fois.md
```

(`6d0f6b73e` est la tête lue par la P3 ; `HEAD` est le commit de remédiation.) Rapport de la P3, pour savoir ce que chaque ajout prétend corriger : `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-1a-p3.md` (V3-1 à V3-12). Ne refais pas la validation de toute la fiche.

## La lentille unique : « ce qui vient d'être écrit est-il vrai au code ? »

Pour **chaque ligne ajoutée** du diff (`+`) :
1. Chaque citation `fichier:ligne`, chaque décompte, chaque commande neuve : est-elle exacte au code de la branche ? Relis la ligne citée ; recompte le décompte avec la commande écrite (p. ex. les **56** appels `create(` du module de tests de `crates/kesh-db/src/repositories/journal_entries.rs`, les trois collisions `:3860-3876`, `:3997-4010`, `:5337-5360`, la seed `crates/kesh-seed/src/pieges.rs:527`, le bouton « Valider » `JournalEntryForm.svelte:1266` et `crates/kesh-i18n/locales/fr-CH/messages.ftl:317`, les types `frontend/src/lib/features/journal-entries/journal-entries.types.ts:175`, `:184`, `:192`, `journal-entries.api.ts:73-77`, les renvois « étape 0-bis » `:600`, `:2197`, les routes `routes/journal_entries.rs:537`, `:576`, `:872`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md:324`, `docs/manual/fr/admin-manual.tex:2652-2664`, `crates/kesh-api/tests/audit_acteur_garde.rs` en-tête).
2. La remédiation corrige-t-elle vraiment le finding de la P3 qu'elle vise, sans en créer un neuf (contradiction avec une autre section de la fiche, AC sans test, mutation intuable, tâche qui manque) ?
3. La frontière 53-1a / 53-1b reste-t-elle nette (la lecture de `details.entry` à la 53-1b seule ; aucune autre trace de la conversion dans la 53-1a hors renvois et pièces communes) ?
4. Le manuel : si une ligne ajoutée parle du manuel (« Valider », « Passer à la 0.21.0 »), vérifie-la contre le `.tex` **et** le PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`), **vers ta seule sortie standard**.

## Garde-fou obligatoire pour tout CRITICAL ou HIGH

Avant de rendre un finding CRITICAL ou HIGH qui affirme l'**absence** d'un texte ou d'un code, ou la **présence** d'un anti-pattern, vérifie-le par `grep -nF "<chaîne exacte>" <fichier>` (`-F` obligatoire ; motif sur plusieurs lignes : une ligne discriminante, ou `grep -nFA <N>`) et **cite la sortie** dans le finding. Sans cette sortie, le finding est irrecevable. Le diff peut mêler des lignes de contexte : juge sur le fichier à `HEAD`, non sur le diff seul.

## Interdits

- **N'écris aucun fichier, nulle part**, et n'exécute **aucune commande qui écrit** : pas d'`Edit`/`Write`, **aucune redirection `>` ni `>>`, pas de `tee`**, pas de `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git restore`, `git mv`, `git push`, pas de `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh` ni aucun script du dépôt, pas de `cargo`, pas de `npm`, pas de `python3`, pas de `docker` ni `docker compose`, pas de `sqlx`, pas de `mariadb`/`mysql`, pas de `curl`, pas de `make`.
- **Jamais `pkill`, `killall` ni `kill`** (ni par motif, ni par PID).
- Aucune commande `gh`.
- Commandes permises : `git diff`, `git show`, `git log`, `git grep`, `grep` (dont `grep -r` : le pathspec `git grep … -- 'crates/*/tests'` ne désigne rien dans ce dépôt), `sed -n`, `awk` (lecture), `cat`, `head`, `tail`, `wc`, `ls`, `pdftotext … -` vers la sortie standard.

## Rapport attendu

En français. Pour chaque finding : identifiant `V4-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), ligne du diff visée, constat **avec la ligne de code ou de fiche lue citée** (et la sortie `grep -nF` pour un CRITICAL ou un HIGH), correctif proposé. Puis le sort de V3-1 à V3-12 (remédié / partiellement / non). Puis le décompte par sévérité, et **la liste des axes exercés et non exercés** (les quatre points ci-dessus ; un axe non exercé dit pourquoi). Un « 0 finding » sans cette liste ne compte pas comme passe. L'orchestrateur range le rapport sous `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-1a-p4.md`.
