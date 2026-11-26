# Story 46-3 : PDF et CSV du bilan et du compte de résultat par groupes

Status: done

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, D9 (étendue), Q2, Q3 ; FR-46-3, FR-46-8,
FR-46-9, FR-46-11 ; NFR-46-1, NFR-46-2. Release visée **0.21.0** (prévision). Approuvée par le responsable de l'epic et par Guy (revue 1,
2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Suit la 46-1 ; commence après la fusion de la 48-11
dans `main`** (arrondi au centime, Epic 48, release 0.20.0) — la PR #1327 (PDF aligné) est déjà fusionnée. Aucune
migration. **Corrigée le 2026-11-21, avant développement**, sur l'API réelle de la 46-1 (relevée sur `story/46-1` à
`de0a582f` : § *L'API de la 46-1*, Dev Notes) et sur la présentation miroir de la 47-3 (`story/47-3` à `b4d17d70`) ;
validation close le 2026-11-21 (P1 Sonnet, P2 Opus ; Change Log).

## Amendement du 22.11 : remplaçant de la 48-11

**Décision du responsable de l'epic (2026-11-22)** : la 46-3 n'attend plus la 48-11 (Epic 50, `backlog`, sans fiche au
2026-11-22 ; issue #1245). Elle pose un **remplaçant minimal et provisoire** de la fonction commune « arrondir au centime
puis formater », conçu pour que la 48-11 le remplace sans douleur :

- **une seule fonction**, dans `kesh-core` : `kesh_core::types::format_two_decimals(amount: Decimal) -> String`, dans
  `crates/kesh-core/src/types/money.rs` (le module qui porte `Money::round_to_centimes`, place que le fichier de
  l'Epic 48, § *48-11*, dit « naturelle »). Nom : celui que le fichier de l'Epic 46 annonce pour la 48-11
  (`format_two_decimals`, l. 144 et 176) ; signature : un `Decimal` exact en entrée, sa chaîne ISO à deux décimales en
  sortie (« arrondir au centime puis formater », ligne `48-11` de `sprint-status.yaml` et #1245). `round_for_display`,
  l'autre nom annoncé, n'est **pas** déplacé : il reste où il est (`pdf.rs`), la 48-11 le déplacera ;
- **arrondi explicite** : `round_dp_with_strategy(2, MidpointAwayFromZero)` — l'équidistant loin de zéro, comme le code
  existant de Kesh pour l'argent (`Money::round_to_centimes`, `amount_due_to_centime`, `round_for_display`) ; un zéro
  négatif (`-0.004`) sort `0.00`, sans signe, comme `round_for_display`. Formatage sans `{:.2}` (`rescale(2)` puis
  `to_string`) ;
- **utilisée par les seuls rendus touchés par la 46-3** : les rendus **groupés** du bilan et du compte de résultat,
  PDF (par un formateur suisse qui y ajoute les apostrophes des milliers) et CSV — corps, totaux de section, lignes
  calculées et équation de ces rendus. Les rendus **à plat** gardent leurs formateurs (`format_swiss_amount`,
  `format_amount_iso`) et leur `{:.2}`, inchangés (AC-2 : CSV identique à l'octet, PDF aux mêmes lignes) : c'est la
  48-11 qui les corrigera ;
- **aucun `{:.2}` neuf** : le séparateur des milliers est extrait de `format_swiss_amount` en une fonction partagée,
  que le formateur à plat et le formateur arrondi appellent ;
- doc-comment et note au fichier de l'epic : « **provisoire, à remplacer par la 48-11 à l'intégration** ». À
  l'intégration, la 48-11 garde sa propre fonction et rebranche dessus les deux formateurs groupés
  (`format_swiss_amount_arrondi` au PDF, l'appel direct au CSV), puis retire celle-ci ; ou, si elle retient le même nom
  et la même signature, il ne reste qu'un conflit de texte à résoudre en gardant la sienne.

Ce que cela change aux critères : l'AC-6 se lit sur cette fonction au lieu de celles de la 48-11 ; la garde de la 48-11
n'existe pas encore dans la branche (rien à garder vert), le contrôle « aucun `{:.2}` neuf » se fait par
`git diff | grep ':\.2}'` au Dev Agent Record. T0 : la condition « 48-11 dans `main`, sinon arrêt » est levée par cette
décision.

## Story

En tant que comptable,
je veux imprimer et exporter le bilan et le compte de résultat **tels que je les vois à l'écran** — même niveau, mêmes
groupes ouverts ou fermés, mêmes sous-totaux —,
afin de remettre un bilan par groupes à ma fiduciaire ou de le retravailler dans un tableur.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-report/pdf`, `kesh-report/csv`, `kesh-api/routes/reports` (paramètres des deux exports),
la documentation (`docs/api-external.md`, `docs/manual/fr/user-manual.tex`), les tests d'intégration
(`crates/kesh-api/tests/reports_export_e2e.rs`). Pas de `kesh-i18n` : les libellés du PDF sont ceux de
`PdfContext::fr_ch_default` (`crates/kesh-report/src/pdf.rs:294`, et `SectionLabels::fr_ch_defaults` `:204`, FR-CH en
dur, i18n des PDF différée).

## Critères d'acceptation

1. **AC-1 — Paramètres de l'export.** `GET /reports/balance-sheet/export` et `/income-statement/export`
   (`export_balance_sheet`, `export_income_statement`, `routes/reports.rs`) acceptent `detailLevel`, `subtotalPosition`,
   `open`, `closed`, pour `format=pdf` comme `format=csv`, **lus comme aux routes JSON de la 46-1** : un second
   extracteur `Query<Vec<(String, String)>>` et `groupes_de(&paires)` (même module, donc sans changement de visibilité),
   qui délègue à `DemandeGroupes::depuis_requete` puis `depuis_valeurs` du service — aucune validation recopiée.
   **Aucun champ n'est ajouté à `ExportQuery`** (revue P2 de la 46-1, L-6) : partagé avec les exports de la balance,
   des journaux et de la TVA, un champ `Option<String>` y ferait d'un paramètre répété un rejet de désérialisation
   d'Axum, sans `VALIDATION_ERROR`. Mêmes refus `400 VALIDATION_ERROR` que la 46-1, **paramètre répété compris**.
   Ordre des contrôles : `format` (`validate_format`, comme aujourd'hui), puis l'accès
   (`acces.exiger(rapports::EXIGENCES_BILAN.export)`, resp. `EXIGENCES_COMPTE_DE_RESULTAT.export`), puis les groupes —
   l'ordre des routes JSON (revue P1 de la 46-1, L-5) ; inobservable aujourd'hui (l'export est ouvert à tout rôle,
   `EXPORT_SANS_MCP`), donc sans test d'ordre. Les deux handlers appellent `rapports::bilan_par_groupes` et
   `rapports::compte_de_resultat_par_groupes` (au lieu de `bilan` et `compte_de_resultat`) avec cette demande : la
   présentation miroir de la 47-3, insérée dans ces deux fonctions avant le regroupement (46-1 AC-9), vaut donc pour
   l'export comme pour l'écran. Les exports de la balance, des journaux et de la TVA ne lisent pas ces paramètres
   (inchangés ; la balance groupée exportée est la 46-6b).
2. **AC-2 — Sans paramètre (NFR-46-1).** CSV **identique octet pour octet** et PDF aux **mêmes lignes de texte**
   (texte extrait par l'extracteur partagé créé par la story, G3 : voir AC-4) qu'avant la story, comparés à des fixtures écrites
   **au commit de base** (plan PME, compte en devise compris, deux exercices). Les sous-titres de rôle des fonds propres
   restent en mode à plat seulement. Sans `detailLevel`, `groupes_de` rend `None` : le service n'ajoute aucune requête
   (46-1 AC-10) et `grouping` est absent, ce qui fait choisir le rendu à plat.
3. **AC-3 — PDF fidèle à l'écran (responsable de l'epic ; remarque 4 du Project Lead).** Le PDF rend les lignes de
   `grouped_rows` (`SectionsBilan { assets, liabilities, equity }` ou `SectionsResultat { revenues, expenses }`, des
   `Vec<LigneRapport>`) dans l'ordre reçu, **avec la règle de rendu de la 46-1**, lue sur les champs réels de
   `LigneRapport` (`kind`, `depth`, `account_number`, `account_name`, `balance`, `currency`, `balance_currency`,
   `open`) : retrait par `depth` (pas fixe, borné pour que le nom ne recouvre pas les montants,
   troncature existante `truncate_with_ellipsis`) ; ligne `GenreLigne::Groupe` en gras, montant sur le titre si
   `grouping.subtotal_position` vaut `PositionSousTotal::Titre` ou si `open == Some(false)`, sinon sur sa ligne
   `GenreLigne::SousTotal` (« Total <nom du groupe> », gras ; même `account_name` et `balance` que la ligne du groupe) ;
   `GenreLigne::Propre` « <nom> (solde propre) » ; `GenreLigne::Compte` comme un compte d'aujourd'hui (la ligne d'un
   compte miroir de la 47-3 comprise : rien de propre au miroir, 47-3 C-47-3-3) ; devise et
   solde en devise d'une ligne de groupe à la place prévue par 37-4 (colonnes en devise si une ligne en porte) ;
   **garder-avec-la-suite** : un titre de groupe ouvert ne reste pas seul en bas de page
   (`ensure_space_for_rows(2.0)`, comme les sous-titres de rôle, `pdf.rs:877`). En-tête : une ligne sous la période
   dit le niveau (`grouping.detail_level` : `NiveauDetail::Niveau(2)` → « Niveau 2 », `NiveauDetail::Tous` → « Tous
   les comptes ») et, si `grouping.open` ou `grouping.closed` — les listes **retenues** par l'écho de la 46-1,
   identifiants ignorés retirés — n'est pas vide, ajoute « , détail ajusté à la main » — sans liste de numéros
   (C-46-3-1). Totaux de section, lignes calculées et
   équation : inchangés. **Rapport vide** : les fonctions groupées reprennent la garde « rapport vide » des fonctions à
   plat (bilan : `BalanceSheet::is_empty`, message « rapport vide » au PDF, garde par section au CSV ; compte de résultat : `revenues` et `expenses` tous deux vides, `pdf.rs:967`, `csv.rs:296`), car l'écran
   groupé étant celui par défaut (46-2 AC-1), l'export d'un bilan vide part désormais avec `detailLevel=all`.
4. **AC-4 — Fidélité prouvée.** Un test d'intégration demande le JSON (46-1) et le PDF avec **les mêmes paramètres**
   (niveau 2, un groupe ouvert à la main, un fermé, `footer`), et affirme que, **section par section**, la suite des
   lignes de texte du PDF entre le titre de la section et sa ligne de total est exactement celle des
   `groupedRows.<section>` (numéro, nom, montant formaté selon la règle de rendu) ; pour **Capitaux propres**, la
   suite attendue est `groupedRows.equity` **suivie** des lignes calculées que le rendu à plat écrit déjà entre les
   comptes et le total (`pdf.rs:896-923` : « Résultat reporté (calculé) » si `shows_retained_earnings_line()`, puis
   « Résultat de l'exercice », toujours ; hors de `groupedRows`, 46-1 D4), puis le total. Comparaison sur le **texte
   normalisé** : retrait de début de ligne et espaces répétés réduits. Le rendu groupé tronque aussi selon le retrait, et
   les libellés rendus sont plus longs que les noms (« Total <nom> » : +6 caractères, « <nom> (solde propre) » : +15) :
   le jeu d'essai est **choisi** (ce n'est pas le plan PME brut, dont `40` « Charges de matières et marchandises »
   compte déjà 35 caractères) pour que **chaque libellé tel que rendu** tienne dans la limite effective à sa profondeur,
   ce que le test affirme d'abord (sinon il normalise « … » des deux côtés, par la même fonction de troncature) ; aucun
   « … » n'entre alors dans la comparaison. **Extracteur** : `pdf_text` et `pdf_text_y` sont privés au `mod tests` de
   `kesh-report` ; la story crée un **extracteur partagé** (fonction de soutien de test exposée par `kesh-report`, ou
   module commun des tests de `kesh-api`, sans copie), qui reconstitue les lignes en groupant les `Tj` par ordonnée
   (`pdf_text_y`) ; le PDF n'est lisible que non compressé, c'est-à-dire sous `cargo test` ; pas de `pdftotext`
   (non garanti en CI). Sonde négative :
   ignorer `open` dans le PDF rougit le test. **Si la 47-3 est dans la branche d'epic au T0**, le même test pose aussi
   un compte miroir (sans parent) qui reçoit un solde, et un second miroir rangé sous un groupe d'**un autre type** que
   le sien (état admis, avertissement non bloquant : décision du responsable de l'Epic 47 du 2026-11-21) : le groupe et
   ses ancêtres sont dissous dans le JSON (46-1 AC-4) et le PDF les rend de même, sans mention propre (C-46-3-3) ;
   sinon ce cas revient au recâblage par le second arrivé (fiche 47-3, § *Recâblage*).
5. **AC-5 — CSV groupé.** En mode groupé seulement, deux colonnes après `Section` : `Niveau` (la `depth`) et
   `TypeLigne` (`groupe`, `compte`, `propre`, `soustotal`, pour `kind` = `group`, `account`, `own`, `subtotal`) ; une ligne par élément de `groupedRows`, montant dans
   `Solde` selon la règle de rendu (vide sur un titre de groupe ouvert en `footer`) ; colonnes en devise comme
   aujourd'hui (`Devise;SoldeDevise` si une ligne en porte) ; lignes de total de section et lignes calculées
   inchangées ; ni `mirrorOf` ni `role` ne sont exportés (47-3 C-47-3-3). En-tête de colonnes en mode groupé documenté
   dans `docs/api-external.md`.
6. **AC-6 — Arrondi (contrainte de Guy, 48-11).** Tout montant écrit passe par les fonctions de `kesh-core` posées
   par la 48-11 (`round_for_display`, `format_two_decimals`, ou les formateurs de `pdf.rs`/`csv.rs` qu'elle aura
   rebranchés) ; **aucun `{:.2}` neuf** dans `kesh-report` : la garde de la 48-11 reste verte. Les sous-totaux arrivent
   exacts de la 46-1 et ne s'arrondissent qu'à l'écriture ; un test sur un groupe de deux comptes à `10.005` et
   `5.006` affirme le sous-total imprimé `15.01` (arrondi de la somme, non somme des arrondis — écart assumé avec la
   somme visuelle des lignes, écrit au manuel).
7. **AC-7 — Pilotable sans écran (FR-46-11).** Tout ce que montre le PDF de l'écran s'obtient par les paramètres de
   l'export seuls ; le MCP (46-4) n'exporte pas de fichier et n'est pas concerné.
8. **AC-8 — Tests et documentation.** Tests unitaires de rendu (texte extrait) pour `header`, `footer`, groupe fermé,
   `own`, devise de groupe, saut de page avec titre gardé, rapport vide groupé, et **retrait croissant avec `depth`, borné** (abscisse du nom relevée comme le fait `Pose`, `pdf.rs:3260-3290`) ; intégration (`reports_export_e2e.rs`)
   des refus (ceux de la 46-1, sur les deux exports et les deux formats, **paramètre répété compris**), d'un paramètre
   de groupes répété **ignoré** par l'export de la balance (inchangé jusqu'à la 46-6b ; test demandé aussi par la 46-6a AC-9 : écrit par la première des deux stories fusionnée, l'autre le reprend), des fixtures sans paramètre
   (AC-2) et de l'AC-4 ; `docs/api-external.md` (paramètres de l'export, colonnes CSV) ;
   manuel utilisateur FR, § *Visualisation et exports* (`user-manual.tex:4188`) : export groupé, fidélité à l'écran,
   affichage de travail (lecture (a) de l'art. 959a CO), `.tex` seul ; la phrase « Aucun des deux ne perd de ligne,
   même quand l'écran en limite le nombre » (`user-manual.tex:4200-4202`) se restreint à l'export **à plat** ou nomme
   l'exception de l'export groupé (les comptes d'un groupe fermé n'y ont pas de ligne, leur somme est sur celle du
   groupe), et la liste des textes abrégés d'un « … » reste juste (aucun nom de groupe n'y entre si le retrait ne
   tronque pas ; sinon, l'y ajouter).

## Tâches / sous-tâches

- [x] **T0** — Reprise de `main` et de la branche d'epic : la **46-1** doit y être fusionnée (sinon arrêt) ; relever
      au Change Log, ligne lue citée, les noms du § *L'API de la 46-1* (Dev Notes) tels qu'ils sont dans la branche, et
      si la **47-3** est fusionnée (`grep -rn "present_balance_sheet" crates/kesh-api/src/services/rapports.rs`). La
      48-11 doit être dans `main` (sinon **arrêt**, le dire au Change Log) ; relever ses noms de
      fonctions exacts (annoncés `round_for_display`, `format_two_decimals`, garde `arrondi_centime_garde.rs` ; non
      relevés à l'écriture de la fiche : la fiche 48-11 était « à écrire » au 2026-11-20 et aucune branche ne la
      portait). Si la 48-11 sort de la 0.20.0 (ordre de sortie de `epic-48-corrections-0-20-0.md`), la 46-3 attend
      quand même sa fusion dans `main` — le responsable de l'epic et le Project Lead décident alors de l'ordre.
- [x] **T1 — fixtures** (AC-2) au commit de base, avant toute modification.
- [x] **T2 — route** (AC-1) : dans `export_balance_sheet` et `export_income_statement`, un second extracteur
      `Query(paires): Query<Vec<(String, String)>>`, `acces.exiger(… .export)?` puis `groupes_de(&paires)?`, et l'appel
      de `bilan_par_groupes` / `compte_de_resultat_par_groupes` ; **`ExportQuery` inchangé** (revue P2 de la 46-1,
      L-6). Le rendu choisi dans la closure d'`exporter` selon `report.grouped_rows` (`Some` : rendu groupé ; `None` :
      rendu à plat de `&report.rapport`, comme aujourd'hui).
- [x] **T3 — PDF** (AC 3, 4, 6) : deux fonctions **neuves** de rendu groupé, qui reçoivent l'enveloppe
      `&AvecGroupes<BalanceSheet, SectionsBilan>` / `&AvecGroupes<IncomeStatement, SectionsResultat>` de la 46-1
      (C-46-1-4 ; `kesh_report::groupes`, même crate) — `rapport` pour l'en-tête, les totaux, les lignes calculées et la
      garde « rapport vide », `grouping` pour la ligne de niveau et la position des sous-totaux, `grouped_rows` pour le
      corps ; `render_balance_sheet_pdf`
      (`pdf.rs:816`) et `render_income_statement_pdf` (`:954`) gardent **leur comportement**, leurs appelants et leurs
      tests : la route choisit l'une ou l'autre selon que `grouping` est présent (AC-2 tenu par construction, et
      prouvé par les fixtures). Libellés neufs dans `SectionLabels` et `fr_ch_defaults` ; une fonction de rendu de
      ligne groupée partagée par les deux rapports **et** par la balance (46-6b) ; l'en-tête, les totaux, les lignes
      calculées, l'équation et la garde « rapport vide » se **partagent** avec les fonctions à plat (extraits en
      fonctions communes, comportement inchangé ; zone de la 48-11, voir le tableau des fichiers communs de l'epic), seul le corps des sections diffère (DRY).
- [x] **T4 — CSV** (AC 5, 6) : de même, deux fonctions neuves à côté de `render_balance_sheet_csv` (`csv.rs:143`) et
      `render_income_statement_csv` (`:277`), inchangées.
- [x] **T5 — tests, documentation** (AC 8).
- [x] **T6 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`, après #1327 et #1346)

- `pdf.rs:84-97` `PdfContext`, `:102` `SectionLabels` (libellés FR-CH en dur), `:316` `format_swiss_amount`
  (contient aujourd'hui un `{amount:.2}`, `:318` — c'est le site que la 48-11 corrige), `:349` `round_for_display`
  (`pub(crate)`, déplacé par la 48-11), `:739` `any_foreign`, `:744` `draw_account_row` (nom tronqué si devise),
  `:816-950` bilan (sous-titres de rôle avec garder-avec-la-suite `:870-893`, `ensure_space_for_rows(2.0)` à `:877`,
lignes calculées, équation en pied),
  `:954` compte de résultat. Montants alignés à droite (`AMOUNT_RIGHT_MM`, `FX_AMOUNT_RIGHT_MM`, #1327).
- `csv.rs:41` `make_writer` (`;`, CRLF, BOM), `:50` `format_amount_iso` (`{amount:.2}` — site de la 48-11), `:101`
  `write_account_balance`, `:143-270` bilan (garde par section, `Total actifs;;;…`), `:277` compte de résultat.
- `routes/reports.rs`, à `29b9ec27` : `:174` `ExportQuery`, `:374` `export_balance_sheet`, `:504`
  `export_income_statement`, `:942` `PdfContext::fr_ch_default` ; sur `story/46-1` à `de0a582f` : `:203`
  `ExportQuery`, `:427` `export_balance_sheet`, `:559` `export_income_statement`, `:999` `PdfContext::fr_ch_default`. Les
  numéros de ligne de `pdf.rs` et `csv.rs` bougeront avec la 48-11 : se fier aux noms, relevés à nouveau au T0.

### L'API de la 46-1 (relevée sur `story/46-1` à `de0a582f`, à revérifier au T0 dans la branche d'epic)

- `kesh_report::groupes` : `AvecGroupes<T, S> { rapport: T, grouping: Option<Grouping>, grouped_rows: Option<S> }`
  (`#[serde(flatten)]` du rapport ; `sans_groupes`, `bilan`, `compte_de_resultat`) ; `SectionsBilan { assets,
  liabilities, equity }`, `SectionsResultat { revenues, expenses }`, chacune un `Vec<LigneRapport>` ; `LigneRapport {
  kind: GenreLigne, depth: u32, account_id, account_number, account_name, balance: Decimal, currency:
  Option<String>, balance_currency: Option<Decimal>, open: Option<bool>, active: Option<bool>, role:
  Option<Option<AccountRole>> }` (`mirror_of` s'y ajoute au recâblage de la 47-3, JSON seulement) ; `GenreLigne::{Groupe,
  Compte, Propre, SousTotal}` (sérialisés `group`, `account`, `own`, `subtotal`) ; `Grouping { detail_level:
  NiveauDetail::{Niveau(u8), Tous}, subtotal_position: PositionSousTotal::{Titre, Pied}, max_depth, open: Vec<i64>,
  closed: Vec<i64> }`. Les montants sont des `Decimal` exacts, jamais arrondis par la 46-1.
- `services/rapports.rs` : `DemandeGroupes { options }` et ses deux entrées `depuis_requete` (chaînes REST) et
  `depuis_valeurs` (valeurs typées, refus du contrat) ; `bilan_par_groupes(acces, lecture, demande,
  Option<DemandeGroupes>, usage)` et `compte_de_resultat_par_groupes(…)` rendent `Rapport<AvecGroupes<…>, U>` ;
  `bilan` et `compte_de_resultat` gardent leurs quatre arguments et délèguent avec `None` (écart E1 de la 46-1) ;
  aujourd'hui les deux exports appellent `bilan` / `compte_de_resultat` et rendent `&report.rapport`.
- `routes/reports.rs` : `PARAMETRES_GROUPES`, `groupes_de(&[(String, String)]) -> Result<Option<DemandeGroupes>,
  AppError>` (paramètre répété → 400 qui le nomme) ; les handlers JSON prennent `Query<ReportQuery>` **et**
  `Query<Vec<(String, String)>>`, contrôlent l'accès, puis lisent les groupes.
- Point d'insertion de la 47-3 : dans `bilan_par_groupes` et `compte_de_resultat_par_groupes`, entre le générateur et
  le `match groupes` — `present_balance_sheet(raw, &links)` / `present_income_statement` (47-3 AC-1) y présentent le
  rapport brut, et les groupes se calculent sur les sections présentées. Cette story n'y touche pas : en appelant les
  fonctions `*_par_groupes`, l'export hérite de la présentation et du regroupement tels quels.

### Ce qui doit rester

Les exports sans paramètre (AC-2), les exports des autres rapports, la ligne « Résultat reporté (calculé) » omise à
zéro (#1346), l'alignement (#1327).

### Choix et écarts

- **C-46-3-1** : le PDF imprime le **niveau**, et signale seulement qu'il y a des écarts manuels ; les groupes ouverts
  ou fermés se voient dans le corps (un groupe fermé porte son sous-total sans ses comptes). Simplifié à la validation
  P1 (KISS) : la liste des numéros, tronquée à une ligne, n'apprenait rien que le corps ne montre, et coûtait une
  troncature et un test de plus. Lecture retenue de la remarque 4 de Guy (« le PDF imprime le niveau et les groupes
  ouverts ou fermés ») : il imprime les groupes **dans l'état** ouvert ou fermé de l'écran.
- **C-46-3-2** : sous-total arrondi une fois, sur la somme exacte (AC-6), comme les totaux de section aujourd'hui.
- **C-46-3-3** (correction du 2026-11-21) : **aucun avertissement imprimé** pour un compte miroir rangé sous un groupe
  d'un autre type. La décision du responsable de l'Epic 47 (2026-11-21) admet cet état avec un avertissement **non
  bloquant**, dont le porteur n'est pas encore désigné (fichier d'epic 47, § *Décision … point e* : « personne à ce
  jour », issue `enhancement` à ouvrir) ; sa place naturelle est l'édition du plan, non un rapport remis à une
  fiduciaire. Le PDF et le CSV rendent l'effet tel que le JSON le donne (groupe dissous, totaux justes) et rien de
  plus. *Écarté* : une note de bas de page « groupe dissous par un compte miroir », qui demanderait au rendu une
  information que `groupedRows` ne porte pas.

### References

- Fiche 46-1 (§ *Le contrat*, Dev Agent Record : écarts E1 à E4, point d'insertion de la 47-3) ; rapports de revue de la
  46-1 (P1 L-4 et L-5, P2 L-6) ; fiche 47-3 (AC-1, AC-3, C-47-3-3, § *Recâblage*) et fichier de l'Epic 47 (§ *Décision
  … du 2026-11-21, point e*) ; fiche 48-11 et `epic-48-corrections-0-20-0.md` § *48-11* (garde `{:.2}`) ;
  `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel* (PDF compilé localement, aplati avant `grep`).

## Note de version

**Added** — Les exports PDF et CSV du bilan et du compte de résultat suivent l'écran par groupes de comptes
(`detailLevel`, `subtotalPosition`, `open`, `closed`, comme les routes JSON) : niveau, groupes ouverts ou fermés,
sous-totaux sur le titre ou en pied ; le PDF dit le niveau imprimé, décale chaque ligne selon sa profondeur et met les
groupes en gras ; le CSV gagne les colonnes `Niveau` et `TypeLigne`. Les montants de ces exports groupés s'arrondissent
au centime, l'équidistant loin de zéro. Sans paramètre, l'export par l'API rend la présentation de l'écran (tous les
groupes ouverts) ; `detailLevel=flat` rend l'export à plat d'avant (CSV identique à l'octet, PDF aux mêmes lignes de
texte). *(Annoté le 2026-11-24, story 46-8, décision du mainteneur : cette note disait « Sans paramètre, l'export ne
change pas ».)*

**Changed** — L'export d'écran suit la hiérarchie affichée, avec les colonnes `Niveau;TypeLigne` et les sous-totaux.
Le CSV groupé (bilan, compte de résultat ou balance) ne s'additionne pas tel quel : le titre d'un groupe (en pied, sa ligne de sous-total) répète les montants de ses comptes, et un groupe fermé n'y porte que son sous-total. Chaque section du bilan et du compte de résultat porte sa ligne de total, la balance sa ligne `Total` ; pour additionner les montants dans un tableur, décocher « Hiérarchie des groupes » pour l'export à plat d'avant, qui porte lui aussi ses lignes de total (à filtrer par `Section` au bilan et au compte de résultat, à écarter à la balance : ligne `Total`) et n'a pas de colonne `TypeLigne` ; les montants groupés sont arrondis au centime. Par l'API,
l'export d'avant, à l'octet, se demande par `detailLevel=flat` (story 46-8 ; cette note disait « Seul un appel de l'API
sans paramètre garde l'export d'avant », annoté le 2026-11-24).

### Choix du développement

- **C-46-3-4** — Une seule fonction de rendu par rapport et par format (`rendre_bilan_pdf`, `rendre_resultat_pdf`,
  `rendre_bilan_csv`, `rendre_resultat_csv`), avec `groupes: Option<(&Grouping, &Sections…)>` : à `None`, le rendu à
  plat, appel pour appel ; seul le corps des sections change, et le formateur des montants (T3, DRY). Les fonctions
  publiques à plat gardent leur signature, leurs appelants et leurs tests ; les fonctions groupées
  (`render_grouped_*`) prennent l'enveloppe `AvecGroupes` et retombent sur le rendu à plat quand elle n'a pas de
  groupes — la route les appelle toujours, et la sélection se fait là, en un seul endroit.
- **C-46-3-5** — Libellés : le titre d'un groupe porte son numéro ; la ligne de sous-total, « Total <nom> », n'en porte
  pas (comme les totaux de section) ; la ligne du solde propre porte le numéro du groupe. Au CSV, numéro et nom restent
  bruts sur toutes les lignes : `TypeLigne` dit le genre.
- **C-46-3-6** — Retrait de 4 mm par niveau, au plus 20 mm ; le numéro et le nom se décalent ensemble. Le rendu groupé
  tronque aussi le nom d'un compte CHF (48 caractères au retrait nul, moins un par 2,5 mm de retrait ;
  `limite_nom_groupe`), que le rendu à plat ne tronque pas : le retrait mange la place.
- **C-46-3-7** — Colonne en devise du rapport groupé : présente si une ligne **imprimée** porte une devise. Un compte
  en devise replié dans un groupe fermé de devises mélangées ne la fait pas apparaître : rien ne s'y écrirait.
- **C-46-3-8** — Le jeu PME de la 46-1 passe dans `crates/kesh-api/tests/support/jeu_pme.rs`, inclus par `#[path]`
  dans `reports_e2e.rs` et `reports_export_e2e.rs` (sans copie) ; les fixtures JSON de la 46-1 restent vertes.
- **C-46-3-9** — Comptes miroirs (AC-4, dernier alinéa) : la 47-3 n'est pas dans la branche d'epic au T0
  (`present_balance_sheet` n'apparaît qu'en commentaire, `services/rapports.rs:798`) ; ce cas revient au recâblage
  par le second arrivé (fiche 47-3, § *Recâblage*).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`), 2026-11-22.

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s46-3/` : `t1-ecrire.log` (fixtures au commit de base), `c1.log`, `c2.log`
(gates ciblés), `clippy-a.log`, `clippy-b.log`, `gate-backend-final.log` (gate final), `front-*.log`,
`mutations.log` (et `mutations-1-sans-touch.log`, `mutations-2.log`, runs antérieurs), `revue-p1-prompt.md`.

### Completion Notes List

- **T0** (relevé à `92434d16`, branche d'epic) : la 46-1 est fusionnée (`ed822f21`) ; noms du § *L'API de la 46-1*
  conformes — `groupes.rs:872` `AvecGroupes<T, S> { rapport, grouping, grouped_rows }`, `:785` `LigneRapport`, `:280`
  `GenreLigne`, `:747` `Grouping` ; `services/rapports.rs:786` `bilan_par_groupes(acces, lecture, demande, groupes,
  usage)`, `:822` `compte_de_resultat_par_groupes` ; `routes/reports.rs:93` `groupes_de(&[(String, String)])`. La
  **47-3 n'est pas fusionnée** : `grep -rn "present_balance_sheet" crates/kesh-api/src/services/rapports.rs` ne rend
  que le commentaire `:798` (C-46-3-9). La **48-11 n'est pas dans `main`** (`origin/main` `a1de7ea7`, ligne
  `48-11-arrondi-au-centime: backlog`) : condition d'arrêt levée par la décision du responsable de l'epic du 22.11
  (§ *Amendement du 22.11*).
- **T1** : fixtures `crates/kesh-api/tests/fixtures/rapports-groupes/export-{bilan,resultat}-{2025,2026}.{csv,pdf.txt}`
  écrites au commit `ded0a090`, dont le diff de `pdf.rs` ne touche que le test `pdf_text_y` (passé à l'extracteur
  partagé) : aucun rendu modifié. Le jeu PME de la 46-1 est passé, sans copie, dans `tests/support/jeu_pme.rs`
  (C-46-3-8) ; les fixtures JSON de la 46-1 restent vertes (`rapports_sans_parametre_identiques_a_la_fixture_de_base`).
- **T2** : `export_balance_sheet`, `export_income_statement` — second extracteur `Query<Vec<(String, String)>>`, ordre
  `validate_format` → `acces.exiger(… .export)` → `groupes_de`, appel des `*_par_groupes` ; `ExportQuery` inchangé.
- **T3, T4** : rendus groupés `render_grouped_{balance_sheet,income_statement}_{pdf,csv}` ; une seule fonction de
  rendu par rapport et par format (C-46-3-4). Libellés neufs dans `SectionLabels` (`grouped_level`,
  `grouped_all_accounts`, `grouped_manual_detail`, `grouped_subtotal_prefix`, `grouped_own_suffix`). AC-6 :
  `kesh_core::types::format_two_decimals` (amendement), `format_swiss_amount_arrondi` au PDF ; aucun `{:.2}` neuf
  (`git diff 92434d16..HEAD | grep '^+' | grep ':\.2}'` ne rend que deux commentaires qui le nomment).
- **T5** : `docs/api-external.md` § « Aux exports » ; manuel utilisateur FR, § *Visualisation et exports* — phrase
  « aucun des deux ne perd de ligne » restreinte à l'export à plat, liste des textes abrégés complétée, paragraphe
  « Le bilan et le compte de résultat par groupes », avertissement de l'affichage de travail (art. 959a CO). `.tex`
  seul ; PDF compilé dans une copie (`make user`, 3 passes, stable) et contrôlé aplati (`pdftotext … | tr '\n' ' '`) :
  les trois passages s'y lisent.
- **Tests ajoutés** (recomptés `grep -cE '^\s*#\[(test|sqlx::test)'`, de `92434d16` à `HEAD`) : **23** — `pdf.rs`
  56 → 67 (+11), `csv.rs` 25 → 31 (+6), `texte_pdf.rs` +1, `money.rs` 20 → 21 (+1), `reports_export_e2e.rs` 20 → 24
  (+4 : fixtures sans paramètre, fidélité JSON/PDF/CSV, refus, balance qui ignore les paramètres) ; plus un exemple de
  documentation (`format_two_decimals`). `reports_e2e.rs` inchangé (45).
- **Mutations** (copie `git archive` hors dépôt, cible à part supprimée ; « tuée » = test nommé en échec sur
  `panicked at`), au commit `b46981ba`, `mutations.log` : **9 sur 9 tuées** — M1 `open` ignoré au PDF
  (`bilan_groupe_footer_et_groupe_ferme`), M2 arrondi remplacé par une troncature (8 tests), M3 garder-avec-la-suite
  retiré (`bilan_groupe_titre_garde_avec_la_suite`), M4 retrait non borné (`bilan_groupe_retrait_croissant_et_borne`),
  M5 solde du titre ouvert en pied écrit au CSV (`bilan_csv_groupe_footer_et_ferme`), M6 écart manuel ignoré à
  l'en-tête (`bilan_groupe_footer_et_groupe_ferme`), M7 totaux du PDF groupé au formateur à plat
  (`bilan_groupe_totaux_arrondis_au_centime`), M9 colonnes `Niveau;TypeLigne` retirées (5 tests), M10 formateur à plat
  au CSV groupé (3 tests). Historique : au premier run, M7 **survivait** (aucun total à demi-centime) — d'où les deux
  tests `*_totaux_arrondis_au_centime` ; la garde du zéro négatif de `format_two_decimals` (M8) survivait aussi,
  mutation **équivalente** (`round_dp_with_strategy` rend déjà un zéro sans signe, le cas `-0.004` du test le tient) :
  garde retirée (`53ff5f29`). Le premier run (`mutations-1-sans-touch.log`) ne vaut pas : la restauration d'un fichier
  sans `touch` laissait à cargo le binaire muté de M2 pour les suivantes ; corrigé dans le script.
- **Gates** au dernier commit de code `b46981ba` : `scripts/test-fast.sh --ci` (base `kesh_s463` remise à zéro) **vert**
  — fmt, clippy `-D warnings`, nextest **6219 exécutés, 6219 passés**, 14 ignorés, un instable rejoué vert
  (`admin_auto_backup_e2e::disque_sature_pendant_l_ecriture`, connu, sans rapport) ; frontend complet vert à
  `ae03d401` (aucun fichier `frontend/` touché depuis, ni par la story) — `check`, `lint-i18n-ownership`, Vitest
  **4627** (278 fichiers), `build`. **E2E partiel : aucune spec** — la story n'ajoute aucune spec Playwright (règle de
  la nuit du 22.11) ; E2E complet et `gate-ci.sh` à la branche d'intégration de fin.
- **Hors périmètre** : la ligne `46-3` de `sprint-status.yaml` disait « commence après la fusion de la 48-11 » —
  levé par l'amendement ; à l'intégration de la 48-11, rebrancher `format_swiss_amount_arrondi` et le formateur du CSV
  groupé sur sa fonction, puis retirer `format_two_decimals` si elle ne la reprend pas (fichier d'epic, § dépendances).

- **Remédiation de la revue P1** (commit `5addbcc1`, rapport `kesh-gate-logs/s46-3/revue-p1-rapport.md`, C 0 / H 0 /
  M 3 / L 6) :
  - **M1** : `bilan_groupe_equation_arrondie_au_centime` (passif 10.004, résultat 0.001 : termes `10.00` et `0.00`,
    équation 10.005 → `10.01`, à plat `10.00`) et `compte_de_resultat_groupe_resultat_net_arrondi_au_centime`
    (110.005 → `110.01`, à plat `110.00`) ; doc-comment de `bilan_groupe_totaux_arrondis_au_centime` restreint aux
    totaux de section et aux lignes calculées (son équation vaut 10.010, que les deux formateurs écrivent `10.01`).
  - **M2** : `bilan_csv_groupe_echappe_les_cellules_texte` — numéros `-10`, `+1000` et noms `=1+1`, `@x` d'un groupe et
    d'un compte, attendus `'-10;'=1+1` et `'+1000;'@x`, montant `-5.00` non neutralisé.
  - **M3** : `bilan_groupe_libelles_longs_tronques_selon_le_retrait` (noms de 60 caractères aux profondeurs 1 à 4 :
    « … » à 48, 47, 45, 44 caractères, longueurs écrites en dur, non tirées de `limite_nom_groupe`) et
    `bilan_groupe_libelle_long_en_devise_tronque_plus_court` (34 et 33 en devise).
  - **L1** : garde du zéro négatif rétablie dans `format_two_decimals` (seul changement de production) — la mutation
    « équivalente » M8 du développement ne l'était pas : `-dec!(0.00)` traverse `round_dp_with_strategy` signé ; cas
    `-0.00` et `-0.0000` ajoutés au test, doc-comment et commentaire corrigés.
  - **L3** : manuel rendu exact plutôt que code aligné sur le rendu à plat (moins risqué : aucun rendu changé, et la
    limite protège la colonne des montants d'un nom décalé) — « par groupes, tout libellé de plus de 48 caractères —
    34 sur une ligne en devise —, un de moins par 2,5 mm de retrait, même celui d'un compte que l'export à plat n'abrège
    pas ». PDF compilé dans une copie (`make user` après `touch`), aucun « Overfull », phrase relue au texte aplati.
  - **L6** : `bilan_csv_groupe_devise_footer` — titre ouvert en `footer` d'un groupe en devise : solde, devise et solde
    en devise vides ; le sous-total les porte.
  - **Notés, sans correctif** : **L2** (la marge de `GROUPE_NOM_MAX_CHARS` ne vaut que pour la capitale moyenne : un nom
    de 48 `M` ou `W` gras au plus long montant recouvrirait celui-ci ; même limite que le rendu à plat de la 37-4 en
    devise, pas de régression ; à reprendre si une mesure par chasse remplace le compte de caractères) ; **L4**
    (`texte_pdf::lignes` ne distingue pas les pages : deux textes de même ordonnée en fin et en début de page
    fusionneraient ; extracteur de test seulement, aucun cas réel) ; **L5** (le test de fidélité JSON/PDF n'exerce ni
    `own`, ni `header`, ni de groupe en devise ; ces genres restent gardés par les tests unitaires).
  - **Tests ajoutés par la remédiation** (recomptés `grep -cE '^\s*#\[(tokio::)?(test|sqlx::test)'`, de `b46981ba` à
    `5addbcc1`) : **6** — `pdf.rs` 67 → 71, `csv.rs` 31 → 33 ; `money.rs` 21 inchangé (deux cas ajoutés à un test
    existant). De `92434d16` à `5addbcc1` : **29**.
  - **Mutations** des neuf de la revue (copie `git archive` de `5addbcc1` hors dépôt, cible `kesh-wt46s3-mut` à part,
    supprimées ; témoin non muté vert, 40 passés ; « tuée » = `panicked at` + nom), `mutations-p1.log` : **9 sur 9
    tuées** — A troncature retirée et B retrait ignoré (les deux tests de troncature), C limite en devise ignorée
    (`…_en_devise_tronque_plus_court`), E et F `txt()` retiré du numéro ou du nom (`bilan_csv_groupe_echappe_les_cellules_texte`),
    I devise d'un titre sans montant (`bilan_csv_groupe_devise_footer`), L équation au formateur à plat
    (`bilan_groupe_equation_arrondie_au_centime`), M résultat net au formateur à plat
    (`compte_de_resultat_groupe_resultat_net_arrondi_au_centime`), Z garde du zéro négatif retirée
    (`format_two_decimals_arrondit_puis_formate`).
  - **Gates** au dernier commit de code `5addbcc1` : `cargo fmt` sans changement, clippy workspace `-D warnings` vert
    (`clippy-p1.log`) ; `scripts/test-fast.sh --ci` (base `kesh_s463` remise à zéro) **vert**, nextest **6225 exécutés,
    6225 passés**, 14 ignorés (`gate-backend-p1.log`). Frontend non rejoué : aucun fichier `frontend/` touché. Aucun E2E
    (aucune spec neuve), pas de `gate-ci.sh` (fin d'epic).

### File List

- `crates/kesh-core/src/types/money.rs`, `crates/kesh-core/src/types/mod.rs` — `format_two_decimals` (provisoire).
- `crates/kesh-report/src/pdf.rs` — rendus groupés, `format_swiss_amount_arrondi`, `grouper_milliers`,
  `draw_header_with`, `draw_totals_footer_with`, `LibelleGroupe`, `limite_nom_groupe`, libellés neufs, tests.
- `crates/kesh-report/src/csv.rs` — rendus groupés, `Disposition`, tests.
- `crates/kesh-report/src/texte_pdf.rs` (neuf) — extracteur de texte PDF partagé des tests.
- `crates/kesh-report/src/essai_groupes.rs` (neuf, `cfg(test)`) — jeu d'essai des rendus groupés.
- `crates/kesh-report/src/lib.rs` — modules et réexports.
- `crates/kesh-api/src/routes/reports.rs` — paramètres de groupes aux deux exports.
- `crates/kesh-api/tests/support/jeu_pme.rs` (neuf), `crates/kesh-api/tests/reports_e2e.rs`,
  `crates/kesh-api/tests/reports_export_e2e.rs`, `crates/kesh-api/tests/fixtures/rapports-groupes/export-*` (8).
- `docs/api-external.md`, `docs/manual/fr/user-manual.tex`.
- `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`,
  cette fiche.

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : fidélité à l'écran prouvée par test (remarque 4 de Guy), niveau et
  écarts imprimés, règle de rendu de la 46-1, dépendance à la 48-11 et interdiction du `{:.2}` (contrainte de Guy).
  Huit critères. `drafted`.
- 2026-11-20 — Validation P1 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 1 M
  / 4 L, remédiés. M : les rendus groupés greffés dans `render_*_pdf`/`render_*_csv` lisaient des champs de
  `BalanceSheet` que la 46-1 n'ajoute plus (enveloppe `AvecGroupes<T, S>`, C-46-1-4) → fonctions neuves, rendus à plat
  inchangés avec leurs tests (T3, T4). L : en-tête du PDF simplifié (niveau et « détail ajusté à la main », sans liste
  tronquée, C-46-3-1, AC-3) ; fidélité de l'AC-4 vérifiée section par section ; noms de la 48-11 annoncés, non
  relevés, et cas d'une 48-11 sortie de la 0.20.0 (T0) ; lignes citées corrigées (`fr_ch_default` `:294`,
  garder-avec-la-suite `:877`). Rapport : `kesh-gate-logs/epic-46/validation/46-3-P1.md`.
- 2026-11-20 — Validation P2 (Claude Sonnet 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 1 M
  / 1 L, remédiés. Lignes vérifiées : `pdf.rs:294`, `:399`, `:487`, `:816`, `:877`, `:954`, `csv.rs:143`, `:277`,
  `routes/reports.rs:174` (`ExportQuery` : ni `deny_unknown_fields` ni les quatre champs), `user-manual.tex:4188`. M :
  les fonctions groupées neuves omettaient la garde « rapport vide », alors que l'export d'un bilan vide part avec
  `detailLevel=all` depuis la 46-2 (AC-3, test `reports-export-pdf.spec.ts:40`). L : en-tête, totaux, équation et garde
  partagés avec les rendus à plat (T3, DRY) ; nom de l'enveloppe à deux paramètres. Rapport :
  `kesh-gate-logs/epic-46/validation/46-3-P2.md`.
- 2026-11-20 — Validation P3 ciblée (Claude Haiku 5.5, contexte frais, lentille sur le seul commit `8a7f0f73`, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 0 M / 2 L, remédiés. L : « `BalanceSheet::is_empty` et son pendant » ne désignait aucune méthode pour le compte de résultat (garde précisée : `revenues` et `expenses` vides, `pdf.rs:967`, `csv.rs:296`) ; T3 disait les fonctions à plat « inchangées » tout en extrayant leurs parties communes (comportement, appelants et tests inchangés ; zone de la 48-11). Rapport : `kesh-gate-logs/epic-46/validation/46-123-P3-ciblee.md`.
- 2026-11-21 — **Corrections avant développement** (orchestrateur de l'epic, sur les rapports de revue de la 46-1 et
  l'API relevée sur `story/46-1` à `de0a582f`, et la fiche 47-3 à `b4d17d70`). AC-1 et T2 : paramètres lus par
  `groupes_de` et un second extracteur `Query<Vec<(String, String)>>`, `ExportQuery` inchangé (revue P2 de la 46-1,
  L-6 — la consigne posée sur `story/46-1` par `de0a582f` disait encore « `ExportQuery` gagne les quatre champs » en
  tête de T2 et « à passer en `pub(crate)` », inutile : les exports sont dans le même module), paramètre répété refusé,
  ordre format → accès → groupes (P1 L-5), appel de `bilan_par_groupes` / `compte_de_resultat_par_groupes` ; AC-3 et
  T3 : champs réels de `LigneRapport`, `GenreLigne`, `Grouping` (`NiveauDetail`, `PositionSousTotal`, listes retenues)
  ; AC-4 : cas des comptes miroirs, dont un miroir sous un groupe d'un autre type (décision du responsable de l'Epic 47
  du 2026-11-21) ; C-46-3-3 : aucun avertissement imprimé ; Dev Notes : § *L'API de la 46-1*, lignes de
  `routes/reports.rs` à la 46-1 ; T0 : 46-1 fusionnée exigée, 47-3 relevée. Validation à reprendre (P1 d'une nouvelle
  boucle, prompt `kesh-gate-logs/s46-specs/validation-p1-prompt.md`).
- 2026-11-21 — **Validation P1 de la nouvelle boucle** (Claude Sonnet 5.5, contexte frais, à `fe8f0eeb`) : 0 C / 0 H /
  1 M / 3 L sur cette fiche, remédiés. M (F1) : l'AC-4 comparait la section Capitaux propres à `groupedRows.equity`
  seul, alors que le PDF écrit les deux lignes calculées entre les comptes et le total (`pdf.rs:896-925`) → suite
  attendue `groupedRows.equity` puis les lignes calculées. L : texte normalisé et jeu d'essai à noms courts, sans
  troncature (F3, AC-4) ; phrase du manuel « aucun des deux ne perd de ligne » à restreindre à l'export à plat (F4,
  AC-8) ; lignes de `routes/reports.rs` séparées par commit (F5). Rapport :
  `kesh-gate-logs/s46-specs/validation-p1-rapport.md`.
- 2026-11-21 — **Validation** (boucle close) : P1 (Claude Sonnet 5.5, 0 C / 0 H / 0 M au-dessus de LOW ; tendance 0/0/2/10 → 0/0/0/10) puis P2 (Claude Opus 5.5, contexte frais, à `e61c9029`) 0 C / 0 H / 0 M / 10 L sur les trois fiches (46-3 : 4, 46-6a : 5, 46-4 : 1) ; aucun finding au-dessus de LOW. LOW de la P2 appliqués dans la fiche : G1 (jeu d'essai et libellés rendus), G2 (test du retrait par `depth` ajouté à l'AC-8), G3 (extracteur partagé créé par la story), G4 (`csv.rs:41`), G7 (test d'ignorance demandé une seule fois), G5 (note de version). Rapport : `kesh-gate-logs/s46-specs/validation-p2-rapport.md`. `ready-for-dev`.
- 2026-11-22 — **Développement** (Claude Opus 5.5, `bmad-dev-story`) : amendement du 22.11 (remplaçant provisoire de
  la 48-11) écrit avant le code ; fixtures au commit de base (`ded0a090`) ; route, rendus groupés PDF et CSV, arrondi,
  documentation ; 23 tests ; gates au dernier commit de code `b46981ba` (Dev Agent Record). Une mutation survivante
  (totaux des rendus groupés au formateur à plat) a fait ajouter deux tests ; une mutation équivalente (garde du zéro
  négatif) a fait retirer la garde. Story en `review` ; prompt de la revue P1 : `kesh-gate-logs/s46-3/revue-p1-prompt.md`.
- 2026-11-22 — **Revue de code P1** (Claude Opus 5.5, complète, une lentille, contexte frais) : 0 C / 0 H / 3 M / 6 L —
  trois gardes de test manquantes (mutations survivantes : arrondi de l'équation et du résultat net groupés,
  neutralisation des formules au CSV groupé, troncature des libellés groupés), pas de défaut de rendu. **Remédiation**
  (`5addbcc1`) : M1, M2, M3, L1 (garde du zéro négatif rétablie, seul changement de production), L3 (manuel), L6
  corrigés ; L2, L4, L5 notés au Dev Agent Record ; six tests, neuf mutations tuées, gate nextest 6225/6225. Prompt de
  la passe P2 (complète, Opus) : `kesh-gate-logs/s46-3/revue-p2-prompt.md`. Tendance : P1 0/0/3/6.
- 2026-11-22 — **Revue de code P2** (Claude Opus 5.5, complète, une lentille, contexte frais, à `ae37d9d5`) : 0 C / 0 H /
  0 M / 6 L, la remédiation de la P1 relue et vérifiée ; boucle close. **Remédiation sans toucher la production** :
  L1 (CSV groupé : `bilan_csv_groupe_chaque_section_porte_ses_lignes`, `compte_de_resultat_csv_groupe`), L2 (colonne en
  devise d'un groupe fermé en devises mélangées : `bilan_csv_groupe_ferme_en_devises_melangees_sans_colonne_en_devise`,
  `bilan_groupe_ferme_en_devises_melangees_sans_colonne_en_devise`), L4 et L5 (doc-comments de `write_grouped_rows`,
  `format_two_decimals`, `api-external.md`, manuel : limites par niveau de retrait), L6 (doc-comment de
  `GROUPE_NOM_MAX_CHARS` nuancé) ; quatre tests neufs ; cinq mutations jouées une à une dans une copie hors dépôt
  (`kesh-gate-logs/s46-3/mutations-p2.log`) : N7, N7b, N13, N14, N15, toutes tuées sur une assertion nommée. L3 (suffixe
  « (solde propre) » tronqué, production) et les L2, L4, L5 de la P1 sont écrites dans
  `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`, à ouvrir. Tendance : P1 0/0/3/6 → P2 0/0/0/6. Modèles : Opus 5.5
  (développement, P1 et P2).
  Gates au dernier commit de code (`dd144a72`, base remise à zéro, journaux `kesh-gate-logs/s46-3/*-p2.log`) : fmt et
  clippy workspace verts ; nextest `package(kesh-report) | package(kesh-core) | binary(reports_export_e2e) |
  binary(reports_e2e) | binary(textes_coherents)` : 799 passés, 1 ignoré ; manuel compilé dans une copie, phrase des
  libellés présente au PDF, aucun « Overfull ».
- 2026-11-26 — **Suivis ouverts** (dette de la story, non réglée par la PR de l'epic) : #1637 (L2 de la revue P1 : la limite de 48 caractères ne vaut que pour la capitale moyenne) ; #1638 (L4 de la revue P1 : extraction du texte d'un PDF entre deux pages) ; #1639 (L5 de la revue P1 : le test de fidélité JSON/PDF ne joue qu'un jeu de paramètres).
