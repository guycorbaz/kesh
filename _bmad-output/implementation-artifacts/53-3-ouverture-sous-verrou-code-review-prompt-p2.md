# Prompt — revue de code P2 de la story 53-3 (#513), passe **ciblée**, lentille unique, Haiku

Tu relis **un seul commit** de la story **53-3** de Kesh (Rust/Axum, MariaDB) : la remédiation de la revue de code P1 (Opus, 0/0/1/6). Passe **ciblée**, contexte frais,
modèle **Haiku**, **une lentille** (ci-dessous). Langue du rapport : français. Kesh est en production ; le commit ne change que des **tests**, des **commentaires** et la **fiche** :
aucun code de production n'est modifié.

Commit : `371cd4ac7` (worktree `/home/gcorbaz/devel/kesh-53-3`, branche `story/53-3-ouverture-sous-verrou`). **Le diff à lire est le fichier aplati
`/home/gcorbaz/devel/kesh-gate-logs/e533/cr-p2-diff.patch`** (`git show 371cd4ac7`, 251 lignes) : lis-le en entier ; ne reconstitue pas le diff toi-même.
Rapport de la P1 (ce que le commit doit remédier) : `/home/gcorbaz/devel/kesh-gate-logs/e533/code-review-p1.md` (R7-1 MEDIUM ; R7-2, R5-1, R5-2, R2-1, R4-1, R12-1 LOW).

## La lentille (unique) : « les gardes lexicales du commit prouvent-elles quelque chose ? »

Fichier : `crates/kesh-db/tests/opening_types_sous_verrou_lexical.rs` (nouveaux tests : `la_lecture_des_types_precede_le_recensement`,
`sonde_negative_l_appel_deplace_apres_le_recensement_rougit`, `sonde_negative_l_extracteur_est_borne_a_la_fonction`). Pour chacun :
1. L'assertion est-elle **vraie par construction** (les deux côtés sortent du même texte ou du même extracteur, un `replace` global, une condition tautologique) ?
2. La **sonde négative** porte-t-elle sur une copie où **seul l'ordre change** (l'appel est déplacé, non supprimé) ? Les `assert_eq!(…matches(…).count(), 1)` et `assert_ne!` garantissent-ils que la sonde a bien déplacé quelque chose ?
3. L'ordre est-il évalué sur le **texte masqué** du corps de `create_opening_entry` (commentaires et chaînes exclus), de sorte qu'un doc-comment qui nomme les deux fonctions ne fausse pas la garde ?
4. Un cas non couvert qui ferait passer la garde à tort : l'appel `count_by_company(` plus d'une fois dans le corps (le test prend la **première** occurrence), une autre lecture ordinaire avant la lecture des types sous un autre nom (limite **écrite** dans la doc du test : à signaler seulement si la doc ne la dit pas) ?
5. Les nouveaux tests compilent-ils sans `#[allow]` superflu, sans doublon de nom avec un test existant du fichier (`grep -nF "fn " crates/kesh-db/tests/opening_types_sous_verrou_lexical.rs`) ?

Hors lentille (ne les juge pas) : le correctif de production de `607fb31e8`, la conception des verrous, les mutations M1 à M12 (déjà jugées), le manuel, l'E2E.
Les doc-comments et la fiche du commit : tu **vérifies seulement** que chaque phrase corrigée dit ce que fait le code (une ligne lue citée par phrase) — pas de relecture complète.

## Garde-fou `grep -nF` (obligatoire)

Pour tout finding CRITICAL ou HIGH qui affirme l'**absence** d'un code attendu ou la **présence** d'un anti-pattern, vérifie **avant de l'écrire** :
`grep -nF "<chaîne exacte>" <fichier>` (`-F` obligatoire ; une ligne discriminante pour un motif sur plusieurs lignes) et cite la ligne rendue. Un finding réfuté est écarté et consigné
comme faux positif. Le diff est aplati sur **un seul commit** : n'indexe pas sur d'autres commits.

## Interdits

**Lecture seule.** N'écris **aucun** fichier du worktree ni du dépôt principal, sauf ton rapport. **Aucune redirection de shell (`>`, `>>`, `tee`) vers autre chose que ton rapport.**
Aucun `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `mysql`, `sqlx`, `curl`, `python3` autre que `python3 -I` en lecture ; aucun `scripts/*` — en particulier ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`. Aucun `git commit`, `git push`, `git checkout`,
`git switch`, `git stash`, `git merge`, `git reset`, `git rebase`, `git worktree`, `git tag`, `git fetch`. **Aucun `pkill`, `killall`, ni `kill` par motif.** Aucun `gh`. Rien sous `~/kesh/`.
Une commande refusée t'arrête : signale-le.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e533/code-review-p2.md` :
- verdict chiffré CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`P2-n`), sévérité, `chemin:ligne` et ligne copiée, ce qui est faux, correction proposée ;
- les points de la lentille **exercés** (1 à 5) et **non exercés** avec la raison — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les phrases de doc-comments et de fiche vérifiées (une ligne citée chacune).

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
