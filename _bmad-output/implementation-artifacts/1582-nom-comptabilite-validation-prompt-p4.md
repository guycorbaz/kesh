# Prompt — validation P4 ciblée de la fiche `1582-nom-comptabilite` (#1582), une lentille, sur le seul commit de remédiation de la P3

Tu relis, **en lecture seule**, la **remédiation de la passe ciblée P3** de la fiche de story `1582-nom-comptabilite` de Kesh (frontend
seul, aucune migration, hors epic, visée v0.20.0). **Passe ciblée : une seule lentille. Tu ne rouvres ni la P1, ni la P2, ni la P3.**

- **Modèle : Sonnet. Contexte frais** (la P3 était Haiku : deux passes ciblées de suite changent de modèle). Langue du rapport : français.
- **Entrée** : le diff **aplati** du seul commit de remédiation de la P3 (la fiche, les prompts P3 et P4), hors fusion :
  `/home/gcorbaz/devel/kesh-gate-logs/s1582/p4-diff.patch` (lecture : `cat`), produit par
  `git diff 44805ebe9 -- _bmad-output/implementation-artifacts/1582-nom-comptabilite.md _bmad-output/implementation-artifacts/1582-nom-comptabilite-validation-prompt-p3.md`
  (`44805ebe9` = fin de la remédiation de la P2). Fiche entière :
  `/home/gcorbaz/devel/kesh-1582/_bmad-output/implementation-artifacts/1582-nom-comptabilite.md`. Rapport P3 :
  `/home/gcorbaz/devel/kesh-gate-logs/s1582/validation-p3.md`.
- **Lentille** : « la remédiation de la P3 est-elle exacte, complète et sans défaut neuf ? » — chaque affirmation **nouvelle** du diff est vraie
  au code, la **méthode de comptage écrite** donne bien les nombres écrits **quand on la rejoue**, aucune mention périmée ne subsiste.

## Garde-fou `grep -nF`

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un texte ou la présence d'un défaut, donne la commande `grep -nF "<chaîne>" <fichier>`
(`-F` obligatoire) et sa sortie copiée ; sans cette preuve, le finding est écarté. Cite toujours `chemin:ligne`.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo`, **ni** `npm` / `npx`, **ni** `make`, **ni** aucun `scripts/*`
(dont `scripts/prepare-release.sh`), **ni** `docker`, `mariadb`, `sqlx`, `curl`, `wget`, **ni** `git commit`, `checkout`, `stash`, `restore`,
`reset`, `merge`, `rebase`, `fetch`, `push`, `worktree`, **ni** `gh` en écriture, **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute redirection vers
un fichier, **ni** `pkill`, `killall`, `kill` par motif, **ni** `python3` ou autre interpréteur. Rien sous `~/kesh/`. Permis : `cat`, `grep`,
`sed -n`, `ls`, `wc`, `find` (sans `-exec` ni `-delete` ; `xargs -0 grep` en lecture permis), `git log`, `git show`, `git diff`, `git grep`,
`gh … view|list`.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s1582/validation-p4.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Points à contrôler (chacun exercé ou déclaré non exercé)

1. **Recompte les pages toi-même, avec la méthode écrite au constat 7** (points (a) à (d)) — sans te fier aux nombres de la fiche : 55 pages ;
   15 sans `<title` dans leur fichier ; 4 d'entre elles tiennent leur titre d'un composant (`grep -rlF "<svelte:head" frontend/src/lib`, puis les
   imports) ; **44 à titre, 11 sans**. Vérifie que `reconciliation/rules/+page.svelte:65-67` a bien son titre sur plusieurs lignes, et que la
   méthode, **rejouée mot à mot**, donne 15 puis 11 (elle ne doit pas être ambiguë ni donner un autre nombre). Les 11 pages listées à
   l'issue P4 et au constat 7 sont les mêmes ; chacune n'a ni `<title`, ni `<svelte:head`, ni import d'un des trois composants.
2. **Propagation du symptôme** (grep de la **valeur**, tout le dépôt de la fiche et des prompts de la story) : `\b43\b`, `\b44\b`,
   `\b12 pages\b`, `\b11 pages\b`, `Douze`, `Onze`, `reconciliation/rules` : chaque occurrence est cohérente avec 44 / 11, ou est une mention
   **historique annotée** (entrée du Change Log de la P2, prompt P3 annoté) — jamais un nombre périmé laissé vivant (étape 0, AC3, constat 7,
   issue P4, Change Log vivant).
3. **V3-2** : `M1 à M12` partout où la borne s'écrit ; `grep -nE 'M1 à M1[0-9]'` sur la fiche et sur les prompts **non historiques** ; la table
   des mutations contient M1 à M12 une fois chacune, dans l'ordre (M11 avant M12), et son en-tête dit « sauf M11 et M12, déclarées sans test ».
4. **V3-3 et V3-4** : la phrase sur PayPal est juste (`git show 1e23edb55:"frontend/src/routes/(app)/reconciliation/paypal/+page.svelte" |
   grep -cF "<title>"` ; `ProviderReviewPage.svelte:310` aujourd'hui) ; plus d'incise sur « les deux cartes SVG ».
5. **V3-6** : l'issue `technical-debt` P3 ne demande plus ce qui existe : relis `crates/kesh-api/tests/onboarding_path_b_e2e.rs:1296-1310` ;
   le cas « nom absent, prénom présent » n'est affirmé nulle part (`grep -rnF` sur `crates/kesh-api/tests`).
6. **Aucun défaut neuf** : numéros de renvoi (`validation-prompt-p4.md`), AC1 à AC12, E2E 1 à 7, aucun mot-clé de fermeture dans la fiche ni dans
   les messages de commit (`git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd]?)?|resolve[sd]?) #[0-9]'`), français correct.
7. **Hors lentille (ne pas traiter)** : le mécanisme du titre, D1 à D11, les tests, le manuel : déjà jugés.

## Rapport

Pour chaque finding : `V4-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), constat avec la ligne lue citée, conséquence, remède. En fin de rapport :
décompte **CRITICAL / HIGH / MEDIUM / LOW** recompté depuis tes findings ; **axes exercés** et **non exercés** (avec la raison) ; les commandes
lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
