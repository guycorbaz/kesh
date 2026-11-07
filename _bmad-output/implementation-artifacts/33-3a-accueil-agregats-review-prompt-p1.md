# Revue de code P1 — Story 33-3a (les agrégats de l'accueil), trois lentilles B, E, A

Contexte frais. Modèle : Sonnet ou Opus, jamais Haiku pour une première passe. Une lentille par agent.

- **Worktree** : `/home/gcorbaz/devel/kesh-e333`, branche `story/33-3-accueil`. **Base** : `bdc3faa4`. Le commit de code de la story est
  `97162a47` ; la 33-3b le suit sur la même branche (prompt à part). Diff : `git show 97162a47`, plus les comptes rendus de la fiche
  au dernier `HEAD`.
- **Fiche** : `_bmad-output/implementation-artifacts/33-3a-accueil-agregats.md`. **Choix** : `e33-choix-autonomes-e332.md` (C-e332-8 à 15).
- **Journaux** : `/home/gcorbaz/devel/kesh-gate-logs/e333/rouges.log`, `/home/gcorbaz/devel/kesh-gate-logs/e333a/mut/`.
- **Rapports** (un par lentille, seul fichier écrit) : `/home/gcorbaz/devel/kesh-gate-logs/e333a-review-p1-B.md`, `…-E.md`, `…-A.md`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `python3`, ni `scripts/prepare-release.sh`, ni aucun script
du dépôt ; ni `git checkout`, `reset`, `stash`, `push`, `tag` ; ni `gh` qui écrit ; ni `rm`. **Aucune redirection, aucun fichier temporaire** hors du rapport.
Dans ce shell, `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence porte une sortie `grep -nF` copiée.
Aucune donnée réelle.

## Les lentilles

- **B (Blind Hunter)** : le diff seul, sans la fiche. Défauts de code, SQL, arrondi, ordre, sérialisation.
- **E (Edge Case Hunter)** : le diff et le code lu autour. Chaque branche et chaque borne : exercice de un jour, à cheval sur un changement d'année, écriture datée hors de son exercice
  (`INCONSISTENT_ENTRIES` existe), paire dont l'origine et la contre-passation sont dans deux exercices, compte de charges archivé, solde nul, égalité, base très petite, `DECIMAL(19,4)` et `share`, plusieurs lignes d'une même écriture sur le même compte, société voisine, lecture sur une connexion `READ ONLY`.
  Chercher tout chemin qui pose `reverses_entry_id` ou qui écrit une contre-passation sans miroir : la prémisse « une paire vaut zéro, compte par compte » en dépend.
- **A (Acceptance Auditor)** : la fiche contre le code et les tests, AC1 à AC19, mutations M1 à M18 (les 22 lignes de `kesh-gate-logs/e333a/mut/synthese.log`, dont M6b, ajoutée) : chaque AC a-t-il un test qui rougit pour la bonne raison ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Axes à exercer par toutes

1. **L'invariant** C-e332-10 : `totalRevenues`/`totalExpenses` égalent le compte de résultat, y compris pour une écriture à date anormale. Lire `income_statement.rs` contre `homepage_summary.rs`.
2. **Le manuel, PDF compris** : `docs/manual/fr/user-manual.tex`, § *Revenus, dépenses et principaux postes*, et `docs/manual/fr/user-manual.pdf`, aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), contre le code : chaque phrase qui décrit ce qui est écarté ou compté est-elle vraie ? `docs/api-external.md`, paragraphe *La synthèse de l'accueil*, de même. Le manuel est porté par la 33-3b, mais il décrit ce code-ci.
3. Le motif des services (`services/mod.rs`, R1 à R8) : exigence, lecture sans trace, inventaire, test de refus, `tests/services_motif.rs`.
4. Performance : deux requêtes groupées, sur les index existants. À dire si un `NOT EXISTS` coûte trop sur un exercice chargé (raisonné, non mesuré).

## Le rapport

Décompte CRITICAL / HIGH / MEDIUM / LOW, recompté. Chaque constat : fichier:ligne, sortie citée, conséquence, remédiation proposée. **La liste des axes exercés et de ceux qui ne l'ont pas été** : une passe qui ne la déclare pas ne compte pas.
