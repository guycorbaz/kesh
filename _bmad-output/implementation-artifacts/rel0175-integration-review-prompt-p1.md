# Revue ciblée — le code écrit à l'intégration de la 0.17.5 (une lentille, lecture seule)

Contexte frais, français soigné. Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11) ; la 0.17.4 tient des
livres réels en production. Les treize stories de l'Epic 33 ont chacune clos leur boucle de revue **sur leur branche**.
Ce que personne n'a relu, c'est **ce qui a été écrit en les fusionnant** : les résolutions de conflit dans du code, les
retouches du commit de mise en ordre, et un correctif de test né du gate en mode CI. C'est ton seul périmètre.

Worktree `/home/gcorbaz/devel/kesh-rel0175`, branche `release/0.17.5`, base `origin/main` = `d9b53d38`. Lis par
`git show` / `git diff`, jamais en supposant le fichier sur disque à jour.

## Le périmètre, exactement

1. **Les résolutions de conflit**, par `git show --remerge-diff <fusion>` (diff entre la fusion automatique avec marqueurs
   et la résolution retenue) :
   - `1dbee0a7` (33-12) : `crates/kesh-qrbill/src/validation.rs` — `first_qr_refused_char` (33-5) délègue désormais à
     `est_caractere_qr` (33-12a), `is_six_annex_c_char` disparaît ; **conséquence déclarée : l'adresse de la société admet
     « & »**. Est-ce voulu par les deux fiches (33-5 : `_bmad-output/implementation-artifacts/33-5-adresse-societe.md` ;
     33-12a : `33-12a-*.md`) ? Les messages d'erreur, les tests des deux stories, le manuel, le CHANGELOG et le frontend
     (contrôle côté client du même jeu de caractères, s'il existe) disent-ils la même chose que le code résolu ?
   - `c2cefb27` (33-3) : `frontend/src/lib/features/homepage/MonthlyBarsCard.svelte`, `homepage-charts.ts`.
   - `81f126a6` (33-4) : `frontend/src/lib/features/open-items/open-items.ts`.
   - `519cd6cf` (33-7) : `crates/kesh-api/tests/audit_route_registry.rs` et le manuel d'administration (« 138 + 4 + 3 = 145 ») :
     recompte depuis le registre, non depuis le texte.
   - `181da8b4`, `26a53935`, `8a68006c`, `86c270ec` : gardes de tests (`i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts`,
     `due-dates-page.test.ts`) — une garde recomptée doit compter, non recopier.
2. **`bbcb06dd`** (mise en ordre), hors `_bmad-output/` et CHANGELOG : `git show bbcb06dd -- crates frontend`.
3. **`19500f88`** (correctif du gate en mode CI, `crates/kesh-api/tests/dates_bornes_e2e.rs`) : la cause (« un compteur
   commun au processus portait le taux de TVA des tests au-delà de 100 sous `cargo test` ») est-elle la vraie ? Le correctif
   la supprime-t-il, ou la déplace-t-il (un autre test du même binaire partageant ce compteur) ? Le test prouve-t-il encore
   ce que son nom dit ?

Rapport de l'intégration et journaux : `/home/gcorbaz/devel/kesh-gate-logs/rel0175/` (en particulier `gate-ci.log`,
`gate-ci-journal.log` — premier passage rouge —, `gate-ci-2.log`, `gate-backend-2.log`, `e2e-run.out`), registre
`_bmad-output/implementation-artifacts/e33-choix-autonomes-rel0175.md`.

## Axes obligatoires

- Pour chaque résolution : **ce que chaque côté voulait** (lu dans les deux parents, `git show <fusion>^1:<f>` et
  `<fusion>^2:<f>`), et si la résolution garde les deux intentions ; ce qu'elle perd en silence.
- Le **manuel** (`docs/manual/fr/*.tex` et le PDF : `git show HEAD:docs/manual/fr/<f>.pdf | pdftotext - - | tr '\n' ' ' |
  tr -s ' '`) dit-il vrai du code résolu, en particulier sur les caractères admis dans l'adresse de la société ?
- Propagation : greper chaque symptôme (le nom `is_six_annex_c_char`, le jeu de caractères, « & ») sur tout le dépôt.

## Contraintes

⛔ Ton rapport s'écrit à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/rel0175-review-p1.md`, avec l'outil
Write. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire ; **aucune compilation, aucune exécution de
test ni de script** (un test shell de cette release a fait tomber la station le 2026-11-07) ; aucune base, aucun `docker` ;
`scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `git merge` et tout `gh` qui écrit sont interdits.
`grep` est une fonction vers ugrep : `/usr/bin/grep` ou `git grep`. Ne lis rien sous `~/kesh/`, `~/kesh-test/`,
`/home/gcorbaz/travail/`.

Le rapport porte : **les axes exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une
sortie `git grep -nF` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
