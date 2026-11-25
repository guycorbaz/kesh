# Revue de code P2 — Story 49-1b « Types de lignes — rendu PDF » (#1242)

*Passe **complète** (la P1 a rendu un MEDIUM), **lentille unique**, contexte frais, modèle **Sonnet** (la P1 était
Opus). Lecture seule.*

## Ce que tu relis, dans cet ordre

1. **La remédiation d'abord** : `git -C /home/gcorbaz/devel/kesh-49-1b show 4dd9932ea` (commit
   `fix(49-1b) : remédiation de la revue P1`), puis les commits qui le suivent, s'il y en a (`git log --oneline 4dd9932ea..HEAD`).
   Lis-la contre les findings qu'elle traite : `/home/gcorbaz/devel/kesh-gate-logs/e491b/review-p1-{B,E,A}.md`
   (MEDIUM B-M1 = A-1 = E-1 ; LOW L1 à L6, E-2 à E-6, A-2 à A-5).
2. **Puis toute la story** : `git -C /home/gcorbaz/devel/kesh-49-1b diff cbdd70026..HEAD -- crates` (base `cbdd70026` =
   la 49-1a fusionnée dans `epic/49-types-de-lignes-facture`).
3. **La fiche** `_bmad-output/implementation-artifacts/49-1b-types-de-lignes-pdf.md` (D1 à D10, AC1 à AC14, Dev Agent Record,
   dont § *Remédiation de la revue P1*) et le registre `e0200-choix-autonomes-e491b.md` (C-e491b-1 à 5).
4. **Journaux** dans `/home/gcorbaz/devel/kesh-gate-logs/e491b/` : `ac1-controles-p1.log`, `gate-cible-p1-1.log`,
   `rouge-*.log`, `mut/bilan.log`, `test-fast-p1.log` ; **captures** `captures/*.png` (refaites après la remédiation).

## Le contexte qui commande la revue

Guy a déjà émis de **vraies factures**, remises sur papier. Rien ne doit changer le PDF d'une facture ou d'un avoir faits
de lignes chiffrées. La preuve en est la comparaison avec des références capturées avant tout code (commits C
`709370ed5` et F `12be35434`), ancrées à `cbdd70026` (0.19.0 + 49-1a). La garde de l'API reste fermée : aucun commentaire
n'est créable par l'utilisateur en 0.20.0.

## Axes (tous à exercer)

1. **La preuve avant / après après la remédiation.** Le chemin chiffré de `draw_invoice_section` et `couper_ligne` à
   8 pt sont-ils intacts (`git diff 03afbfbca..HEAD -- crates/kesh-qrbill/src/pdf.rs crates/kesh-qrbill/src/texte.rs`) ?
   `texte.rs` ne change-t-il que de visibilité ? Rejoue en lecture `kesh-gate-logs/e491b/ac1-controles.sh` (lis-le d'abord : git
   seulement) : (1) à (6).
2. **La garde `configuration_transmise`** (le MEDIUM de la P1) : l'exclusion de fichier a disparu ; les deux autorisations
   exactes d'`empreinte.rs` ; `lues_distribuees` n'écarte-t-elle que les variables de test, et seulement de (T) et (E) ?
   `controle_manifeste_features_de_test` : une activation de `test-support` (liste `default`, feature d'un autre crate,
   `[workspace.dependencies]`, `[dependencies.x]`, `[target.….dependencies]`, liste sur plusieurs lignes, ligne commentée)
   peut-elle encore passer ? `la_feature_de_test_n_entre_pas_en_production` : le `cargo tree -e normal --offline` est-il
   sûr en CI (`gate-ci`, job `backend` : un processus par binaire, `cargo test` en cours), borné, sans réseau ? Sa sonde
   positive suffit-elle ?
3. **Le chemin des commentaires et des titres** (`texte_de_commentaire`, `lignes_de_commentaire`) : `\r`, CRLF, sauts
   Unicode, blancs Unicode, U+00AD, lettres repliées, caractères non imprimables, lignes vides de tête, de fin et
   intérieures, ’ et —, mot plus large que la ligne, seuil de capacité. La chaîne mesurée est-elle exactement la chaîne
   dessinée ? Un caractère imprimable peut-il être retiré à tort (la règle : largeur nulle à 9 pt = non imprimable) ?
4. **Les tests ajoutés** : T4 (polices et ressources égales), T5 (cas E-2 à E-5), T9 (gel et regel d'une facture à
   commentaire, sur le dossier des documents partagé `/tmp/kesh-documents-test`), le dossier temporaire du test HTTP ;
   prouvent-ils ce qu'ils disent ? Chaque mutation de `mut/bilan.log` (G-3 à G-8, P1-E2 à P1-E5) est-elle jouée et tuée
   par le test visé ?
5. **Les écrits** : `README.md` de la crate, `types.rs`, doc-comment de `draw_invoice_section` (réduction au-delà de 45
   caractères) ; en-têtes de `rendu_avant_apres.rs` et `pdf_avant_apres_e2e.rs` (ancrage, angles morts du comparateur) ;
   Dev Agent Record honnête (chaque chiffre a son journal).
6. **Ce que la P1 n'a pas exercé** : comportement de `lopdf` sur les caractères de contrôle, contenu des fichiers
   `.empreinte`, manuels autres que le français.

## Le manuel (obligatoire)

Vérifie le manuel contre le code, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-49-1b/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(existence par `ls -l`). Lis en particulier `docs/manual/fr/user-manual.tex:1594` (« le reste du corps (libellés des
lignes…) n'a pas ce repli » ; « le reste du corps les garde tels quels ») : vraie en 0.20.0 tant qu'aucun commentaire
n'est créable ; ce qu'elle devra dire à la 49-1c est écrit au Dev Agent Record — vérifie que c'est exact au code.

## Les captures (obligatoire)

Lis les six PNG `kesh-gate-logs/e491b/captures/49-1b-*.png` : le témoin est-il la facture S2 (cinq lignes, deux taux,
ligne « Arrondi ») ? Les commentaires sont-ils lisibles, à gauche, sans colonnes, coupés sans troncature ?

## Interdits

Lecture seule. N'exécute **aucune** de ces commandes : `cargo` (build, test, run, fmt, clippy, tree, metadata), `npm`,
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`,
`kesh-gate-logs/e491b/mut/mutations.py`, `kesh-gate-logs/e491b/capture-e491b.sh`, `kesh-gate-logs/e491b/gates-review.sh`,
`kesh-gate-logs/e491b/gate-cible.sh`, `git commit`, `git checkout`, `git restore`, `git stash`, `git worktree`, `git reset`,
`rm`, `mv`, `cp`, `touch`, `sed -i`, `docker`, ni redirection vers un fichier, ni fichier temporaire. Seule écriture
permise : ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e491b/review-p2.md`. Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et
ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute affirmation de présence ou d'absence s'appuie
sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et non exercés** (un « 0 finding » sans elle ne
compte pas) et le décompte par sévérité.
