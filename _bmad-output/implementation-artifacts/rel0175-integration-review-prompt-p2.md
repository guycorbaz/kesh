# Revue ciblée P2 — la remédiation de M-1 et L-1 (une lentille, lecture seule)

Contexte frais, français soigné. Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11) ; la 0.17.4 tient des
livres réels en production. La P1 de l'intégration de la 0.17.5 (`/home/gcorbaz/devel/kesh-gate-logs/rel0175-review-p1.md`,
prompt `rel0175-integration-review-prompt-p1.md`) a rendu un MEDIUM (M-1) et un LOW (L-1). Ils ont été remédiés. **Ton
seul périmètre est cette remédiation** : la sévérité se déplace vers ce qu'on vient d'écrire, et personne ne l'a relue.

Worktree `/home/gcorbaz/devel/kesh-rel0175`, branche `release/0.17.5`. Bornes : **`c5f3d1f1..11b0315e`** (le commit de
code), plus le commit de documentation qui le suit (CHANGELOG, manuel `.tex` et PDF, registre). Lis par
`git show` / `git diff c5f3d1f1..11b0315e`, jamais en supposant le fichier sur disque à jour.

## Ce qui a été fait (à vérifier, non à croire)

- `crates/kesh-qrbill/src/validation.rs` : `est_vide` devient `pub fn champ_vide_pour_qr`, mêmes appels internes ;
  réexportée par `crates/kesh-qrbill/src/lib.rs`.
- `crates/kesh-api/src/address_input.rs` : `validate_required` juge « obligatoire » NPA et localité par
  `champ_vide_pour_qr`, champ par champ, nouveau message « adresse : le champ {label} est obligatoire (format structuré
  exigé par les banques suisses) » (l'ancien message commun disparaît) ; `validate_required_for_qr` l'hérite, et
  l'assistant (`crates/kesh-api/src/routes/onboarding.rs`, `body.address.validate_required()`) aussi.
- Tests : `a_required_field_with_nothing_visible_is_refused_here` et le témoin `what_the_route_accepts_the_qr_bill_produces`
  (`crates/kesh-api/tests/company_address_e2e.rs`), `required_fields_with_nothing_visible_are_refused` (unitaire).
- Registre `_bmad-output/implementation-artifacts/e33-choix-autonomes-rel0175.md` : C-rel0175-1 complété, C-rel0175-9.
- Manuel utilisateur `docs/manual/fr/user-manual.tex` (« Corriger votre adresse ») et son PDF ; CHANGELOG `[0.17.5]`,
  puce #909.

## Axes obligatoires

1. **La règle n'est pas dupliquée et reste celle de `validate`** : `champ_vide_pour_qr` est-il bien l'unique définition,
   appelé par `validate_address` aux mêmes champs qu'avant (nom, NPA, localité en type S, ligne 2 en type K) ?
2. **Ce que le nouveau message casse** : quelqu'un relisait-il l'ancien texte « le NPA et la localité sont obligatoires »
   (tests Rust, Vitest, Playwright, frontend qui affiche ou compare le message serveur, i18n des quatre locales, manuels) ?
   L'assistant (`frontend/src/routes/onboarding/+page.svelte`) affiche-t-il encore quelque chose de juste ?
3. **Ce que la garde refuse en trop** : une valeur légitime (U+00AD entre deux lettres, NPA étranger, U+00A0 intérieur,
   lettre accentuée) est-elle désormais refusée ? `is_invisible` (`crates/kesh-core/src/text.rs`) inclut `is_control` et
   `is_whitespace` : quelle conséquence pour un NPA ou une localité ?
4. **Ordre des contrôles** : `check_lengths` passe avant la garde, `first_qr_refused_char` après ; un champ fait d'un
   caractère refusé par le jeu et invisible (U+200B) reçoit-il un message juste ?
5. **Les tests prouvent-ils ce que leur nom dit** : le test de route rougirait-il sans la garde (journal de la mutation
   `/home/gcorbaz/devel/kesh-gate-logs/rel0175/rem-p1/mutation-garde.log`, rouge `rouge.log`) ? Le témoin du PDF
   exerce-t-il réellement `&`, `’` et U+00AD jusqu'au PDF (statut 200 du PDF, non seulement du `PUT`) ?
6. **Ce qui reste** : les contrôles de lecture de `crates/kesh-api/src/routes/invoice_pdf_service.rs` (lignes 287 et 310,
   `trim().is_empty()`), laissés et tracés au registre (C-rel0175-9) : le jugement « seul le message diffère » est-il exact ?
   Autres sites `trim().is_empty()` sur un NPA ou une localité destinés à la QR-facture (`git grep`).
7. **Le manuel et le CHANGELOG disent vrai** du code : `.tex` et PDF aplati
   (`git show HEAD:docs/manual/fr/user-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`), la phrase ajoutée y figure-t-elle,
   et dit-elle « ici comme à l'assistant » à bon droit ?
8. **Décomptes et registre** : le registre dit-il vrai des écartées et de ce qui est laissé ?

## Contraintes

⛔ Ton rapport s'écrit à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/rel0175-review-p2.md`, avec l'outil
Write. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire ; **aucune compilation, aucune exécution de
test ni de script, aucune commande qui écrive** ; aucune base, aucun `docker` ; `scripts/prepare-release.sh`,
`git checkout`, `git stash`, `git switch`, `git merge`, `git commit` et tout `gh` qui écrit sont interdits. `grep` est une
fonction vers ugrep : `/usr/bin/grep` ou `git grep`. Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`.

Le rapport porte : **la liste des axes exercés et de ceux qui ne l'ont pas été** (un « 0 finding » sans elle ne compte
pas) ; chaque finding avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), fichier et ligne, extrait lu, scénario, remédiation ;
toute présence ou absence adossée à une sortie `git grep -nF` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
