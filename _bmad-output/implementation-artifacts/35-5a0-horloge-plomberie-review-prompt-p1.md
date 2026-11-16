# Prompt — revue de code P1 de la Story 35-5a0 (refs #929, #1022, #907), trois lentilles, première passe complète

Tu relis la **Story 35-5a0** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **la plomberie d'une horloge injectable, sans changement de comportement** — trait `Horloge` et
`HorlogeFixe` dans `kesh-core`, `HorlogeSysteme` et `horloge_depuis`/`horloge_de_test` dans `kesh-api`, champ `AppState.horloge`,
`Lecture { pool, horloge }`, câblage du MCP ; `kesh-db` **reçoit la date du jour en dernier argument** au lieu de lire l'horloge ;
les neuf sites de production que la 35-5a basculera au jour suisse lisent l'horloge de l'état, **une fois par requête** ; la variable
de tests `KESH_TEST_INSTANT` fige l'horloge des tests d'intégration ; une garde lexicale (`date_du_jour_garde.rs`) ferme les formes.
**Aucune migration, aucun texte visible, aucun manuel.** Elle prépare la 35-5a (date du jour suisse, #929/#1022/#907) : ce qui y est
faux sera faux dans les livres réels de l'utilisateur après la 35-5a (dates de contre-passation, verrous d'exercice, soldes
d'ouverture).

Worktree `/home/gcorbaz/devel/kesh-e355a0`, branche `story/35-5a0-horloge-plomberie`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e355a0 diff 2b1c0ec86 0deb9abcf` (`2b1c0ec86` = pointe de la 35-3, close, base de la story ;
`0deb9abcf` = dernier commit de code ; `d84889e7d` = compte rendu seul). 80 fichiers : lis la production en entier, les tests par
échantillon raisonné (voir l'axe 4). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-5a0-horloge-plomberie.md` en entier (*Ce qui se lit au code*, D1 à D6, AC1 à AC6, mutations
M1 à M6, *Dérogation règle de splitting*, *Ce qui doit rester vrai*, **Change Log** et **Dev Agent Record**) ; la fiche de la 35-5a
(`35-5a-*.md`, D4 de la garde) pour ce que la plomberie doit lui permettre ; les registres `e35-choix-autonomes-e350.md` (C-e350-8,
-21, -24, -26, -28, -29) et `e35-choix-autonomes-e355a0.md` (C-e355a0-1 à -7). Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e355a0/` : `etape0-*.log`,
`rouges.log`, `c2-*.log`, `c3-*.log`, `c5-*.log` (dont `c5-test-fast.log`, `c5-gate-ci.log`, `c5-e2e.log`, `c5-front-*.log`),
`garde-comptes-bruts.log`, `mut/`, `c3-conversion.log`. Les scripts `*.sh` et `*.py` sont à **lire**, jamais à exécuter. **Ne lis pas
`gwt/`** (clone de gate, copie du dépôt).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
  aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Aucun comportement ne change (AC1)** : même jour (UTC), mêmes refus, mêmes écritures. Le contrôle des attendus retirés
   (`git diff -U0 --diff-filter=M 2b1c0ec86 0deb9abcf -- 'crates/*/tests/*' 'crates/*/src/*' | /usr/bin/grep -E '^-' | …`, commande
   de la fiche) : refais-le, et confronte chaque ligne à la liste du Change Log (34 lignes dites « reflux de rustfmt »). Une date, un
   montant, un statut ou un code d'erreur changé est un finding.
2. **La date de pièce (D4)** : aucun appel ne passe une date d'écriture, de facture ou de paiement comme `aujourd_hui`. Les pièges
   nommés (`payer_par`, `payer`, `facture_payee` et sa variable `jour`) : relis-les au code. Dans les appels **de production** de
   `kesh-api` (18) et les appels internes de `kesh-db`, d'où vient la date passée ? Toujours de l'horloge de l'état, lue une fois ?
   Une fonction de `kesh-db` qui reçoit la date et la **transmet** à une autre (bloqueur → contre-passation) : la même variable ?
   M4 (deux lectures) est-elle vraiment impossible, ou seulement non relevée ?
3. **La lecture une fois par requête** : avant `retry_on_deadlock` et l'attente du verrou (D2, « nuance voulue ») — vérifie chaque
   site au code. Un site qui lit l'horloge **dans** la boucle de reprise, ou deux fois sur un même chemin, est un finding.
   `ReversalDating::Today` : porte-t-il la date, ou la lit-il encore ? Les 19 sites de la 35-5b sont-ils intacts ?
4. **La conversion par script (C-e355a0-3)** : 311 appels de tests ×33 fichiers et 26 déclarations de helper, par
   `conv-e355a0.py` et `decl-e355a0.py`. Lis les deux scripts : le script ne fait-il **qu'ajouter** `aujourd_hui_test()` en dernier
   argument, sur les seuls sites des erreurs E0061 (`c3-erreurs.json`) ? Peut-il avoir ajouté l'argument au mauvais appel d'une
   ligne qui en porte deux, ou dans une macro ? Échantillonne au moins dix sites convertis dans des fichiers différents
   (`git diff 2b1c0ec86 0deb9abcf -- crates/kesh-db/tests crates/kesh-api/tests crates/kesh-report/tests`) et un test qui portait déjà
   une variable `today`/`aujourd_hui` (D4 : il devait la passer, non `aujourd_hui_test()` — sinon le test et le serveur peuvent lire
   deux jours différents autour de minuit).
5. **L'horloge et `KESH_TEST_INSTANT` (D1, D3, D6)** : `main.rs` construit `HorlogeSysteme` et ne lit pas la variable ; aucun code de
   production n'appelle `new_for_tests` ni `horloge_de_test` ; `horloge_de_test` lue à chaque appel, sans mémoire. L'écart à D3
   (`config::env_nonempty` au lieu de `std::env::var`, C-e355a0-7) : `env_nonempty` rend-il `None` pour une chaîne vide **et** pour
   une chaîne de blancs, et est-ce voulu ? Le correctif de `configuration_transmise.rs` (liste `VARIABLES_DE_TEST`) ouvre-t-il une
   porte (une variable de production déclarée « de test » pour échapper à la garde) ? Le test qui exige l'absence de la variable de
   `.env.example` et des fichiers compose. Le `mod tests` de `src` : `HorlogeFixe(Utc::now())` partout, jamais `horloge_de_test` ?
6. **La garde (D5, AC5)** : listes closes en égalité stricte par fichier, commentaires `//` écartés, retrait des `use` borné au seul
   compte de `HorlogeSysteme`, noms comme mots. Le compte `NOM_VARIABLE_INSTANT` 2 dans `configuration_transmise.rs` (écart à
   « 0 ailleurs ») : justifié, et sans trou ? Les sondes d'AC5 sont-elles toutes au Change Log, avec leur résultat (rouges et
   vertes) ? Une forme qui échappe : un `Utc::now().date_naive()` coupé sur deux lignes, `chrono::Local::today`, `CURDATE()` en SQL,
   `NOW()`, `CURRENT_DATE` ? Cite la liste des formes relevées par la garde et dis ce qui reste hors d'elle.
