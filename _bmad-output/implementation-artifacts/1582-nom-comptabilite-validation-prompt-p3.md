# Prompt — validation P3 ciblée de la fiche `1582-nom-comptabilite` (#1582), une lentille, sur le seul commit de remédiation de la P2

Tu relis, **en lecture seule**, la **remédiation de la validation P2** de la fiche de story `1582-nom-comptabilite` de Kesh (frontend
seul, aucune migration, hors epic, visée v0.20.0). **Passe ciblée : une seule lentille, ci-dessous. Tu ne rouvres ni la P1 ni la P2.**

- **Modèle : Haiku. Contexte frais.** Langue du rapport : français.
- **Entrée** : le diff **aplati** de la remédiation, **hors fusion de `main`**, des seuls fichiers de la fiche :
  `/home/gcorbaz/devel/kesh-gate-logs/s1582/p3-diff.patch` (lecture : `cat`), produit par
  `git diff 93a7f07a5 -- _bmad-output/implementation-artifacts/1582-nom-comptabilite.md _bmad-output/implementation-artifacts/1582-nom-comptabilite-validation-prompt-p2.md`
  et la ligne `1582-nom-comptabilite:` de `sprint-status.yaml`. `93a7f07a5` = fin de la remédiation de la P1. La fiche entière :
  `/home/gcorbaz/devel/kesh-1582/_bmad-output/implementation-artifacts/1582-nom-comptabilite.md`. Le rapport P2 :
  `/home/gcorbaz/devel/kesh-gate-logs/s1582/validation-p2.md`.
- **Lentille** : « la remédiation de la P2 est-elle exacte, complète et sans défaut neuf ? » — chaque affirmation **nouvelle** du diff est
  vraie au code, aucune mention périmée ne subsiste, aucun chiffre ni renvoi n'est devenu faux.

*Annotation datée du 2026-11-25 (après la passe) : les nombres « 43 à titre, 12 sans » du point 1 et la méthode `<title>` avec le `>` étaient faux ; la passe l'a établi (55 pages, 44 à titre, 11 sans). Voir le Change Log de la fiche. Ce prompt est gardé tel qu'il a été donné.*

## Garde-fou `grep -nF` (obligatoire)

Pour **tout** finding CRITICAL ou HIGH qui affirme l'absence d'un texte attendu ou la présence d'un défaut, donne la commande
`grep -nF "<chaîne>" <fichier>` (`-F` obligatoire) et sa sortie copiée ; un finding sans cette preuve est écarté. Un MEDIUM qui reproche à la
fiche l'absence d'un livrable que la fiche **prescrit** pour le développement (code, test, capture) est une erreur de catégorie : le livrable
n'est pas encore écrit. Cite toujours `chemin:ligne`.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo`, **ni** `npm` / `npx`, **ni** `make`, **ni** aucun `scripts/*`
(dont `scripts/prepare-release.sh`), **ni** `docker`, `mariadb`, `sqlx`, `curl`, `wget`, **ni** `git commit`, `checkout`, `stash`, `restore`,
`reset`, `merge`, `rebase`, `fetch`, `push`, `worktree`, **ni** `gh` en écriture, **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute redirection vers
un fichier, **ni** `pkill`, `killall`, `kill` par motif, **ni** `python3` ou autre interpréteur. Rien sous `~/kesh/`. Permis : `cat`, `grep`,
`sed -n`, `ls`, `wc`, `find` (sans `-exec` ni `-delete`), `git log`, `git show`, `git diff`, `git grep`, `gh … view|list`.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s1582/validation-p3.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Points à contrôler (chacun exercé ou déclaré non exercé)

1. **Décompte des pages (V2-4)** : le constat 7 dit 55 pages, 43 à titre (39 par leur `<title>`, 4 par un composant), 12 sans. Recompte :
   `find "frontend/src/routes/(app)" -name "+page.svelte" | wc -l` ; `grep -rlF "<title>" "frontend/src/routes/(app)" | wc -l` ;
   `grep -rl "<svelte:head" frontend/src/lib` (composants qui portent un titre : `AdminBackupPanel`, `AdminRestorePanel`,
   `ProviderReviewPage`) ; que les 12 pages listées n'importent aucun de ces composants. AC3, étape 0 et le texte de l'issue P4 portent les
   mêmes nombres (`grep -nF "55" `, `grep -nF "43 sur 55"`, `grep -nF "12 pages"`).
2. **Retrait de l'E2E « personne physique » (V2-1)** : la ligne citée existe (`crates/kesh-api/src/routes/onboarding.rs:542-543`), la
   fiche dit « vérifiée à la lecture, non testée » à **tous** ses sites (`grep -nF "542-545" ` ne doit plus rien donner de périmé ;
   `grep -nF "déjà testé"` vide), le texte de l'issue `technical-debt` P3 est exact (lignes `onboarding_path_b_e2e.rs:1218-1296` : relis-les),
   et la story reste frontend seul (AC12).
3. **V2-2, V2-3, V2-5, V2-6** : « Sept E2E restent » ; l'en-tête de la fiche (ligne 5) et la ligne de `sprint-status.yaml` ne disent plus que la P1
   est à lancer ; la tâche 5 n'a plus la condition `authState.isAuthenticated` (`grep -nF "isAuthenticated"` : ne reste que l'explication) ;
   M12 est dans la table des mutations, sans test, déclarée.
4. **Aucun défaut neuf** : numéros de mutations (M1 à M12 uniques), d'AC (AC1 à AC12 définies), d'E2E (1 à 7, renvois M1/M5/M6/M8 exacts),
   références croisées (« issue à ouvrir », « Change Log », `validation-prompt-p3.md`), aucun mot-clé de fermeture (`closes`, `fixes`,
   `resolves` et variantes : `grep -niE '\b(close[sd]?|fix(e[sd]?)?|resolve[sd]?) #[0-9]'` sur la fiche), français correct.
5. **Hors lentille (ne pas traiter)** : le mécanisme du titre, D1 à D11, les tests, le manuel : déjà jugés aux P1 et P2.

## Rapport

Pour chaque finding : `V3-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), constat avec la ligne lue citée, conséquence, remède. En fin de
rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW** recompté depuis tes findings ; **axes exercés** et **non exercés** (avec la raison) ; les
commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
