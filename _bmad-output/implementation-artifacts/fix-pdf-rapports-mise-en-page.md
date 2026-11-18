# Correctif : mise en page des PDF de rapports (montants alignés à droite, titre accentué, en-têtes de devise)

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/pdf-montants-alignes`, partie de `main` à `70be12a6` (squash de l'Epic 37,
PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` (aucune publication intermédiaire) et va
à la prochaine release planifiée. **Aucune migration.** Constaté par Beat sur les PDF de la recette 2024
(`data-not-committed/2024-qesh/sorties-kesh/`, bilan et compte de résultat). Périmètre élargi par Beat le même jour à
deux points de mise en page (titre des propriétés, en-têtes de devise) ; la fiche s'appelait
`fix-pdf-montants-alignes.md`.*

## Intention

Trois défauts de mise en page des PDF de rapports (`crates/kesh-report/src/pdf.rs`) :

1. **Montants alignés à gauche.** Chaque montant commençait à l'abscisse de sa colonne : ses décimales ne s'alignaient
   pas d'une ligne à l'autre (« 2'168.17 » au-dessus de « 4.67 »), dans la colonne en devise principale comme dans
   celle du solde en devise (« EUR 17'072.20 ») ajoutée par la Story 37-4.
2. **Titre des propriétés mal encodé.** printpdf 0.7 écrit le `/Title` du dictionnaire `Info` avec les octets UTF-8 du
   titre, qu'un lecteur décode en PDFDocEncoding : « Compte de rÃ©sultat », « DÃ©compte TVA ».
3. **Colonnes sans devise.** Un rapport qui montre un montant en devise à côté du CHF ne dit nulle part que l'autre
   colonne est en CHF.

## Critères d'acceptation

1. **AC-1 — Bord droit commun.** Dans tout PDF de rapport, chaque montant d'une colonne finit au même bord droit, quelle
   que soit sa longueur, son signe, sa graisse (totaux et sous-totaux en gras) : bilan, compte de résultat, balance,
   grand livre, journaux, décompte TVA, dépenses et rendement par compte analytique.
2. **AC-2 — Devise.** Le solde en devise du bilan et du compte de résultat (« USD 1'233.57 ») est aligné à droite sur
   son propre bord, code et montant lisibles ; au grand livre, le solde en devise des lignes d'encadrement (ouverture,
   report, rupture, clôture) aussi.
3. **AC-3 — Mesure.** La position se calcule par la métrique de la police (Helvetica et Helvetica-Bold, chasses AFM en
   WinAnsi), non par un nombre de caractères ; une seule fonction d'alignement, partagée (DRY), documentée.
4. **AC-4 — Contenu.** En mono-devise, mêmes chaînes écrites, dans le même ordre : seules les abscisses changent. En
   multi-devises, les seuls textes ajoutés ou modifiés sont les en-têtes de l'AC-9. Les secondes lignes en petit
   (phrases du grand livre et de la balance) restent alignées à gauche.
5. **AC-5 — Pas de chevauchement.** Le plus long montant plausible (« -12'345'678.90 ») et le plus long en-tête ne
   recouvrent ni la colonne qui les précède ni la marge droite (15 mm, symétrique de la gauche).
6. **AC-6 — Tests.** Unitaires sur la position calculée et sur le titre ; de bout en bout sur le PDF rendu ; tests PDF
   existants verts.
7. **AC-7 — Recette visuelle.** Captures avant/après de chaque rapport touché, mono-devise et multi-devises
   (`kesh-gate-logs/fix-pdf-alignement/captures/`).
8. **AC-8 — Titre des propriétés.** Le titre d'un PDF de rapport se relit juste dans ses propriétés, accents compris
   (« Compte de résultat », « Décompte TVA », tout titre non ASCII, y compris hors du plan de base), dans un PDF dont la
   structure reste valide ; un titre ASCII ne change pas.
9. **AC-9 — En-têtes de devise.** Dès qu'un rapport contient un compte en devise : « Devise » et « CHF » en tête des
   deux colonnes de montant du bilan et du compte de résultat, alignés à droite sur elles ; à la balance et au grand
   livre, les en-têtes de montant suivis de « CHF » (« Débit CHF »), pour tous les comptes du rapport. Libellés dans
   les quatre locales.

## Hors périmètre

- Les **factures** (`kesh-qrbill`) partagent le défaut d'alignement — montants du tableau des lignes, du récapitulatif
  TVA et du total écrits à l'abscisse de leur colonne (`crates/kesh-qrbill/src/pdf.rs`, `col_qty`, `col_unit`,
  `col_vat`, `col_tot`) — mais ne sont pas touchées : à tracer à part.
- La ligne « Résultat reporté (calculé) » (décision de Beat : traitée ailleurs).
- Relevés en passant, non traités : flèche « → » de la période, « ↳ » des sous-projets et « ⚠ » du bandeau du grand
  livre, hors WinAnsi, qui disparaissent à l'impression ; intitulé de compte des journaux non tronqué. Tous tracés au
  § *Issues à ouvrir*. (Le commentaire de `draw_empty_message`, devenu faux, est corrigé — revue P1, P1-3.)

## Tâches

- [x] `right_aligned_x` (bord droit − largeur imprimée) et `PdfBuilder::write_right`, partagés par toutes les colonnes de
  montant ; la largeur vient de `kesh_qrbill::largeur_mm`, désormais exportée (table engendrée de la Story 36-12a).
- [x] Bords des colonnes : constantes nommées, documentées, avec la mesure du plus long montant.
- [x] Huit rapports convertis, en-têtes des colonnes numériques compris (balance, grand livre, TVA, rendement).
- [x] Titre des propriétés en UTF-16BE (`title_placeholder`, `pdf_utf16_hex`, `set_pdf_title`).
- [x] En-têtes de devise (`draw_currency_columns_header`, `SectionLabels::amount_header`), libellés
  `col_base_currency` / `col_foreign_currency`, clés i18n dans les quatre locales.
- [x] Tests unitaires et de bout en bout ; mutations qui les font rougir.
- [x] Recette indépendante par poppler (`pdftotext -bbox-layout`, `pdfinfo`) et comparaison des textes avant/après.
- [x] Manuel utilisateur (`.tex` seulement).

## Notes de développement

- **Source des largeurs.** La table des chasses d'Helvetica existe déjà, engendrée et vérifiée
  (`crates/kesh-qrbill/src/largeurs.rs`, Story 36-12a). Plutôt que de la recopier, `largeur_mm` devient publique et
  `kesh-report` dépend de `kesh-qrbill` (déjà au graphe par `kesh-db`). ⚠️ Écart à relire : la décision 12 de
  l'architecture sépare les PDF des deux crates ; seule une table de métriques est partagée, non une couche de rendu.
  L'autre voie — déplacer la table dans `kesh-core` — toucherait le script `engendrer-largeurs-helvetica.py` et les
  tests propres à la QR-facture de `largeurs.rs` ; elle n'a pas été prise. **Décision de Beat (2026-11-18)** : la
  dépendance est acceptée ; la ligne ajoutée aux « Dépendances croisées » de
  `_bmad-output/planning-artifacts/architecture.md` en fait foi (revue P1, P1-1).
- **Chiffres de même chasse.** Les dix chiffres d'Helvetica valent 556 millièmes d'em en maigre comme en gras ;
  l'apostrophe (191 / 238) et le signe moins (333), non : la largeur est mesurée, non déduite du nombre de caractères.
- **Bords retenus** (mm depuis le bord gauche de la page A4) : montants en devise principale et totaux 195 (marge
  droite) ; solde en devise du bilan et du compte de résultat 160 ; balance 117, 143, 169, 195 ; journaux 160 et 195 ;
  TVA 150 et 195 ; rendement 108, 138, 168, 195 ; grand livre (8 pt) 149, 172, 195, et 127 pour le solde en devise des
  lignes d'encadrement. « -12'345'678.90 » mesure 23,1 mm en 10 pt (23,4 en gras), 18,5 mm en 8 pt.
- **Titre : substitution à longueur égale, sans relecture.** printpdf n'offre aucun moyen d'écrire un titre autrement
  qu'en octets UTF-8 (son titre est une `String`). Relire et réécrire le PDF par lopdf aurait coûté une analyse
  complète de chaque document (un grand livre fait plusieurs Mo). Le document est donc créé sous un gabarit ASCII
  (`KeshTitrePdf` complété de `x`) de la longueur exacte de la chaîne `<FEFF…>` (4 + 4 par unité UTF-16), puis la
  chaîne littérale `(gabarit)` est remplacée dans les octets : la table des références croisées reste juste. Le
  remplacement exige une occurrence unique, sinon le rendu échoue (`ReportError::PdfGeneration`). Le test relit le
  titre par lopdf, ce qui contrôle aussi la validité de la structure.
- **En-têtes de devise : décision mono-devise.** Un rapport mono-devise n'a qu'une colonne de montant, en CHF, et
  le manuel promet qu'une société sans compte en devise voit ses rapports « comme avant, à l'écran comme en PDF »
  (§ *Les rapports d'une société multi-devises*) : il ne reçoit **aucun** en-tête neuf — la garde « mêmes lignes de
  texte en mono-devise » tient (`comparer-textes.log`). En multi-devises, la règle vaut pour **tout le rapport** (tous
  les comptes du grand livre, même en CHF), pour un rendu égal d'un compte à l'autre. Au bilan et au compte de
  résultat, une ligne d'en-têtes « Devise » / « CHF » ; à la balance et au grand livre, « CHF » suit l'en-tête de
  chaque colonne de montant (les montants en devise y sont dans la seconde ligne, qui porte le code de la devise).
- **En-têtes de devise : libellés.** Tous les libellés des PDF de rapports sont aujourd'hui les défauts FR-CH de
  `SectionLabels::fr_ch_defaults` (« i18n PDF complète déférée », `load_pdf_context` de `kesh-api`). Les deux libellés
  neufs suivent la même voie ; leurs clés `reports-pdf-col-base-currency` (CHF) et `reports-pdf-col-foreign-currency`
  (Devise, Währung, Valuta, Currency) sont posées dans les quatre locales, comme `reports-pdf-header-period`, mais ne
  sont pas encore lues : un seul mot allemand dans un PDF tout en français serait pire que le français partout.
  **Décision de Beat (2026-11-18)** : les deux clés restent **non lues** par les PDF jusqu'à la traduction complète des
  PDF — pas de mot allemand isolé dans un PDF français. Issue à ouvrir (§ *Issues à ouvrir*, I-7).

## Change Log

- 2026-11-18 — Alignement à droite (`4de1b96b` code, `bd087f4a` tests), base `70be12a6`. Mutation : `write_right`
  ramené à une abscisse fixe → les six tests de bout en bout rougissent, les trois unitaires restent verts.
- 2026-11-18 — Périmètre élargi par Beat : titre des propriétés et en-têtes de devise (`cc6aabe3`, code et tests).
  Mutations : titre écrit tel quel (sans gabarit ni substitution) → `le_titre_des_proprietes_du_pdf_se_relit_juste`
  rouge ; en-tête « CHF » de la balance et du grand livre retiré → `balance_et_grand_livre_disent_chf_en_multi_devises`
  et `balance_pdf_aligne_les_quatre_colonnes_a_droite` rouges ; ligne d'en-têtes du bilan retirée →
  `bilan_et_resultat_disent_la_devise_de_leurs_colonnes_en_multi_devises` rouge. Tests de `pdf.rs`
  (`grep -c '#\[test\]'`) : 41 à `70be12a6`, 50 à `bd087f4a`, 55 à `cc6aabe3` — quatorze ajoutés au total.

- 2026-11-18 — **Revue de code P1** (Opus 5.5, passe complète, lentilles B, E et A réunies, contexte frais ;
  `kesh-gate-logs/fix-pdf-alignement/revue-p1.md`) : **0 CRITICAL / 0 HIGH / 0 MEDIUM / 8 LOW**. Corrigés sans code
  exécutable (commentaires, documentation, manuel), commit `docs(...)` : P1-1 (ligne d'architecture sur la dépendance),
  P1-2 (commentaire de `draw_account_row`), P1-3 (commentaire de `draw_empty_message` ; doc de `largeur_mm` qui
  renvoyait à des items non publics, `texte_imprime` et `WINANSI` ; doc du module `pdf` qui renvoyait à
  `PdfBuilder::write_right`, privé ; en-tête de `kesh-qrbill/src/lib.rs`, « standalone », périmé), P1-5 (manuel :
  « aucun des deux n'est tronqué » était faux, le PDF abrège les intitulés trop longs ; le solde en devise n'a sa propre
  colonne qu'au bilan et au compte de résultat), P1-7 en partie (réserve « à revoir à toute montée de printpdf ou de
  lopdf » au doc-comment de `title_placeholder`). Non corrigés, tracés au § *Issues à ouvrir* : P1-4, P1-6, P1-7, P1-8.
  Contrôles après correction : `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo doc -p kesh-qrbill -p kesh-report --no-deps` verts — `post-p1-checks.log` ; `cargo doc` y garde cinq
  avertissements de liens vers des items privés, tous **préexistants** (`kesh-qrbill` : `REMINDER_*_MAX_CHARS`, une
  URL de `validation.rs` ; `kesh-report` : `NAMED_POSTS` de `homepage_summary.rs`), aucun de cette branche. Pas de gate
  complet (aucun code exécutable touché). Trend : P1 Opus 0 > LOW (8 LOW) ; passe ciblée P2 sur le commit de
  correction à lancer (`revue-p2-ciblee-prompt.md`).
- 2026-11-18 — **Décisions de Beat** : (1) clés `reports-pdf-col-*` non lues jusqu'à la traduction complète des PDF ;
  (2) dépendance `kesh-report` → `kesh-qrbill` acceptée, la ligne d'architecture fait foi.
- 2026-11-18 — passe P2 ciblée (Haiku) sur `62ccfa4a` : 0 CRITICAL/HIGH, **1 MEDIUM**, 4 LOW. MEDIUM-1 : `crates/kesh-qrbill/README.md` disait encore le crate « standalone, zero dependency on kesh-core » (propagation manquée de P1-3) — corrigé. LOW-1 (manuel : troncature à un nombre fixe de caractères, non « trop longs pour leur colonne »), LOW-2 (`WINANSI` est `pub(crate)`), LOW-3 (message vide non centré), LOW-4 (`architecture.md` : `kesh-report` ne dépend pas de `kesh-i18n`) — corrigés dans le même commit, sans code exécutable. Passe P3 ciblée à suivre (une passe à MEDIUM appelle une passe de plus).
- 2026-11-18 — passe P3 ciblée (Sonnet) sur `a2759eed` : **0 au-dessus de LOW**, 4 LOW (copies restées fausses de la description de `kesh-qrbill` et des dépendances de `kesh-report` dans des documents de planning et le README ; tracées en une issue de documentation, I-8). **Boucle close.** Trend : P1 Opus 0 > LOW (8 LOW) → P2 ciblée Haiku 1 MEDIUM + 4 LOW → P3 ciblée Sonnet 0 > LOW (4 LOW). Dernier commit de code `cc6aabe3`, gates complets verts à ce commit ; les commits suivants ne portent que documentation et commentaires.
- 2026-11-18 — **Reprise de `main` après la v0.19.0, puis gate complet groupé.** `origin/main` (`773df80d`, tag
  `v0.19.0`) repris par fusion (`4da013c1`), sans rebase ; conflits résolus en gardant les deux intentions (`sprint-status.yaml` : les deux blocs).
  Gate ciblé après la reprise : `fmt`, `clippy` du workspace, nextest de `kesh-report` et `kesh-qrbill` **353 passés, 2 ignorés**, garde i18n verte (`kesh-gate-logs/fix-pdf-alignement/reprise-v0190-*.log`). Puis `origin/main` `37c2f875` (documentation seule : `CLAUDE.md`, bilan de la
  0.19.0, `sprint-status.yaml`) repris de même. **Gate complet groupé** sur la branche jetable `integ/fixes-0.19.0`
  (`773df80d` + les quatre correctifs fusionnés dans l'ordre PDF, monnaies retirées, résultat reporté, parent du
  compte ; seul conflit, `sprint-status.yaml`), **au commit `5fc6fb1b`**, base `kesh_integ` remise à zéro :
  `scripts/test-fast.sh --ci` **6165 passés, 14 ignorés** ; frontend `check` 0 erreur (29 avertissements
  préexistants), `lint-i18n-ownership`, `test:unit` **4627 tests (278 fichiers)**, `build` ; **E2E complet**
  (`LANG=fr_CH.UTF-8`, port 3382) **478 passés, 10 échecs, 18 ignorés** — les dix attendus, jugés un à un : sept KF-029
  (#97 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`), `backup-restore-new-install.spec.ts:262`, `:381`, `opening-balances.spec.ts:156` (`docs/testing.md`
  inchangé par la 0.19.0) ; `scripts/gate-ci.sh` (copie `git archive`) **6184 passés, 0 échoué, 18 ignorés**.
  Journaux : `kesh-gate-logs/integ-fixes-0.19.0/` (`gate-backend.log`, `front-*.log`, `e2e.log`, `gate-ci.log`) ; un
  premier essai, arrêté par le disque plein de la station, est gardé dans `avorte-disque-plein/`. Le commit de
  documentation `37c2f875` n'a pas été rejoué au gate (aucun code).

## Issues à ouvrir

*(Non ouvertes par l'agent — ni `gh`, ni issue ; reprises dans `kesh-gate-logs/issues-a-ouvrir-fixes.md`, une entrée
par point, priorité et étiquette proposées.)*

- **I-1 (P1-4)** — Tests de bord manquants : colonne « Rendement » (`RET_RENDEMENT_RIGHT_MM`) jamais contrôlée ;
  bord des en-têtes du grand livre, de la TVA et du rendement non testé. P3, `technical-debt`.
- **I-2 (P1-6)** — Bilan et compte de résultat multi-devises sur plusieurs pages : l'en-tête « Devise / CHF » n'est
  pas répété à partir de la page 2. P4, `bug`.
- **I-3 (P1-7)** — Titre des propriétés : procédé lié au fonctionnement interne de printpdf 0.7 / lopdf 0.31 ; à
  revoir à toute montée, et un `pdfinfo` d'un PDF produit par l'image release à la recette de release. P3,
  `technical-debt`.
- **I-4 (P1-8)** — Factures (`kesh-qrbill`) : montants du tableau des lignes, du récapitulatif TVA et du total alignés à
  gauche. P4, `bug`.
- **I-5 (P1-8)** — PDF de rapports : « → » (période), « ↳ » (sous-projets) et « ⚠ » (bandeau du grand livre), hors
  WinAnsi, disparaissent à l'impression. P4, `bug`.
- **I-6 (P1-8)** — PDF des journaux : intitulé de compte non tronqué, peut recouvrir la colonne débit. P4, `bug`.
- **I-7 (décision de Beat)** — PDF de rapports : lire les libellés dans la langue de l'utilisateur (i18n complète), y
  compris les deux en-têtes neufs. P4, `enhancement`.

## Notes de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Fixed** — PDF des rapports (bilan, compte de résultat, balance, grand livre, journaux, décompte TVA, rapports par
  compte analytique) : les montants sont alignés à droite dans leur colonne, décimales l'une sous l'autre, totaux et
  soldes en devise compris ; ils étaient alignés à gauche.
- **Fixed** — PDF des rapports : le titre du document, dans ses propriétés, garde ses accents (« Compte de résultat »
  s'y lisait « Compte de rÃ©sultat », de même « Décompte TVA »).
- **Fixed** — PDF des rapports d'une société multi-devises : dès qu'un compte en devise y figure, les colonnes de
  montant disent leur devise — « Devise » et « CHF » au bilan et au compte de résultat, « Débit CHF », « Crédit CHF »…
  à la balance et au grand livre. Sans compte en devise, rien ne change.

## Dev Agent Record

### Agent Model Used

Opus 5.5.

### Recette

- Captures avant/après, 12 PDF chacune et leur rendu PNG (bilan, compte de résultat, balance, grand livre en mono et
  multi-devises ; journaux, TVA, dépenses et rendement par compte analytique en mono) :
  `kesh-gate-logs/fix-pdf-alignement/captures/`.
- `verifier-alignement.py` (bords droits mesurés par poppler, indépendant de la table de Kesh) : avant, 27 colonnes
  non alignées ; après, toutes alignées (`recette-avant.log`, `recette-apres.log`) ; PDF réels de 2024, avant : échec
  (`recette-avant-donnees-2024.log`).
- `comparer-textes.py` : mono-devise, textes identiques dans les 8 PDF ; multi-devises, seuls écarts les en-têtes
  « Devise », « CHF », « … CHF » (`comparer-textes.log`).
- `titres.log` (`pdfinfo`) : « Compte de rÃ©sultat » et « DÃ©compte TVA » avant, « Compte de résultat » et
  « Décompte TVA » après.

### Gates

**Gate complet groupé (2026-11-18)** : vert au commit `5fc6fb1b` de `integ/fixes-0.19.0` — chiffres au Change Log,
journaux dans `kesh-gate-logs/integ-fixes-0.19.0/`.

Voir `kesh-gate-logs/fix-pdf-alignement/` (relevés au rapport de l'agent).

### File List

- `Cargo.lock`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-qrbill/src/largeurs.rs`
- `crates/kesh-qrbill/src/lib.rs`
- `crates/kesh-report/Cargo.toml`
- `crates/kesh-report/src/pdf.rs`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/fix-pdf-rapports-mise-en-page.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
