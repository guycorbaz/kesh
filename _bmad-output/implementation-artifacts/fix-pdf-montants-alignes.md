# Correctif : montants des PDF de rapports alignés à droite

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/pdf-montants-alignes`, partie de `main` à `70be12a6` (squash de l'Epic 37,
PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` (aucune publication intermédiaire) et va
à la prochaine release planifiée. **Aucune migration.** Constaté par Beat sur les PDF de la recette 2024
(`data-not-committed/2024-qesh/sorties-kesh/`, bilan et compte de résultat).*

## Intention

Dans les PDF des rapports (`crates/kesh-report/src/pdf.rs`), chaque montant commençait à l'abscisse de sa colonne :
aligné à **gauche**, ses décimales ne s'alignaient pas d'une ligne à l'autre (« 2'168.17 » au-dessus de « 4.67 »). Cela
valait pour la colonne en devise principale et pour le solde en devise (« EUR 17'072.20 ») ajouté par la Story 37-4. Le
correctif fait finir chaque montant au **bord droit** de sa colonne : les décimales tombent l'une sous l'autre.

## Critères d'acceptation

1. **AC-1 — Bord droit commun.** Dans tout PDF de rapport, chaque montant d'une colonne finit au même bord droit, quelle
   que soit sa longueur, son signe, sa graisse (totaux et sous-totaux en gras) : bilan, compte de résultat, balance,
   grand livre, journaux, décompte TVA, dépenses et rendement par compte analytique.
2. **AC-2 — Devise.** Le solde en devise du bilan et du compte de résultat (« USD 1'233.57 ») est aligné à droite sur
   son propre bord, code et montant lisibles ; au grand livre, le solde en devise des lignes d'encadrement (ouverture,
   report, rupture, clôture) aussi.
3. **AC-3 — Mesure.** La position se calcule par la métrique de la police (Helvetica et Helvetica-Bold, chasses AFM en
   WinAnsi), non par un nombre de caractères ; une seule fonction d'alignement, partagée (DRY), documentée.
4. **AC-4 — Contenu inchangé.** Mêmes chaînes écrites, dans le même ordre : seules les abscisses changent. Les secondes
   lignes en petit (phrases du grand livre et de la balance) restent alignées à gauche.
5. **AC-5 — Pas de chevauchement.** Le plus long montant plausible (« -12'345'678.90 ») ne recouvre ni la colonne qui le
   précède ni la marge droite (15 mm, symétrique de la gauche).
6. **AC-6 — Tests.** Unitaires sur la position calculée ; de bout en bout sur le PDF rendu ; tests PDF existants verts.
7. **AC-7 — Recette visuelle.** Captures avant/après de chaque rapport touché, mono-devise et multi-devises
   (`kesh-gate-logs/fix-pdf-alignement/captures/`).

## Hors périmètre

- Les **factures** (`kesh-qrbill`) partagent le défaut — montants du tableau des lignes, du récapitulatif TVA et du total
  écrits à l'abscisse de leur colonne (`crates/kesh-qrbill/src/pdf.rs`, `col_qty`, `col_unit`, `col_vat`, `col_tot`) —
  mais ne sont pas touchées : à tracer à part.
- Relevés à la revue des PDF de 2024, non traités : titre des propriétés du PDF mal encodé (« Compte de rÃ©sultat ») ;
  aucun en-tête de colonne (« CHF ») au bilan et au compte de résultat ; ligne « Résultat reporté (calculé) » ; flèche
  « → » de la période et « ↳ » des sous-projets, hors WinAnsi, qui disparaissent à l'impression.

## Tâches

- [x] `right_aligned_x` (bord droit − largeur imprimée) et `PdfBuilder::write_right`, partagés par toutes les colonnes de
  montant ; la largeur vient de `kesh_qrbill::largeur_mm`, désormais exportée (table engendrée de la Story 36-12a).
- [x] Bords des colonnes : constantes nommées, documentées, avec la mesure du plus long montant.
- [x] Huit rapports convertis (bilan, compte de résultat, balance, grand livre, journaux, TVA, dépenses et rendement par
  compte analytique), en-têtes des colonnes numériques compris (balance, grand livre, TVA, rendement).
- [x] Tests unitaires et de bout en bout ; mutation (alignement à gauche) qui les fait rougir.
- [x] Recette indépendante par `pdftotext -bbox-layout` et comparaison des textes avant/après.
- [x] Manuel utilisateur (`.tex` seulement) : une phrase à la note des exports PDF.

## Notes de développement

- **Choix de la source des largeurs.** La table des chasses d'Helvetica existe déjà, engendrée et vérifiée
  (`crates/kesh-qrbill/src/largeurs.rs`, Story 36-12a). Plutôt que de la recopier, `largeur_mm` devient publique et
  `kesh-report` dépend de `kesh-qrbill` (déjà au graphe par `kesh-db`). ⚠️ Écart à relire : la décision 12 de
  l'architecture sépare les PDF des deux crates ; seule une table de métriques est partagée, non une couche de rendu.
  L'autre voie — déplacer la table dans `kesh-core` — toucherait le script `engendrer-largeurs-helvetica.py` et les
  tests propres à la QR-facture de `largeurs.rs` ; elle n'a pas été prise.
- **Chiffres de même chasse.** Les dix chiffres d'Helvetica valent 556 millièmes d'em en maigre comme en gras ; le point
  aussi d'une ligne à l'autre ; l'apostrophe (191 / 238) et le signe moins (333), non : c'est pourquoi la largeur est
  mesurée, et non déduite du nombre de caractères. Test : `les_chiffres_d_helvetica_ont_une_meme_chasse`.
- **Bords retenus** (mm depuis le bord gauche de la page A4) : montants en devise principale et totaux 195 (marge
  droite) ; solde en devise du bilan et du compte de résultat 160 ; balance 117, 143, 169, 195 ; journaux 160 et 195 ;
  TVA 150 et 195 ; rendement 108, 138, 168, 195 ; grand livre (8 pt) 149, 172, 195, et 127 pour le solde en devise des
  lignes d'encadrement.

## Change Log

- 2026-11-18 — Implémentation (`4de1b96b` code, `bd087f4a` tests), base `70be12a6`. Neuf tests ajoutés à
  `crates/kesh-report/src/pdf.rs` (`grep -c '#\[test\]'` : 41 à `70be12a6`, 50 à `bd087f4a`). Mutation : `write_right`
  ramené à une abscisse fixe → les six tests de bout en bout rougissent, les trois unitaires restent verts (ils ne
  passent pas par le rendu).

## Notes de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Fixed** — PDF des rapports (bilan, compte de résultat, balance, grand livre, journaux, décompte TVA, rapports par
  compte analytique) : les montants sont alignés à droite dans leur colonne, décimales l'une sous l'autre, totaux et
  soldes en devise compris ; ils étaient alignés à gauche.

## Dev Agent Record

### Agent Model Used

Opus 5.5.

### Recette

- Captures avant/après, 12 PDF chacune (bilan, compte de résultat, balance, grand livre en mono et multi-devises ;
  journaux, TVA, dépenses et rendement par compte analytique en mono) : `kesh-gate-logs/fix-pdf-alignement/captures/`.
- `verifier-alignement.py` (bords droits mesurés par poppler, indépendant de la table de Kesh) : avant, 27 colonnes non
  alignées ; après, toutes alignées (`recette-avant.log`, `recette-apres.log`) ; PDF réels de 2024, avant : échec
  (`recette-avant-donnees-2024.log`).
- `comparer-textes.py` : mêmes textes, même ordre, dans les 12 PDF (`comparer-textes.log`).

### Gates

Voir `kesh-gate-logs/fix-pdf-alignement/` (relevés au rapport de l'agent).

### File List

- `Cargo.lock`
- `crates/kesh-qrbill/src/largeurs.rs`
- `crates/kesh-qrbill/src/lib.rs`
- `crates/kesh-report/Cargo.toml`
- `crates/kesh-report/src/pdf.rs`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/fix-pdf-montants-alignes.md`
