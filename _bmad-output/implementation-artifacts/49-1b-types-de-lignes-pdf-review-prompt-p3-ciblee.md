# Revue de code P3 ciblée — Story 49-1b (#1242)

*Passe **ciblée** : une seule lentille, contexte frais, modèle **Haiku** (la P2 était Sonnet). Objet : **le seul commit**
`c69162665` (`fix(49-1b) : cargo tree verrouillé, sonde positive resserrée`), remédiation du LOW L1 de la P2. Lecture
seule.*

## Ce que tu relis

1. **Le diff aplati du commit, et lui seul** :
   `git -C /home/gcorbaz/devel/kesh-49-1b show --format= c69162665 -- crates/kesh-api/tests/configuration_transmise.rs`
   (un seul fichier modifié ; vérifie-le par `git -C /home/gcorbaz/devel/kesh-49-1b show --stat --format= c69162665`).
2. **Le finding qu'il traite** : `/home/gcorbaz/devel/kesh-gate-logs/e491b/review-p2.md`, § *L1*.
3. **Le journal qui l'a joué** : `/home/gcorbaz/devel/kesh-gate-logs/e491b/l1-p2.log`.
4. Pour le contexte, les fonctions touchées dans le fichier à HEAD :
   `la_feature_de_test_n_entre_pas_en_production`, `features_du_paquet`, `s_features_du_paquet`, `arbre_des_features`.

## Lentille unique : la remédiation fait-elle ce qu'elle dit, sans rien casser ?

- `--locked` est-il bien passé à `cargo tree`, avec `--offline` ? Un `Cargo.lock` en retard rougit-il au lieu d'être réécrit ?
- `features_du_paquet` lit-elle exactement les features de la ligne d'un paquet de `cargo tree -f "{p} {f}"` ? Cas :
  ligne suivie de ` (*)`, paquet sans feature, liste à plusieurs features (`a,b`), paquet dont le nom en contient un autre
  (`autre-kesh-qrbill`), paquet racine (sans espace avant le nom), chemin contenant une parenthèse.
- Le contrôle négatif rougit-il si `kesh-qrbill` porte la feature dans les dépendances normales, et seulement dans ce
  cas ? Peut-il passer à vide (`kesh-qrbill` absent de l'arbre : l'assertion de montage le voit-elle) ?
- La sonde positive ne reconnaît-elle plus que la ligne de `kesh-qrbill` portant la feature ?
- `s_features_du_paquet` prouve-t-il ce qu'il dit ? Ses deux côtés sortent-ils du même extracteur (vrai par construction) ?
- Le journal `l1-p2.log` montre-t-il le test vert **sous `cargo test`** (un processus par binaire), et pas seulement sous
  nextest ?

Ne relis **pas** le reste de la story : il a été revu en P1 et en P2.

## Haiku : vérification obligatoire

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu ou la présence d'un anti-pattern, copie la
sortie de `/usr/bin/grep -nF "<chaîne>" crates/kesh-api/tests/configuration_transmise.rs` qui le prouve, faute de quoi il
ne compte pas.

## Interdits

Lecture seule. N'exécute **aucune** de ces commandes : `cargo` (build, test, run, fmt, clippy, tree, metadata, nextest),
`npm`, `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`,
`kesh-gate-logs/e491b/mut/mutations.py`, `kesh-gate-logs/e491b/l1-p2.sh`, `kesh-gate-logs/e491b/gate-cible.sh`,
`kesh-gate-logs/e491b/gates-review.sh`, `kesh-gate-logs/e491b/capture-e491b.sh`, `git commit`, `git checkout`, `git restore`,
`git stash`, `git reset`, `git worktree`, `rm`, `mv`, `cp`, `touch`, `sed -i`, `docker`, ni redirection vers un fichier, ni
fichier temporaire. Seule écriture permise : ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep`
ou `git grep`.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e491b/review-p3-ciblee.md`. Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW),
fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Termine par la liste des **axes exercés et
non exercés** (un « 0 finding » sans elle ne compte pas) et le décompte par sévérité.