7. **`kesh-db` et l'exception de `CLAUDE.md`** (gate complet dès qu'un patch touche un repository) : les commits (3) et (4) n'ont eu
   que fmt, clippy et compilation (C-e355a0-5, écart déclaré) ; le gate complet du commit (5) couvre-t-il leur contenu (rien n'a
   bougé dans `kesh-db` entre (3) et (5) ?) ? `allow(clippy::duplicate_mod)` (C-e355a0-4) : masque-t-il autre chose que le double
   `jour.rs` ? L'inclusion `#[path]` imbriquée se résout-elle comme la fiche le dit ?
8. **Les témoins et les mutations** : AC2 (2 des 9 sites, contre-passation et verrou, à `HorlogeFixe` 2026-06-15 et 2025-12-31T23:30Z),
   AC3 (MCP, jour distinct du jour réel), AC4 (`assert_ne!` après `sleep`) : chaque test peut-il rougir ? M1a, M1b, M1c, M2 à M6
   tuées chacune par le test **nommé** au journal (`mut/`) ? Propose des mutations plausibles non jouées (un site qui lit
   `Utc::now()` sous un autre nom, la date du jour passée à une seule des deux fonctions d'un chemin).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `kesh-core/src/horloge.rs`, `kesh-api/src/horloge.rs`, `main.rs`, `lib.rs`, `services/poignees.rs`, les signatures changées de
`kesh-db` et leurs appels de production, `date_du_jour_garde.rs`, `configuration_transmise.rs`. Cherche : une horloge figée au
démarrage ; une date lue deux fois sur un chemin ; une date de pièce passée comme date du jour ; un `Arc<dyn Horloge>` cloné vers la
mauvaise source ; une garde qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Minuit UTC (23:59:59 → 00:00:00) pendant une requête qui attend un verrou ou rejoue sur interblocage ; `KESH_TEST_INSTANT` vide,
blanche, invalide, avec fuseau `+01:00`, sans fuseau ; un test qui fixe l'horloge puis appelle un helper `aujourd_hui_test()` (même
jour ?) ; la Saint-Sylvestre (2025-12-31T23:30Z) ; le MCP ; l'assistant de démarrage (site du rang 5) ; `unlock_books` ; les soldes
d'ouverture ×4. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC6 : livré, testé, au lieu prescrit ? L'étape 0 (158 appels / 23 fichiers, 43 `Lecture::new(`, 9 littéraux,
« 50 / 36 / 68 » non refait) citée au Change Log ? Les gates au dernier commit de code `0deb9abcf` : test-fast 5343, gate-ci 5350,
Vitest 3188, E2E 403 / 7 KF-029 — chaque chiffre au journal, horodaté après le commit ? Le rouge du commit (2) (2 échecs
`configuration_transmise`) est-il dit exactement ? Les écarts (scripts de conversion, cinquième commit, `env_nonempty`, compte 2)
sont-ils exacts, déclarés, et sans effet sur le comportement ? Ce que la story doit transmettre à la 35-5a est-il écrit ? Le Dev Agent
Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
